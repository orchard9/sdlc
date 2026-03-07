use crate::output::print_json;
use anyhow::Context;
use std::path::Path;
use std::process::Command;

/// Run a git command in the project root, returning stdout as a trimmed string.
fn git(root: &Path, args: &[&str]) -> anyhow::Result<String> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        anyhow::bail!("git {} failed: {}", args.join(" "), stderr.trim());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

/// Run a git command, returning (success, stdout).
fn git_try(root: &Path, args: &[&str]) -> anyhow::Result<(bool, String)> {
    let out = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .with_context(|| format!("failed to run git {}", args.join(" ")))?;
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    Ok((out.status.success(), stdout))
}

/// Generate a brief commit message (<=120 chars) from the staged diff.
fn generate_commit_message(root: &Path) -> anyhow::Result<String> {
    // Get the diffstat for staged changes
    let stat = git(root, &["diff", "--cached", "--stat"])?;
    let name_only = git(root, &["diff", "--cached", "--name-only"])?;

    let files: Vec<&str> = name_only.lines().collect();
    let file_count = files.len();

    // Classify the change type from file paths
    let has_new = !git(root, &["diff", "--cached", "--diff-filter=A", "--name-only"])?
        .is_empty();
    let has_deleted = !git(root, &["diff", "--cached", "--diff-filter=D", "--name-only"])?
        .is_empty();
    let has_modified = !git(root, &["diff", "--cached", "--diff-filter=M", "--name-only"])?
        .is_empty();

    // Determine prefix verb
    let verb = if has_new && !has_modified && !has_deleted {
        "add"
    } else if has_deleted && !has_new && !has_modified {
        "remove"
    } else {
        "update"
    };

    // Find common directory components to summarize scope
    let components: Vec<Vec<&str>> = files
        .iter()
        .map(|f| f.split('/').collect::<Vec<_>>())
        .collect();

    let scope = if file_count == 1 {
        // Single file — use the filename
        files[0]
            .rsplit('/')
            .next()
            .unwrap_or(files[0])
            .to_string()
    } else {
        // Find the deepest common prefix directory
        let common = common_prefix(&components);
        if common.is_empty() {
            // No common prefix — summarize by extension or top-level dirs
            let top_dirs: Vec<&str> = components
                .iter()
                .filter_map(|c| c.first().copied())
                .collect::<std::collections::BTreeSet<_>>()
                .into_iter()
                .take(3)
                .collect();
            if top_dirs.len() == 1 {
                top_dirs[0].to_string()
            } else {
                top_dirs.join(", ")
            }
        } else {
            common
        }
    };

    // Extract insertions/deletions from the stat summary (last line)
    let stat_summary = stat
        .lines()
        .last()
        .unwrap_or("")
        .trim();

    let mut parts = Vec::new();
    if let Some(ins) = extract_number(stat_summary, "insertion") {
        parts.push(format!("+{ins}"));
    }
    if let Some(del) = extract_number(stat_summary, "deletion") {
        parts.push(format!("-{del}"));
    }
    let delta = if parts.is_empty() {
        String::new()
    } else {
        format!(" ({})", parts.join("/"))
    };

    let file_note = if file_count > 1 {
        format!(", {file_count} files")
    } else {
        String::new()
    };

    let msg = format!("{verb}: {scope}{file_note}{delta}");

    // Truncate to 120 chars
    if msg.len() > 120 {
        Ok(format!("{}...", &msg[..117]))
    } else {
        Ok(msg)
    }
}

/// Find the deepest common directory prefix from path components.
fn common_prefix(components: &[Vec<&str>]) -> String {
    if components.is_empty() {
        return String::new();
    }
    let first = &components[0];
    let mut depth = 0;
    for i in 0..first.len().saturating_sub(1) {
        // Don't include the filename — only directories
        if components.iter().all(|c| c.len() > i + 1 && c[i] == first[i]) {
            depth = i + 1;
        } else {
            break;
        }
    }
    if depth == 0 {
        String::new()
    } else {
        first[..depth].join("/")
    }
}

/// Extract a number preceding a keyword like "insertion" or "deletion" from git stat summary.
fn extract_number(line: &str, keyword: &str) -> Option<usize> {
    let idx = line.find(keyword)?;
    let before = line[..idx].trim();
    before.rsplit(' ').next()?.parse().ok()
}

pub fn run(root: &Path, message: Option<&str>, json: bool) -> anyhow::Result<()> {
    // 1. Verify on main
    let branch = git(root, &["branch", "--show-current"])?;
    if branch != "main" {
        anyhow::bail!("not on main (current branch: {branch}). Switch to main first.");
    }

    // 2. Check for changes
    let status = git(root, &["status", "--short"])?;
    if status.is_empty() {
        if json {
            print_json(&serde_json::json!({
                "committed": false,
                "merged": false,
                "reason": "nothing to commit",
            }))?;
        } else {
            println!("Nothing to commit — working tree clean.");
        }
        return Ok(());
    }

    // 3. Stage all changes
    git(root, &["add", "-A"])?;

    // 4. Generate commit message if none provided
    let generated;
    let msg = match message {
        Some(m) => m,
        None => {
            generated = generate_commit_message(root)?;
            generated.as_str()
        }
    };
    git(root, &["commit", "-m", msg])?;

    let commit_sha = git(root, &["rev-parse", "--short", "HEAD"])?;
    if !json {
        println!("Committed {commit_sha}: {msg}");
    }

    // 5. Verify clean
    let post_status = git(root, &["status", "--short"])?;
    if !post_status.is_empty() {
        anyhow::bail!(
            "working tree not clean after commit — untracked files remain:\n{post_status}"
        );
    }

    // 6. Fetch origin
    let (fetch_ok, _) = git_try(root, &["fetch", "origin"])?;
    if !fetch_ok {
        if json {
            print_json(&serde_json::json!({
                "committed": true,
                "commit": commit_sha,
                "message": msg,
                "merged": false,
                "reason": "fetch failed — offline or no remote",
            }))?;
        } else {
            println!("Fetch failed (offline or no remote). Commit is local-only.");
        }
        return Ok(());
    }

    // 7. Check if origin/main exists
    let (has_origin, _) = git_try(root, &["rev-parse", "origin/main"])?;
    if !has_origin {
        if json {
            print_json(&serde_json::json!({
                "committed": true,
                "commit": commit_sha,
                "message": msg,
                "merged": false,
                "reason": "no origin/main",
            }))?;
        } else {
            println!("No origin/main found. Commit is local-only.");
        }
        return Ok(());
    }

    // 8. Check divergence: ahead/behind
    let counts = git(root, &["rev-list", "--left-right", "--count", "main...origin/main"])?;
    let parts: Vec<&str> = counts.split_whitespace().collect();
    let ahead: u64 = parts.first().unwrap_or(&"0").parse().unwrap_or(0);
    let behind: u64 = parts.get(1).unwrap_or(&"0").parse().unwrap_or(0);

    if behind == 0 {
        // No divergence — safe to push
        if json {
            print_json(&serde_json::json!({
                "committed": true,
                "commit": commit_sha,
                "message": msg,
                "merged": false,
                "ahead": ahead,
                "behind": 0,
                "status": "ready to push",
            }))?;
        } else {
            println!("Local main is {ahead} ahead, 0 behind origin/main. Ready to push.");
        }
        return Ok(());
    }

    // 9. Diverged — reconcile
    if !json {
        println!(
            "Diverged: {ahead} ahead, {behind} behind origin/main. Reconciling..."
        );
    }

    // Rename local main → dev/xist
    git(root, &["branch", "-m", "main", "dev/xist"])?;
    let _ = git_try(root, &["branch", "--unset-upstream", "dev/xist"]);

    // Create new main from origin/main
    git(root, &["checkout", "-b", "main", "origin/main"])?;

    // Merge dev/xist into main
    let (merge_ok, merge_out) = git_try(root, &["merge", "dev/xist", "--no-edit"])?;

    if !merge_ok {
        // Conflict — report and stop
        let conflict_files = git(root, &["diff", "--name-only", "--diff-filter=U"])
            .unwrap_or_default();
        if json {
            print_json(&serde_json::json!({
                "committed": true,
                "commit": commit_sha,
                "message": msg,
                "merged": false,
                "conflict": true,
                "conflict_files": conflict_files.lines().collect::<Vec<_>>(),
                "instructions": "Resolve conflicts, then: git merge --continue && git branch -d dev/xist",
            }))?;
        } else {
            eprintln!("Merge conflict. Conflicting files:");
            for f in conflict_files.lines() {
                eprintln!("  {f}");
            }
            eprintln!();
            eprintln!("Resolve conflicts, then run:");
            eprintln!("  git merge --continue");
            eprintln!("  git branch -d dev/xist");
        }
        return Ok(());
    }

    // Merge succeeded — clean up temp branch
    let _ = git_try(root, &["branch", "-d", "dev/xist"]);

    let final_sha = git(root, &["rev-parse", "--short", "HEAD"])?;
    let final_counts = git(root, &["rev-list", "--left-right", "--count", "main...origin/main"])?;
    let final_parts: Vec<&str> = final_counts.split_whitespace().collect();
    let final_ahead: u64 = final_parts.first().unwrap_or(&"0").parse().unwrap_or(0);

    if json {
        print_json(&serde_json::json!({
            "committed": true,
            "commit": commit_sha,
            "message": msg,
            "merged": true,
            "merge_head": final_sha,
            "ahead": final_ahead,
            "behind": 0,
            "status": "ready to push",
        }))?;
    } else {
        if !merge_out.is_empty() {
            println!("{merge_out}");
        }
        println!("Merged origin/main into local main at {final_sha}.");
        println!("{final_ahead} ahead, 0 behind. Ready to push.");
    }

    Ok(())
}
