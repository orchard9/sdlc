use axum::extract::{Query, State};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use crate::error::AppError;
use crate::state::{AppState, SseMessage};

use super::runs::{sdlc_query_options, spawn_agent_run};

/// Per-directory count of changed files.
#[derive(Debug, Serialize, Clone)]
pub struct DirectoryCount {
    pub directory: String,
    pub count: u32,
}

/// Composite git repository status with computed severity.
#[derive(Debug, Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub dirty_count: u32,
    pub staged_count: u32,
    pub untracked_count: u32,
    pub ahead: u32,
    pub behind: u32,
    pub has_conflicts: bool,
    pub conflict_count: u32,
    pub severity: String,
    pub summary: String,
    pub directory_counts: Vec<DirectoryCount>,
}

/// A single file entry with its git status.
#[derive(Debug, Serialize, Clone)]
pub struct GitFileEntry {
    pub path: String,
    pub status: String,
    pub staged: bool,
    pub unstaged: bool,
}

/// Query parameters for GET /api/git/files.
#[derive(Debug, Deserialize)]
pub struct GitFilesQuery {
    #[serde(default)]
    pub include_clean: bool,
}

/// GET /api/git/status — collect git repo health and return composite status.
pub async fn get_git_status(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || collect_git_status(&root))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

    match result {
        Ok(status) => Ok(Json(
            serde_json::to_value(status)
                .map_err(|e| AppError(anyhow::anyhow!("serialize error: {e}")))?,
        )),
        Err(e) if is_not_git_repo(&e) => {
            Ok(Json(serde_json::json!({ "error": "not_a_git_repo" })))
        }
        Err(e) => Err(AppError(e)),
    }
}

/// Check whether an error indicates the directory is not a git repository.
fn is_not_git_repo(err: &anyhow::Error) -> bool {
    err.to_string().contains("not_a_git_repo")
}

/// Run git commands and collect status into a `GitStatus` struct.
fn collect_git_status(root: &Path) -> Result<GitStatus, anyhow::Error> {
    // First verify this is a git repo
    let check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !check.status.success() {
        return Err(anyhow::anyhow!("not_a_git_repo"));
    }

    // Get porcelain v2 status with branch info
    let output = Command::new("git")
        .args(["status", "--porcelain=v2", "--branch"])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git status: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("git status failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    parse_porcelain_v2(&stdout)
}

/// Parse `git status --porcelain=v2 --branch` output into a `GitStatus`.
pub(crate) fn parse_porcelain_v2(output: &str) -> Result<GitStatus, anyhow::Error> {
    let mut branch = String::from("HEAD");
    let mut ahead: u32 = 0;
    let mut behind: u32 = 0;
    let mut dirty_count: u32 = 0;
    let mut staged_count: u32 = 0;
    let mut untracked_count: u32 = 0;
    let mut conflict_count: u32 = 0;
    let mut dir_map: HashMap<String, u32> = HashMap::new();

    for line in output.lines() {
        if let Some(rest) = line.strip_prefix("# branch.head ") {
            branch = rest.to_string();
        } else if let Some(rest) = line.strip_prefix("# branch.ab ") {
            // Format: +N -M
            let parts: Vec<&str> = rest.split_whitespace().collect();
            if parts.len() >= 2 {
                ahead = parts[0]
                    .strip_prefix('+')
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
                behind = parts[1]
                    .strip_prefix('-')
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0);
            }
        } else if line.starts_with("1 ") || line.starts_with("2 ") {
            // Ordinary or renamed/copied entry — format: "1 XY ..." or "2 XY ..."
            let xy = extract_xy(line);
            if let Some((x, y)) = xy {
                if x != '.' {
                    staged_count += 1;
                }
                if y != '.' {
                    dirty_count += 1;
                }
            }
            if let Some(path) = extract_file_path(line) {
                let dir = parent_dir(&path);
                *dir_map.entry(dir).or_insert(0) += 1;
            }
        } else if line.starts_with("u ") {
            // Unmerged (conflict) entry
            conflict_count += 1;
            if let Some(path) = extract_conflict_path(line) {
                let dir = parent_dir(&path);
                *dir_map.entry(dir).or_insert(0) += 1;
            }
        } else if line.starts_with("? ") {
            // Untracked file
            untracked_count += 1;
            if let Some(path) = line.strip_prefix("? ") {
                let dir = parent_dir(path);
                *dir_map.entry(dir).or_insert(0) += 1;
            }
        }
    }

    let has_conflicts = conflict_count > 0;
    let severity = compute_severity(dirty_count, behind, untracked_count, has_conflicts);
    let summary = build_summary(
        dirty_count,
        staged_count,
        untracked_count,
        ahead,
        behind,
        conflict_count,
    );

    let mut directory_counts: Vec<DirectoryCount> = dir_map
        .into_iter()
        .map(|(directory, count)| DirectoryCount { directory, count })
        .collect();
    directory_counts.sort_by(|a, b| b.count.cmp(&a.count).then(a.directory.cmp(&b.directory)));

    Ok(GitStatus {
        branch,
        dirty_count,
        staged_count,
        untracked_count,
        ahead,
        behind,
        has_conflicts,
        conflict_count,
        severity: severity.to_string(),
        summary,
        directory_counts,
    })
}

/// Extract the XY status flags from a porcelain v2 ordinary or renamed entry.
/// Line format: "1 XY ..." or "2 XY ..."
fn extract_xy(line: &str) -> Option<(char, char)> {
    // The XY field is the second space-separated token
    let mut parts = line.split_whitespace();
    parts.next()?; // skip the type marker ("1" or "2")
    let xy = parts.next()?;
    let mut chars = xy.chars();
    let x = chars.next()?;
    let y = chars.next()?;
    Some((x, y))
}

/// Extract file path from a porcelain v2 ordinary ("1 ...") or renamed ("2 ...") entry.
/// For ordinary entries the path is the 9th space-separated field.
/// For renamed entries the destination path follows the tab separator.
fn extract_file_path(line: &str) -> Option<String> {
    if line.starts_with("2 ") {
        // Renamed/copied: destination is after tab
        line.split('\t').nth(1).map(|s| s.to_string())
    } else {
        // Ordinary: path is field index 8
        line.split_whitespace().nth(8).map(|s| s.to_string())
    }
}

/// Extract file path from a porcelain v2 unmerged entry ("u ...").
/// The path is the 11th space-separated field (index 10).
fn extract_conflict_path(line: &str) -> Option<String> {
    line.split_whitespace().nth(10).map(|s| s.to_string())
}

/// Return the immediate parent directory of a file path, or `"."` for root-level files.
fn parent_dir(path: &str) -> String {
    match path.rfind('/') {
        Some(pos) => path[..pos].to_string(),
        None => ".".to_string(),
    }
}

/// Compute severity level based on status fields.
fn compute_severity(
    dirty_count: u32,
    behind: u32,
    untracked_count: u32,
    has_conflicts: bool,
) -> &'static str {
    if has_conflicts || behind > 10 {
        "red"
    } else if dirty_count > 0 || behind > 0 || untracked_count > 5 {
        "yellow"
    } else {
        "green"
    }
}

/// Build a human-readable summary string from status fields.
fn build_summary(
    dirty_count: u32,
    staged_count: u32,
    untracked_count: u32,
    ahead: u32,
    behind: u32,
    conflict_count: u32,
) -> String {
    let mut parts = Vec::new();

    if conflict_count > 0 {
        parts.push(format!("{conflict_count} conflicts"));
    }
    if behind > 0 {
        parts.push(format!("{behind} behind upstream"));
    }
    if ahead > 0 {
        parts.push(format!("{ahead} ahead"));
    }
    if dirty_count > 0 {
        parts.push(format!("{dirty_count} dirty files"));
    }
    if staged_count > 0 {
        parts.push(format!("{staged_count} staged"));
    }
    if untracked_count > 0 {
        parts.push(format!("{untracked_count} untracked"));
    }

    if parts.is_empty() {
        "clean".to_string()
    } else {
        parts.join(", ")
    }
}

// ---------------------------------------------------------------------------
// Git log (commit history with pagination)
// ---------------------------------------------------------------------------

/// A single commit entry from git log.
#[derive(Debug, Serialize, Clone)]
pub struct CommitEntry {
    pub hash: String,
    pub short_hash: String,
    pub author_name: String,
    pub author_email: String,
    pub date: String,
    pub subject: String,
    pub body: String,
}

/// Query parameters for `GET /api/git/log`.
#[derive(Debug, Deserialize)]
pub struct GitLogQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

/// Response shape for `GET /api/git/log`.
#[derive(Debug, Serialize)]
pub struct GitLogResponse {
    pub commits: Vec<CommitEntry>,
    pub page: u32,
    pub per_page: u32,
    pub total_commits: u32,
}

/// Record separator between fields within a commit.
const FIELD_SEP: char = '\x1e';
/// Group separator between commits.
const COMMIT_SEP: char = '\x1d';

/// GET /api/git/log — return paginated commit history.
pub async fn get_git_log(
    State(app): State<AppState>,
    Query(params): Query<GitLogQuery>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(25).clamp(1, 100);

    let result = tokio::task::spawn_blocking(move || collect_git_log(&root, page, per_page))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

    match result {
        Ok(response) => Ok(Json(
            serde_json::to_value(response)
                .map_err(|e| AppError(anyhow::anyhow!("serialize error: {e}")))?,
        )),
        Err(e) if is_not_git_repo(&e) => {
            Ok(Json(serde_json::json!({ "error": "not_a_git_repo" })))
        }
        Err(e) => Err(AppError(e)),
    }
}

/// Run git commands to collect paginated commit log.
fn collect_git_log(root: &Path, page: u32, per_page: u32) -> Result<GitLogResponse, anyhow::Error> {
    // Verify this is a git repo
    let check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !check.status.success() {
        return Err(anyhow::anyhow!("not_a_git_repo"));
    }

    // Check if there are any commits
    let count_output = Command::new("git")
        .args(["rev-list", "--count", "HEAD"])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git rev-list: {e}"))?;

    if !count_output.status.success() {
        // No commits yet (empty repo)
        return Ok(GitLogResponse {
            commits: Vec::new(),
            page,
            per_page,
            total_commits: 0,
        });
    }

    let total_commits: u32 = String::from_utf8_lossy(&count_output.stdout)
        .trim()
        .parse()
        .unwrap_or(0);

    if total_commits == 0 {
        return Ok(GitLogResponse {
            commits: Vec::new(),
            page,
            per_page,
            total_commits: 0,
        });
    }

    let skip = (page - 1) * per_page;
    let format = format!("%H{0}%h{0}%an{0}%ae{0}%aI{0}%s{0}%b{1}", FIELD_SEP, COMMIT_SEP);

    let log_output = Command::new("git")
        .args([
            "log",
            &format!("--format={format}"),
            &format!("--skip={skip}"),
            &format!("-n{per_page}"),
        ])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git log: {e}"))?;

    if !log_output.status.success() {
        let stderr = String::from_utf8_lossy(&log_output.stderr);
        return Err(anyhow::anyhow!("git log failed: {stderr}"));
    }

    let stdout = String::from_utf8_lossy(&log_output.stdout);
    let commits = parse_git_log_output(&stdout);

    Ok(GitLogResponse {
        commits,
        page,
        per_page,
        total_commits,
    })
}

/// Parse separator-delimited git log output into commit entries.
pub(crate) fn parse_git_log_output(output: &str) -> Vec<CommitEntry> {
    let mut commits = Vec::new();

    for record in output.split(COMMIT_SEP) {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }

        let fields: Vec<&str> = record.splitn(7, FIELD_SEP).collect();
        if fields.len() < 6 {
            continue;
        }

        commits.push(CommitEntry {
            hash: fields[0].to_string(),
            short_hash: fields[1].to_string(),
            author_name: fields[2].to_string(),
            author_email: fields[3].to_string(),
            date: fields[4].to_string(),
            subject: fields[5].to_string(),
            body: fields.get(6).unwrap_or(&"").trim().to_string(),
        });
    }

    commits
}

// ---------------------------------------------------------------------------
// GET /api/git/show/{sha} — commit detail
// ---------------------------------------------------------------------------

/// GET /api/git/show/{sha} — return details for a single commit.
pub async fn get_commit_detail(
    State(app): State<AppState>,
    axum::extract::Path(sha): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, AppError> {
    let root = app.root.clone();
    let result = tokio::task::spawn_blocking(move || {
        let check = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(&root)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

        if !check.status.success() {
            return Err(anyhow::anyhow!("not_a_git_repo"));
        }

        let format_str = format!("%H{0}%h{0}%an{0}%ae{0}%aI{0}%s{0}%b{1}",
            '\x1e', '\x1d');
        let output = Command::new("git")
            .args([
                "show",
                &format!("--format={}", format_str),
                "--no-patch",
                &sha,
            ])
            .current_dir(&root)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run git show: {e}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("git show failed: {stderr}"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let commits = parse_git_log_output(&stdout);
        match commits.into_iter().next() {
            Some(c) => Ok(serde_json::to_value(c)
                .map_err(|e| anyhow::anyhow!("serialize error: {e}"))?),
            None => Err(anyhow::anyhow!("commit not found: {sha}")),
        }
    })
    .await
    .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

    match result {
        Ok(val) => Ok(Json(val)),
        Err(e) if is_not_git_repo(&e) => {
            Ok(Json(serde_json::json!({ "error": "not_a_git_repo" })))
        }
        Err(e) => Err(AppError(e)),
    }
}

// ---------------------------------------------------------------------------
// POST /api/git/commit — agent-driven commit of current changes
// ---------------------------------------------------------------------------

/// Start an agent run that reads the current diff, generates a conventional-commit
/// message, and executes `sdlc commit --message "<msg>"`.
pub async fn start_git_commit(
    State(app): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let prompt = r#"You are a git commit assistant. Your job is to commit the current working tree changes with a well-crafted commit message.

## Steps

1. Check for changes:
   ```bash
   git status --short
   ```
   If the output is empty, report "Nothing to commit — working tree clean." and stop.

2. Read the diff to understand what changed:
   ```bash
   git diff HEAD --stat
   git diff HEAD
   ```

3. Generate a single-line commit message (120 chars max) using conventional-commit prefixes:
   - `feat:` — new feature or capability
   - `fix:` — bug fix
   - `refactor:` — code restructuring without behavior change
   - `docs:` — documentation only
   - `chore:` — maintenance, dependencies, config
   - `test:` — test additions or changes

   Focus on **what changed and why**, not which files were touched.

4. Execute the commit:
   ```bash
   sdlc commit --message "<your generated message>"
   ```

5. Report the result: commit SHA, whether a merge was needed, ahead/behind status."#
        .to_string();

    let opts = sdlc_query_options(app.root.clone(), 10, None);

    spawn_agent_run(
        "git-commit".to_string(),
        prompt,
        opts,
        &app,
        "git_commit",
        "Git commit",
        Some(SseMessage::GitCommitCompleted),
    )
    .await
}

// ---------------------------------------------------------------------------
// Git diff (single file diff)
// ---------------------------------------------------------------------------

/// Query parameters for `GET /api/git/diff`.
#[derive(Debug, Deserialize)]
pub struct DiffParams {
    pub path: Option<String>,
    #[serde(default)]
    pub staged: Option<bool>,
}

/// Result of diffing a single file.
#[derive(Debug, Serialize)]
pub struct DiffResult {
    pub path: String,
    pub diff: String,
    pub status: String,
    pub is_new: bool,
    pub is_deleted: bool,
    pub is_binary: bool,
}

/// Validate that a relative path does not contain traversal components.
fn validate_diff_path(path: &str) -> Result<(), &'static str> {
    for component in std::path::Path::new(path).components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err("invalid_path");
        }
    }
    Ok(())
}

/// Determine file status from `git status --porcelain=v2 -- <path>` output.
fn parse_file_status(porcelain_output: &str) -> (&'static str, bool, bool) {
    let line = porcelain_output.lines().next().unwrap_or("");

    if line.starts_with("? ") {
        return ("untracked", true, false);
    }

    if line.starts_with("2 ") {
        return ("renamed", false, false);
    }

    if line.starts_with("1 ") {
        if let Some(xy) = extract_xy(line) {
            return match xy {
                (_, 'D') | ('D', _) => ("deleted", false, true),
                ('A', _) => ("added", true, false),
                _ => ("modified", false, false),
            };
        }
    }

    // No output or unrecognised → unchanged
    ("unchanged", false, false)
}

/// Check if diff output indicates a binary file.
fn is_binary_diff(diff_output: &str) -> bool {
    diff_output
        .lines()
        .any(|line| line.starts_with("Binary files"))
}

/// Run git commands to produce a diff for a single file.
fn collect_git_diff(root: &Path, file_path: &str, staged: bool) -> Result<DiffResult, anyhow::Error> {
    // Verify this is a git repo
    let check = Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git: {e}"))?;

    if !check.status.success() {
        return Err(anyhow::anyhow!("not_a_git_repo"));
    }

    // Determine file status via porcelain v2
    let status_output = Command::new("git")
        .args(["status", "--porcelain=v2", "--", file_path])
        .current_dir(root)
        .output()
        .map_err(|e| anyhow::anyhow!("failed to run git status: {e}"))?;

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    let (status, is_new, is_deleted) = parse_file_status(&status_stdout);

    // Get diff content
    let diff_output = if status == "untracked" {
        let out = Command::new("git")
            .args(["diff", "--no-index", "/dev/null", file_path])
            .current_dir(root)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run git diff: {e}"))?;
        // git diff --no-index exits with 1 when there are differences (expected)
        String::from_utf8_lossy(&out.stdout).to_string()
    } else if status == "unchanged" {
        String::new()
    } else {
        let mut args = vec!["diff"];
        if staged {
            args.push("--cached");
        }
        args.push("--");
        args.push(file_path);

        let out = Command::new("git")
            .args(&args)
            .current_dir(root)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to run git diff: {e}"))?;

        String::from_utf8_lossy(&out.stdout).to_string()
    };

    let is_binary = is_binary_diff(&diff_output);

    Ok(DiffResult {
        path: file_path.to_string(),
        diff: diff_output,
        status: status.to_string(),
        is_new,
        is_deleted,
        is_binary,
    })
}

/// GET /api/git/diff — return unified diff for a single file.
pub async fn get_git_diff(
    State(app): State<AppState>,
    Query(params): Query<DiffParams>,
) -> Result<Json<serde_json::Value>, AppError> {
    let file_path = match params.path {
        Some(p) if !p.is_empty() => p,
        _ => {
            return Ok(Json(
                serde_json::json!({ "error": "missing_path_parameter" }),
            ));
        }
    };

    // Validate path — reject traversal attempts
    if validate_diff_path(&file_path).is_err() {
        return Ok(Json(serde_json::json!({ "error": "invalid_path" })));
    }

    let staged = params.staged.unwrap_or(false);
    let root = app.root.clone();
    let path_clone = file_path.clone();

    // Check if file exists (allow deleted files that git still tracks)
    let full_path = root.join(&file_path);
    if !full_path.exists() {
        let git_check = tokio::task::spawn_blocking({
            let root = root.clone();
            let fp = file_path.clone();
            move || {
                let out = Command::new("git")
                    .args(["ls-files", "--", &fp])
                    .current_dir(&root)
                    .output();
                match out {
                    Ok(o) => !String::from_utf8_lossy(&o.stdout).trim().is_empty(),
                    Err(_) => false,
                }
            }
        })
        .await
        .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

        if !git_check {
            return Ok(Json(
                serde_json::json!({ "error": "file_not_found", "path": file_path }),
            ));
        }
    }

    let result = tokio::task::spawn_blocking(move || collect_git_diff(&root, &path_clone, staged))
        .await
        .map_err(|e| AppError(anyhow::anyhow!("join error: {e}")))?;

    match result {
        Ok(diff) => Ok(Json(
            serde_json::to_value(diff)
                .map_err(|e| AppError(anyhow::anyhow!("serialize error: {e}")))?,
        )),
        Err(e) if is_not_git_repo(&e) => {
            Ok(Json(serde_json::json!({ "error": "not_a_git_repo" })))
        }
        Err(e) => Err(AppError(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_clean_repo() {
        let output = "# branch.head main\n# branch.ab +0 -0\n";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.branch, "main");
        assert_eq!(status.dirty_count, 0);
        assert_eq!(status.staged_count, 0);
        assert_eq!(status.untracked_count, 0);
        assert_eq!(status.ahead, 0);
        assert_eq!(status.behind, 0);
        assert!(!status.has_conflicts);
        assert_eq!(status.severity, "green");
        assert_eq!(status.summary, "clean");
    }

    #[test]
    fn parse_dirty_files() {
        let output = "\
# branch.head feature-x
# branch.ab +0 -0
1 .M N... 100644 100644 100644 abc123 def456 src/main.rs
1 M. N... 100644 100644 100644 abc123 def456 src/lib.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.branch, "feature-x");
        assert_eq!(status.dirty_count, 1); // .M has Y=M (dirty)
        assert_eq!(status.staged_count, 1); // M. has X=M (staged)
        assert_eq!(status.severity, "yellow");
    }

    #[test]
    fn parse_untracked() {
        let output = "\
# branch.head main
# branch.ab +0 -0
? new-file.txt
? another.txt
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.untracked_count, 2);
        assert_eq!(status.severity, "green"); // only 2, threshold is >5
    }

    #[test]
    fn parse_untracked_above_threshold() {
        let output = "\
# branch.head main
# branch.ab +0 -0
? a.txt
? b.txt
? c.txt
? d.txt
? e.txt
? f.txt
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.untracked_count, 6);
        assert_eq!(status.severity, "yellow");
    }

    #[test]
    fn parse_conflicts() {
        let output = "\
# branch.head main
# branch.ab +0 -0
u UU N... 100644 100644 100644 100644 abc123 def456 ghi789 conflict.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert!(status.has_conflicts);
        assert_eq!(status.conflict_count, 1);
        assert_eq!(status.severity, "red");
    }

    #[test]
    fn parse_ahead_behind() {
        let output = "# branch.head dev\n# branch.ab +3 -2\n";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.ahead, 3);
        assert_eq!(status.behind, 2);
        assert_eq!(status.severity, "yellow"); // behind > 0
        assert!(status.summary.contains("2 behind upstream"));
        assert!(status.summary.contains("3 ahead"));
    }

    #[test]
    fn parse_far_behind_is_red() {
        let output = "# branch.head main\n# branch.ab +0 -11\n";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.behind, 11);
        assert_eq!(status.severity, "red");
    }

    #[test]
    fn parse_detached_head() {
        let output = "# branch.head (detached)\n";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.branch, "(detached)");
    }

    #[test]
    fn severity_green() {
        assert_eq!(compute_severity(0, 0, 0, false), "green");
    }

    #[test]
    fn severity_yellow_dirty() {
        assert_eq!(compute_severity(1, 0, 0, false), "yellow");
    }

    #[test]
    fn severity_yellow_behind() {
        assert_eq!(compute_severity(0, 1, 0, false), "yellow");
    }

    #[test]
    fn severity_red_conflicts() {
        assert_eq!(compute_severity(0, 0, 0, true), "red");
    }

    #[test]
    fn severity_red_far_behind() {
        assert_eq!(compute_severity(0, 11, 0, false), "red");
    }

    #[test]
    fn summary_clean() {
        assert_eq!(build_summary(0, 0, 0, 0, 0, 0), "clean");
    }

    #[test]
    fn summary_multiple_fields() {
        let s = build_summary(2, 1, 3, 1, 4, 0);
        assert!(s.contains("4 behind upstream"));
        assert!(s.contains("1 ahead"));
        assert!(s.contains("2 dirty files"));
        assert!(s.contains("1 staged"));
        assert!(s.contains("3 untracked"));
    }

    #[test]
    fn summary_conflicts() {
        let s = build_summary(0, 0, 0, 0, 0, 2);
        assert_eq!(s, "2 conflicts");
    }

    #[test]
    fn extract_xy_ordinary() {
        assert_eq!(extract_xy("1 .M N... 100644 100644 100644 abc def file.rs"), Some(('.', 'M')));
    }

    #[test]
    fn extract_xy_staged() {
        assert_eq!(extract_xy("1 M. N... 100644 100644 100644 abc def file.rs"), Some(('M', '.')));
    }

    #[test]
    fn extract_xy_renamed() {
        assert_eq!(extract_xy("2 R. N... 100644 100644 abc def R100 old\tnew"), Some(('R', '.')));
    }

    // ── Diff tests ─────────────────────────────────────────────────────

    #[test]
    fn validate_path_rejects_dotdot() {
        assert!(validate_diff_path("../../../etc/passwd").is_err());
        assert!(validate_diff_path("foo/../bar").is_err());
        assert!(validate_diff_path("..").is_err());
    }

    #[test]
    fn validate_path_accepts_normal() {
        assert!(validate_diff_path("src/main.rs").is_ok());
        assert!(validate_diff_path("a/b/c/d.rs").is_ok());
        assert!(validate_diff_path("file.txt").is_ok());
        assert!(validate_diff_path("frontend/src/App.tsx").is_ok());
    }

    #[test]
    fn parse_file_status_modified() {
        let output = "1 .M N... 100644 100644 100644 abc123 def456 src/main.rs\n";
        let (status, is_new, is_deleted) = parse_file_status(output);
        assert_eq!(status, "modified");
        assert!(!is_new);
        assert!(!is_deleted);
    }

    #[test]
    fn parse_file_status_added() {
        let output = "1 A. N... 000000 100644 100644 abc123 def456 new-file.rs\n";
        let (status, is_new, is_deleted) = parse_file_status(output);
        assert_eq!(status, "added");
        assert!(is_new);
        assert!(!is_deleted);
    }

    #[test]
    fn parse_file_status_deleted() {
        let output = "1 .D N... 100644 100644 000000 abc123 def456 removed.rs\n";
        let (status, is_new, is_deleted) = parse_file_status(output);
        assert_eq!(status, "deleted");
        assert!(!is_new);
        assert!(is_deleted);
    }

    #[test]
    fn parse_file_status_renamed() {
        let output = "2 R. N... 100644 100644 abc123 def456 R100 old.rs\tnew.rs\n";
        let (status, is_new, is_deleted) = parse_file_status(output);
        assert_eq!(status, "renamed");
        assert!(!is_new);
        assert!(!is_deleted);
    }

    #[test]
    fn parse_file_status_untracked() {
        let output = "? brand-new.txt\n";
        let (status, is_new, is_deleted) = parse_file_status(output);
        assert_eq!(status, "untracked");
        assert!(is_new);
        assert!(!is_deleted);
    }

    #[test]
    fn parse_file_status_empty() {
        let (status, is_new, is_deleted) = parse_file_status("");
        assert_eq!(status, "unchanged");
        assert!(!is_new);
        assert!(!is_deleted);
    }

    #[test]
    fn detect_binary_true() {
        let diff = "diff --git a/image.png b/image.png\nBinary files a/image.png and b/image.png differ\n";
        assert!(is_binary_diff(diff));
    }

    #[test]
    fn detect_binary_false() {
        let diff = "diff --git a/src/main.rs b/src/main.rs\n--- a/src/main.rs\n+++ b/src/main.rs\n@@ -1,3 +1,4 @@\n fn main() {\n+    println!(\"hello\");\n }\n";
        assert!(!is_binary_diff(diff));
    }

    #[test]
    fn diff_result_serializes_correctly() {
        let result = DiffResult {
            path: "src/lib.rs".to_string(),
            diff: "+new line".to_string(),
            status: "modified".to_string(),
            is_new: false,
            is_deleted: false,
            is_binary: false,
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["path"], "src/lib.rs");
        assert_eq!(json["status"], "modified");
        assert_eq!(json["is_new"], false);
        assert_eq!(json["is_deleted"], false);
        assert_eq!(json["is_binary"], false);
    }

    // ── Git log parser tests ─────────────────────────────────────────

    #[test]
    fn parse_log_multiple_commits() {
        let output = format!(
            "abc123def456789012345678901234567890abcd{0}abc123d{0}Alice{0}alice@example.com{0}2026-03-07T10:00:00+00:00{0}feat: first commit{0}{1}\
             def456789012345678901234567890abcdef1234{0}def4567{0}Bob{0}bob@example.com{0}2026-03-06T09:00:00+00:00{0}fix: second commit{0}Some body text{1}\
             789012345678901234567890abcdef1234567890{0}7890123{0}Carol{0}carol@example.com{0}2026-03-05T08:00:00+00:00{0}docs: third commit{0}Line one\nLine two{1}",
            FIELD_SEP, COMMIT_SEP
        );

        let commits = parse_git_log_output(&output);
        assert_eq!(commits.len(), 3);

        assert_eq!(commits[0].hash, "abc123def456789012345678901234567890abcd");
        assert_eq!(commits[0].short_hash, "abc123d");
        assert_eq!(commits[0].author_name, "Alice");
        assert_eq!(commits[0].author_email, "alice@example.com");
        assert_eq!(commits[0].date, "2026-03-07T10:00:00+00:00");
        assert_eq!(commits[0].subject, "feat: first commit");
        assert_eq!(commits[0].body, "");

        assert_eq!(commits[1].subject, "fix: second commit");
        assert_eq!(commits[1].body, "Some body text");

        assert_eq!(commits[2].subject, "docs: third commit");
        assert_eq!(commits[2].body, "Line one\nLine two");
    }

    #[test]
    fn parse_log_empty_output() {
        let commits = parse_git_log_output("");
        assert!(commits.is_empty());
    }

    #[test]
    fn parse_log_single_commit() {
        let output = format!(
            "abcdef1234567890abcdef1234567890abcdef12{0}abcdef1{0}Dev{0}dev@test.io{0}2026-01-01T00:00:00Z{0}initial commit{0}{1}",
            FIELD_SEP, COMMIT_SEP
        );
        let commits = parse_git_log_output(&output);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "initial commit");
        assert_eq!(commits[0].body, "");
    }

    #[test]
    fn parse_log_multiline_body() {
        let output = format!(
            "abcdef1234567890abcdef1234567890abcdef12{0}abcdef1{0}Dev{0}dev@test.io{0}2026-01-01T00:00:00Z{0}feat: big change{0}Paragraph one.\n\nParagraph two.\nWith continuation.{1}",
            FIELD_SEP, COMMIT_SEP
        );
        let commits = parse_git_log_output(&output);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].subject, "feat: big change");
        assert!(commits[0].body.contains("Paragraph one."));
        assert!(commits[0].body.contains("Paragraph two."));
        assert!(commits[0].body.contains("With continuation."));
    }

    #[test]
    fn parse_log_special_characters() {
        let output = format!(
            "abcdef1234567890abcdef1234567890abcdef12{0}abcdef1{0}O'Brien{0}ob@test.io{0}2026-01-01T00:00:00Z{0}fix: handle <angle> & \"quotes\"{0}{1}",
            FIELD_SEP, COMMIT_SEP
        );
        let commits = parse_git_log_output(&output);
        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].author_name, "O'Brien");
        assert_eq!(commits[0].subject, "fix: handle <angle> & \"quotes\"");
    }

    #[test]
    fn parse_log_whitespace_only_records_skipped() {
        let output = format!("  {0}  \n  ", COMMIT_SEP);
        let commits = parse_git_log_output(&output);
        assert!(commits.is_empty());
    }

    // ── Directory counts tests ───────────────────────────────────────

    #[test]
    fn dir_counts_clean_repo() {
        let output = "# branch.head main\n# branch.ab +0 -0\n";
        let status = parse_porcelain_v2(output).unwrap();
        assert!(status.directory_counts.is_empty());
    }

    #[test]
    fn dir_counts_single_directory() {
        let output = "\
# branch.head main
# branch.ab +0 -0
1 .M N... 100644 100644 100644 abc123 def456 src/main.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 1);
        assert_eq!(status.directory_counts[0].directory, "src");
        assert_eq!(status.directory_counts[0].count, 1);
    }

    #[test]
    fn dir_counts_multiple_directories_sorted() {
        let output = "\
# branch.head main
# branch.ab +0 -0
1 .M N... 100644 100644 100644 abc123 def456 frontend/src/App.tsx
1 .M N... 100644 100644 100644 abc123 def456 frontend/src/index.ts
1 .M N... 100644 100644 100644 abc123 def456 frontend/src/utils.ts
1 .M N... 100644 100644 100644 abc123 def456 crates/server/lib.rs
? new-root-file.txt
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 3);
        // frontend/src has 3 files — should be first
        assert_eq!(status.directory_counts[0].directory, "frontend/src");
        assert_eq!(status.directory_counts[0].count, 3);
        // Remaining entries sorted alphabetically (both have count 1)
        assert_eq!(status.directory_counts[1].directory, ".");
        assert_eq!(status.directory_counts[1].count, 1);
        assert_eq!(status.directory_counts[2].directory, "crates/server");
        assert_eq!(status.directory_counts[2].count, 1);
    }

    #[test]
    fn dir_counts_root_level_files() {
        let output = "\
# branch.head main
# branch.ab +0 -0
? README.md
? Cargo.toml
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 1);
        assert_eq!(status.directory_counts[0].directory, ".");
        assert_eq!(status.directory_counts[0].count, 2);
    }

    #[test]
    fn dir_counts_renamed_uses_destination() {
        let output = "\
# branch.head main
# branch.ab +0 -0
2 R. N... 100644 100644 abc123 def456 R100 old/path.rs\tnew/dir/path.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 1);
        assert_eq!(status.directory_counts[0].directory, "new/dir");
        assert_eq!(status.directory_counts[0].count, 1);
    }

    #[test]
    fn dir_counts_mixed_types_same_dir() {
        let output = "\
# branch.head main
# branch.ab +0 -0
1 .M N... 100644 100644 100644 abc123 def456 src/lib.rs
1 M. N... 100644 100644 100644 abc123 def456 src/main.rs
? src/new.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 1);
        assert_eq!(status.directory_counts[0].directory, "src");
        assert_eq!(status.directory_counts[0].count, 3);
    }

    #[test]
    fn dir_counts_conflict_entry() {
        let output = "\
# branch.head main
# branch.ab +0 -0
u UU N... 100644 100644 100644 100644 abc123 def456 ghi789 src/conflict.rs
";
        let status = parse_porcelain_v2(output).unwrap();
        assert_eq!(status.directory_counts.len(), 1);
        assert_eq!(status.directory_counts[0].directory, "src");
        assert_eq!(status.directory_counts[0].count, 1);
    }

    #[test]
    fn parent_dir_nested() {
        assert_eq!(parent_dir("a/b/c.rs"), "a/b");
    }

    #[test]
    fn parent_dir_root_level() {
        assert_eq!(parent_dir("file.rs"), ".");
    }

    #[test]
    fn parent_dir_single_level() {
        assert_eq!(parent_dir("src/main.rs"), "src");
    }
}
