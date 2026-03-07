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

    // 4. Commit
    let msg = message.unwrap_or("wip");
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
