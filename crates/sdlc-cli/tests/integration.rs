#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn sdlc(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("sdlc").unwrap();
    cmd.current_dir(dir.path()).env("SDLC_ROOT", dir.path());
    cmd
}

fn sdlc_with_home(project: &TempDir, home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("sdlc").unwrap();
    cmd.current_dir(project.path())
        .env("SDLC_ROOT", project.path())
        .env("HOME", home.path());
    cmd
}

fn init_project(dir: &TempDir) {
    // Create a throwaway home dir so init doesn't touch the real ~/.
    let home = TempDir::new().unwrap();
    sdlc_with_home(dir, &home).arg("init").assert().success();
}

// ---------------------------------------------------------------------------
// sdlc init
// ---------------------------------------------------------------------------

#[test]
fn init_creates_directory_tree() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Project-level SDLC structure
    assert!(dir.path().join(".sdlc").is_dir());
    assert!(dir.path().join(".sdlc/features").is_dir());
    assert!(dir.path().join(".sdlc/config.yaml").exists());
    assert!(dir.path().join(".sdlc/state.yaml").exists());
    assert!(dir.path().join(".ai").is_dir());
    assert!(dir.path().join(".ai/index.md").exists());
    assert!(dir.path().join(".ai/patterns").is_dir());
    assert!(dir.path().join(".ai/decisions").is_dir());
    assert!(dir.path().join("AGENTS.md").exists());

    // Commands are installed to user HOME, NOT project dir
    assert!(
        !dir.path().join(".claude/commands/sdlc-next.md").exists(),
        ".claude/commands/ should NOT be created in project dir"
    );
    assert!(
        !dir.path().join(".gemini/commands/sdlc-next.toml").exists(),
        ".gemini/commands/ should NOT be created in project dir"
    );
    assert!(
        !dir.path().join(".opencode/command/sdlc-next.md").exists(),
        ".opencode/command/ should NOT be created in project dir"
    );
    assert!(
        !dir.path()
            .join(".agents/skills/sdlc-next/SKILL.md")
            .exists(),
        ".agents/skills/ should NOT be created in project dir"
    );
}

#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    // Run twice — should succeed both times without error
    sdlc_with_home(&dir, &home).arg("init").assert().success();
    sdlc_with_home(&dir, &home).arg("init").assert().success();
}

#[test]
fn init_migrates_legacy_agent_scaffolds() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    // Create legacy project-level files from older sdlc versions
    std::fs::create_dir_all(dir.path().join(".claude/commands")).unwrap();
    std::fs::create_dir_all(dir.path().join(".gemini/commands")).unwrap();
    std::fs::create_dir_all(dir.path().join(".opencode/commands")).unwrap();
    std::fs::create_dir_all(dir.path().join(".codex/commands")).unwrap();

    for file in ["sdlc-next.md", "sdlc-status.md", "sdlc-approve.md"] {
        std::fs::write(dir.path().join(".claude/commands").join(file), "# legacy").unwrap();
        std::fs::write(dir.path().join(".gemini/commands").join(file), "# legacy").unwrap();
        std::fs::write(dir.path().join(".opencode/commands").join(file), "# legacy").unwrap();
        std::fs::write(dir.path().join(".codex/commands").join(file), "# legacy").unwrap();
    }

    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Legacy project-level files should be removed
    for file in ["sdlc-next.md", "sdlc-status.md", "sdlc-approve.md"] {
        assert!(
            !dir.path().join(".claude/commands").join(file).exists(),
            "legacy claude file should be removed: {file}"
        );
        assert!(
            !dir.path().join(".gemini/commands").join(file).exists(),
            "legacy gemini file should be removed: {file}"
        );
        assert!(
            !dir.path().join(".opencode/commands").join(file).exists(),
            "legacy opencode file should be removed: {file}"
        );
        assert!(
            !dir.path().join(".codex/commands").join(file).exists(),
            "legacy codex file should be removed: {file}"
        );
    }

    // New commands are in user home, not project dir
    assert!(home.path().join(".gemini/commands/sdlc-next.toml").exists());
    assert!(home.path().join(".opencode/command/sdlc-next.md").exists());
    assert!(home
        .path()
        .join(".agents/skills/sdlc-next/SKILL.md")
        .exists());
}

#[test]
fn init_appends_sdlc_section_to_existing_agents_md() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# Existing content\n").unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("# Existing content"));
    assert!(content.contains("## SDLC"));
}

#[test]
fn init_does_not_duplicate_sdlc_section() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    let count = content.matches("## SDLC").count();
    assert_eq!(count, 1, "SDLC section should appear exactly once");
}

#[test]
fn init_installs_user_level_commands() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Claude commands (including sdlc-specialize)
    assert!(home.path().join(".claude/commands/sdlc-next.md").exists());
    assert!(home.path().join(".claude/commands/sdlc-status.md").exists());
    assert!(home
        .path()
        .join(".claude/commands/sdlc-approve.md")
        .exists());
    assert!(home
        .path()
        .join(".claude/commands/sdlc-specialize.md")
        .exists());

    // Gemini commands
    assert!(home.path().join(".gemini/commands/sdlc-next.toml").exists());
    assert!(home
        .path()
        .join(".gemini/commands/sdlc-status.toml")
        .exists());
    assert!(home
        .path()
        .join(".gemini/commands/sdlc-approve.toml")
        .exists());

    // OpenCode commands
    assert!(home.path().join(".opencode/command/sdlc-next.md").exists());
    assert!(home
        .path()
        .join(".opencode/command/sdlc-status.md")
        .exists());
    assert!(home
        .path()
        .join(".opencode/command/sdlc-approve.md")
        .exists());

    // Agent skills
    assert!(home
        .path()
        .join(".agents/skills/sdlc-next/SKILL.md")
        .exists());
    assert!(home
        .path()
        .join(".agents/skills/sdlc-status/SKILL.md")
        .exists());
    assert!(home
        .path()
        .join(".agents/skills/sdlc-approve/SKILL.md")
        .exists());

    // None of the above in project dir
    assert!(!dir.path().join(".claude/commands").exists());
    assert!(!dir.path().join(".gemini/commands").exists());
    assert!(!dir.path().join(".opencode/command").exists());
    assert!(!dir.path().join(".agents/skills").exists());
}

#[test]
fn init_upserts_user_commands_on_second_run() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    sdlc_with_home(&dir, &home).arg("init").assert().success();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Files exist and contain expected content
    let next_content =
        std::fs::read_to_string(home.path().join(".claude/commands/sdlc-next.md")).unwrap();
    assert!(next_content.contains("sdlc-next"));

    let specialize_content =
        std::fs::read_to_string(home.path().join(".claude/commands/sdlc-specialize.md")).unwrap();
    assert!(specialize_content.contains("sdlc-specialize"));

    let skill_content =
        std::fs::read_to_string(home.path().join(".agents/skills/sdlc-next/SKILL.md")).unwrap();
    assert!(skill_content.contains("SDLC Next Skill"));
}

// ---------------------------------------------------------------------------
// sdlc init — sdlc_version stamping + marker support
// ---------------------------------------------------------------------------

#[test]
fn init_stamps_sdlc_version_in_config() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let config_yaml = std::fs::read_to_string(dir.path().join(".sdlc/config.yaml")).unwrap();
    assert!(
        config_yaml.contains("sdlc_version:"),
        "config.yaml should contain sdlc_version after init"
    );
}

#[test]
fn init_writes_agents_md_with_markers() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        content.contains("<!-- sdlc:start -->"),
        "AGENTS.md should contain sdlc:start marker"
    );
    assert!(
        content.contains("<!-- sdlc:end -->"),
        "AGENTS.md should contain sdlc:end marker"
    );
}

#[test]
fn init_updates_agents_md_via_markers_on_second_run() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    // Should have exactly one start marker — no duplication
    assert_eq!(
        content.matches("<!-- sdlc:start -->").count(),
        1,
        "sdlc:start marker must appear exactly once"
    );
    assert_eq!(
        content.matches("<!-- sdlc:end -->").count(),
        1,
        "sdlc:end marker must appear exactly once"
    );
}

#[test]
fn init_migrates_legacy_agents_md_to_markers() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    // Write an AGENTS.md that looks like the old format (no markers, but has ## SDLC)
    std::fs::write(
        dir.path().join("AGENTS.md"),
        "# AGENTS.md\n\nAgent instructions.\n\n## SDLC\n\nOld content here.\n\nProject: legacy\n",
    )
    .unwrap();

    sdlc_with_home(&dir, &home).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(
        content.contains("<!-- sdlc:start -->"),
        "legacy AGENTS.md should be migrated to marker format"
    );
    assert!(
        content.contains("<!-- sdlc:end -->"),
        "legacy AGENTS.md should be migrated to marker format"
    );
    // Old content should be replaced, not duplicated
    assert_eq!(
        content.matches("## SDLC").count(),
        1,
        "## SDLC should appear exactly once after migration"
    );
}

// ---------------------------------------------------------------------------
// sdlc update
// ---------------------------------------------------------------------------

#[test]
fn update_fails_on_uninitialized_project() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home)
        .arg("update")
        .assert()
        .failure()
        .stderr(predicate::str::contains("sdlc init"));
}

#[test]
fn update_refreshes_user_commands() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Overwrite a command file to simulate stale content
    let next_path = home.path().join(".claude/commands/sdlc-next.md");
    std::fs::write(&next_path, "stale content").unwrap();

    sdlc_with_home(&dir, &home).arg("update").assert().success();

    let refreshed = std::fs::read_to_string(&next_path).unwrap();
    assert!(
        !refreshed.contains("stale content"),
        "sdlc update should overwrite stale command files"
    );
    assert!(
        refreshed.contains("sdlc-next"),
        "refreshed command should contain expected content"
    );
}

#[test]
fn update_stamps_sdlc_version() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();
    sdlc_with_home(&dir, &home).arg("update").assert().success();

    let config_yaml = std::fs::read_to_string(dir.path().join(".sdlc/config.yaml")).unwrap();
    assert!(config_yaml.contains("sdlc_version:"));
}

#[test]
fn update_refreshes_agents_md_section() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();
    sdlc_with_home(&dir, &home).arg("init").assert().success();

    // Corrupt the content between markers to simulate stale content
    let agents_path = dir.path().join("AGENTS.md");
    let content = std::fs::read_to_string(&agents_path).unwrap();
    let corrupted = content.replace("Key Commands", "OLD KEY COMMANDS STALE");
    std::fs::write(&agents_path, corrupted).unwrap();

    sdlc_with_home(&dir, &home).arg("update").assert().success();

    let refreshed = std::fs::read_to_string(&agents_path).unwrap();
    assert!(
        !refreshed.contains("OLD KEY COMMANDS STALE"),
        "sdlc update should replace stale AGENTS.md content between markers"
    );
    assert!(
        refreshed.contains("Key Commands"),
        "refreshed AGENTS.md should contain current section content"
    );
}

// ---------------------------------------------------------------------------
// sdlc feature create / list / show
// ---------------------------------------------------------------------------

#[test]
fn feature_create_and_list() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth-login", "--title", "Auth Login"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["feature", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth-login"));
}

#[test]
fn feature_create_invalid_slug_fails() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "INVALID SLUG"])
        .assert()
        .failure();
}

#[test]
fn feature_create_duplicate_fails() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .failure();
}

#[test]
fn feature_show() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "my-feature", "--title", "My Feature"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["feature", "show", "my-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("My Feature"))
        .stdout(predicate::str::contains("draft"));
}

// ---------------------------------------------------------------------------
// sdlc next
// ---------------------------------------------------------------------------

#[test]
fn next_returns_create_spec_for_new_feature() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth-login"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["next", "--for", "auth-login", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_spec"));
}

#[test]
fn next_json_output_has_expected_fields() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "f1"])
        .assert()
        .success();

    let output = sdlc(&dir)
        .args(["next", "--for", "f1", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert!(json.get("feature").is_some());
    assert!(json.get("current_phase").is_some());
    assert!(json.get("action").is_some());
    assert!(json.get("message").is_some());
}

// ---------------------------------------------------------------------------
// sdlc artifact approve → phase transition
// ---------------------------------------------------------------------------

#[test]
fn approve_spec_enables_transition_to_specified() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth-login"])
        .assert()
        .success();

    // Next should be create_spec
    sdlc(&dir)
        .args(["next", "--for", "auth-login", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_spec"));

    // Mark spec as draft then approve
    sdlc(&dir)
        .args(["artifact", "draft", "auth-login", "spec"])
        .assert()
        .success();

    // Next should now be approve_spec
    sdlc(&dir)
        .args(["next", "--for", "auth-login", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("approve_spec"));

    // Approve spec
    sdlc(&dir)
        .args(["artifact", "approve", "auth-login", "spec"])
        .assert()
        .success();

    // Transition to specified
    sdlc(&dir)
        .args(["feature", "transition", "auth-login", "specified"])
        .assert()
        .success();

    // Phase should now be specified
    sdlc(&dir)
        .args(["feature", "show", "auth-login"])
        .assert()
        .success()
        .stdout(predicate::str::contains("specified"));
}

// ---------------------------------------------------------------------------
// sdlc task
// ---------------------------------------------------------------------------

#[test]
fn task_lifecycle() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "feat"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["task", "add", "feat", "Write", "tests"])
        .assert()
        .success()
        .stdout(predicate::str::contains("T1"));

    sdlc(&dir)
        .args(["task", "start", "feat", "T1"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["task", "complete", "feat", "T1"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["task", "list", "feat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));
}

// ---------------------------------------------------------------------------
// sdlc state
// ---------------------------------------------------------------------------

#[test]
fn state_shows_features() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "my-feat"])
        .assert()
        .success();

    sdlc(&dir)
        .arg("state")
        .assert()
        .success()
        .stdout(predicate::str::contains("my-feat"));
}

// ---------------------------------------------------------------------------
// sdlc merge
// ---------------------------------------------------------------------------

#[test]
fn merge_transitions_feature_to_released() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "merge-me"])
        .assert()
        .success();

    // Enter merge phase (manual transition remains intentionally available).
    sdlc(&dir)
        .args(["artifact", "approve", "merge-me", "qa_results"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "transition", "merge-me", "merge"])
        .assert()
        .success();

    sdlc(&dir).args(["merge", "merge-me"]).assert().success();

    sdlc(&dir)
        .args(["feature", "show", "merge-me"])
        .assert()
        .success()
        .stdout(predicate::str::contains("released"));

    sdlc(&dir)
        .args(["next", "--for", "merge-me", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"action\": \"done\""));
}

#[test]
fn merge_fails_when_feature_not_in_merge_phase() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "not-ready"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["merge", "not-ready"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("move it to 'merge' first"));
}

// ---------------------------------------------------------------------------
// sdlc query
// ---------------------------------------------------------------------------

#[test]
fn query_needs_approval_empty_initially() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["query", "needs-approval"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No features need approval"));
}

#[test]
fn query_needs_approval_includes_wait_for_approval_actions() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "approval-gap"])
        .assert()
        .success();

    // Move to specified, then put tasks in draft to trigger approve_tasks (agent verification).
    sdlc(&dir)
        .args(["artifact", "approve", "approval-gap", "spec"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "transition", "approval-gap", "specified"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["artifact", "approve", "approval-gap", "design"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["artifact", "draft", "approval-gap", "tasks"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["next", "--for", "approval-gap", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"action\": \"approve_tasks\""));

    sdlc(&dir)
        .args(["query", "needs-approval"])
        .assert()
        .success()
        .stdout(predicate::str::contains("approval-gap"))
        .stdout(predicate::str::contains(
            "artifact approve approval-gap tasks",
        ));
}

// ---------------------------------------------------------------------------
// sdlc task edit
// ---------------------------------------------------------------------------

#[test]
fn task_edit_title() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Old title"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "edit", "auth", "T1", "--title", "New title"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "get", "auth", "T1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("New title"))
        .stdout(predicate::str::contains("Old title").not());
}

#[test]
fn task_edit_description_and_depends() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "First task"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Second task"])
        .assert()
        .success();
    sdlc(&dir)
        .args([
            "task",
            "edit",
            "auth",
            "T2",
            "--description",
            "Must run after T1",
            "--depends",
            "T1",
        ])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "get", "auth", "T2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Must run after T1"))
        .stdout(predicate::str::contains("T1"));
}

// ---------------------------------------------------------------------------
// sdlc comment
// ---------------------------------------------------------------------------

#[test]
fn comment_create_and_list() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["comment", "create", "auth", "Spec section 3 is incomplete"])
        .assert()
        .success()
        .stdout(predicate::str::contains("C1"));
    sdlc(&dir)
        .args(["comment", "list", "auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Spec section 3 is incomplete"));
}

#[test]
fn comment_with_flag_and_task() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write tests"])
        .assert()
        .success();
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "auth",
            "Blocked by legal",
            "--flag",
            "blocker",
            "--task",
            "T1",
        ])
        .assert()
        .success();
    // Scoped list shows the comment
    sdlc(&dir)
        .args(["comment", "list", "auth", "--task", "T1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Blocked by legal"));
}

#[test]
fn comment_json_output() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args([
            "comment", "create", "auth", "A note", "--flag", "fyi", "--by", "alice",
        ])
        .assert()
        .success();
    let out = sdlc(&dir)
        .args(["--json", "comment", "list", "auth"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v[0]["flag"], "fyi");
    assert_eq!(v[0]["author"], "alice");
    assert_eq!(v[0]["body"], "A note");
}

#[test]
fn blocker_comment_surfaces_in_next() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "auth",
            "Waiting on security review",
            "--flag",
            "blocker",
        ])
        .assert()
        .success();
    // sdlc next should surface the blocker comment, not create_spec
    let out = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["action"], "wait_for_approval");
    assert!(v["message"].as_str().unwrap().contains("blocker comment"));
}

#[test]
fn comment_resolve_unblocks_pipeline() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();

    // Add a blocker comment — pipeline stalls
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "auth",
            "Waiting on Stripe account",
            "--flag",
            "blocker",
        ])
        .assert()
        .success();
    let out = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["action"], "wait_for_approval");

    // Resolve the comment — pipeline resumes
    sdlc(&dir)
        .args(["comment", "resolve", "auth", "C1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved comment [C1]"));
    let out2 = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v2: serde_json::Value = serde_json::from_slice(&out2).unwrap();
    assert_ne!(v2["action"], "wait_for_approval");
}

#[test]
fn comment_resolve_nonexistent_errors() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["comment", "resolve", "auth", "C99"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("not found"));
}

#[test]
fn question_flag_comment_blocks_pipeline() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();

    sdlc(&dir)
        .args([
            "comment",
            "create",
            "auth",
            "What auth strategy?",
            "--flag",
            "question",
        ])
        .assert()
        .success();
    let out = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["action"], "wait_for_approval");
    assert!(v["message"]
        .as_str()
        .unwrap()
        .contains("What auth strategy?"));
}

// ---------------------------------------------------------------------------
// sdlc project
// ---------------------------------------------------------------------------

#[test]
fn project_status_shows_feature_counts() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["project", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active features: 1"));
}

#[test]
fn project_status_json() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    let out = sdlc(&dir)
        .args(["--json", "project", "status"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["active_feature_count"], 1);
}

#[test]
fn project_stats_runs() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["project", "stats"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Actions"));
}

#[test]
fn project_blockers_empty() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["project", "blockers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No blocked features"));
}

// ---------------------------------------------------------------------------
// sdlc task get / search
// ---------------------------------------------------------------------------

#[test]
fn task_get_shows_detail() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write login form"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "get", "auth", "T1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Write login form"));
}

#[test]
fn task_search_returns_matching_task() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write login form"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write signup page"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "search", "login"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Write login form"));
    // "signup" should NOT appear
    sdlc(&dir)
        .args(["task", "search", "login"])
        .assert()
        .success()
        .stdout(predicate::str::contains("signup").not());
}

#[test]
fn task_search_no_results_exits_zero() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write login form"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "search", "notfound"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No tasks matching 'notfound'."));
}

#[test]
fn task_search_limit_flag_respected() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    // Add three tasks that all match "implement"
    for title in &["Implement login", "Implement signup", "Implement logout"] {
        sdlc(&dir)
            .args(["task", "add", "auth", title])
            .assert()
            .success();
    }
    // With --limit 1, only one result row should appear
    let out = sdlc(&dir)
        .args(["task", "search", "implement", "--limit", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    // The count header says "1 result"
    assert!(
        text.contains("1 result for"),
        "expected '1 result for' in output, got: {text}"
    );
}

#[test]
fn task_search_slug_scopes_to_feature() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "payments"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Fix login bug"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "payments", "Fix login redirect"])
        .assert()
        .success();
    // Scoped to "auth" — only auth task should appear
    sdlc(&dir)
        .args(["task", "search", "login", "--slug", "auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix login bug"))
        .stdout(predicate::str::contains("Fix login redirect").not());
}

#[test]
fn task_search_status_field_scope() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "api"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "api", "Build endpoint"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "api", "Write tests"])
        .assert()
        .success();
    // Block T1
    sdlc(&dir)
        .args(["task", "block", "api", "T1", "waiting for infra"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "search", "status:blocked"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build endpoint"))
        .stdout(predicate::str::contains("Write tests").not());
}

#[test]
fn task_search_json_output_has_score() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write login form"])
        .assert()
        .success();
    let out = sdlc(&dir)
        .args(["task", "search", "login", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(out).unwrap();
    let json: serde_json::Value = serde_json::from_str(&text).expect("invalid JSON");
    let arr = json.as_array().expect("expected JSON array");
    assert!(!arr.is_empty(), "expected at least one result");
    assert!(
        arr[0].get("score").is_some(),
        "expected 'score' field in JSON result"
    );
    assert_eq!(arr[0]["task_id"], "T1");
    assert_eq!(arr[0]["feature"], "auth");
}

// ---------------------------------------------------------------------------
// Happy-path smoke test through SPECIFIED phase
// ---------------------------------------------------------------------------

#[test]
fn happy_path_through_specified() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create feature
    sdlc(&dir)
        .args(["feature", "create", "user-auth", "--title", "User Auth"])
        .assert()
        .success();

    // Verify starts at draft / create_spec
    let out = sdlc(&dir)
        .args(["next", "--for", "user-auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["action"], "create_spec");
    assert_eq!(json["current_phase"], "draft");

    // Write and approve spec
    let spec_path = dir.path().join(".sdlc/features/user-auth/spec.md");
    std::fs::write(&spec_path, "# User Auth Spec\n\nDetails here.").unwrap();
    sdlc(&dir)
        .args(["artifact", "draft", "user-auth", "spec"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["artifact", "approve", "user-auth", "spec"])
        .assert()
        .success();

    // Transition to specified
    sdlc(&dir)
        .args(["feature", "transition", "user-auth", "specified"])
        .assert()
        .success();

    // Verify phase is now specified
    let out = sdlc(&dir)
        .args(["next", "--for", "user-auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["current_phase"], "specified");
    assert_eq!(json["action"], "create_design");
}

// ---------------------------------------------------------------------------
// sdlc milestone
// ---------------------------------------------------------------------------

#[test]
fn milestone_create_and_list() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["milestone", "create", "v2-launch", "--title", "v2.0 Launch"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Created milestone 'v2-launch'"));

    sdlc(&dir)
        .args(["milestone", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("v2-launch"))
        .stdout(predicates::str::contains("v2.0 Launch"))
        .stdout(predicates::str::contains("active"));
}

#[test]
fn milestone_create_duplicate_fails() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2 again"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("v2"));
}

#[test]
fn milestone_info_shows_details() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["milestone", "create", "beta", "--title", "Beta Release"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "info", "beta"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Beta Release"))
        .stdout(predicates::str::contains("active"));
}

#[test]
fn milestone_add_and_remove_feature() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "auth"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Added feature 'auth' to milestone 'v2'",
        ));

    // Adding again fails (already present)
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "auth"])
        .assert()
        .failure();

    sdlc(&dir)
        .args(["milestone", "remove-feature", "v2", "auth"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Removed feature 'auth' from milestone 'v2'",
        ));

    // Removing again fails (not present)
    sdlc(&dir)
        .args(["milestone", "remove-feature", "v2", "auth"])
        .assert()
        .failure();
}

#[test]
fn milestone_complete_and_cancel() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "complete", "v2"])
        .assert()
        .success()
        .stdout(predicates::str::contains("marked complete"));

    sdlc(&dir)
        .args(["milestone", "create", "v3", "--title", "v3"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "cancel", "v3"])
        .assert()
        .success()
        .stdout(predicates::str::contains("cancelled"));
}

#[test]
fn milestone_tasks_aggregates_across_features() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Write login form"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "auth"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "tasks", "v2"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Write login form"))
        .stdout(predicates::str::contains("auth"));
}

#[test]
fn milestone_review_shows_classifier_output() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "auth"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "review", "v2"])
        .assert()
        .success()
        .stdout(predicates::str::contains("auth"))
        .stdout(predicates::str::contains("no")); // not blocked
}

#[test]
fn milestone_review_json() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "auth"])
        .assert()
        .success();

    let out = sdlc(&dir)
        .args(["--json", "milestone", "review", "v2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["milestone"], "v2");
    assert_eq!(json["feature_count"], 1);
    assert!(json["features"].as_array().unwrap().len() == 1);
    assert_eq!(json["features"][0]["feature"], "auth");
    assert_eq!(json["features"][0]["blocked"], false);
}

// ---------------------------------------------------------------------------
// sdlc milestone reorder / add-feature --position
// ---------------------------------------------------------------------------

#[test]
fn milestone_reorder_changes_feature_order() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "f1", "--title", "F1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "f2", "--title", "F2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "f3", "--title", "F3"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f3"])
        .assert()
        .success();

    // Reorder: f3, f1, f2
    sdlc(&dir)
        .args(["milestone", "reorder", "v2", "f3", "f1", "f2"])
        .assert()
        .success()
        .stdout(predicates::str::contains("1. f3"))
        .stdout(predicates::str::contains("2. f1"))
        .stdout(predicates::str::contains("3. f2"));

    // Verify JSON round-trip
    let out = sdlc(&dir)
        .args(["--json", "milestone", "info", "v2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["features"][0], "f3");
    assert_eq!(json["features"][1], "f1");
    assert_eq!(json["features"][2], "f2");
}

#[test]
fn milestone_reorder_rejects_missing_slug() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "f1", "--title", "F1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "f2", "--title", "F2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f2"])
        .assert()
        .success();

    // Only provide f1 — f2 is missing
    sdlc(&dir)
        .args(["milestone", "reorder", "v2", "f1"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing slug"));
}

#[test]
fn milestone_reorder_rejects_extra_slug() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "f1", "--title", "F1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f1"])
        .assert()
        .success();

    // "ghost" is not in the milestone
    sdlc(&dir)
        .args(["milestone", "reorder", "v2", "f1", "ghost"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not in this milestone"));
}

#[test]
fn milestone_add_feature_with_position() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "f1", "--title", "F1"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "f2", "--title", "F2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "create", "f3", "--title", "F3"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f2"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f3"])
        .assert()
        .success();

    // Insert f1 at position 0 — should be first
    sdlc(&dir)
        .args(["milestone", "add-feature", "v2", "f1", "--position", "0"])
        .assert()
        .success();

    let out = sdlc(&dir)
        .args(["--json", "milestone", "info", "v2"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(json["features"][0], "f1");
    assert_eq!(json["features"][1], "f2");
    assert_eq!(json["features"][2], "f3");
}

// ---------------------------------------------------------------------------
// sdlc platform
// ---------------------------------------------------------------------------

fn init_with_platform(dir: &TempDir) {
    sdlc(dir)
        .args(["init", "--platform", "masquerade"])
        .assert()
        .success();
}

#[test]
fn platform_init_creates_scripts_and_config() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    assert!(dir.path().join(".sdlc/platform/deploy.sh").exists());
    assert!(dir.path().join(".sdlc/platform/logs.sh").exists());
    assert!(dir.path().join(".sdlc/platform/dev-start.sh").exists());
    assert!(dir.path().join(".sdlc/platform/dev-stop.sh").exists());
    assert!(dir.path().join(".sdlc/platform/dev-quality.sh").exists());
    assert!(dir.path().join(".sdlc/platform/dev-migrate.sh").exists());

    let config = std::fs::read_to_string(dir.path().join(".sdlc/config.yaml")).unwrap();
    assert!(config.contains("platform:"));
    assert!(config.contains("deploy"));
}

#[test]
fn platform_list_shows_commands() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "list"])
        .assert()
        .success()
        .stdout(predicates::str::contains("deploy"))
        .stdout(predicates::str::contains("logs"))
        .stdout(predicates::str::contains("dev"));
}

#[test]
fn platform_list_json() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    let out = sdlc(&dir)
        .args(["--json", "platform", "list"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let arr = json.as_array().unwrap();
    let names: Vec<&str> = arr.iter().map(|v| v["name"].as_str().unwrap()).collect();
    assert!(names.contains(&"deploy"));
    assert!(names.contains(&"logs"));
    assert!(names.contains(&"dev"));
}

#[test]
fn platform_missing_required_arg_errors() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    // deploy with no args — missing required 'service'
    sdlc(&dir)
        .args(["platform", "deploy"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("missing required argument"));
}

#[test]
fn platform_invalid_choice_errors() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    // deploy with invalid environment
    sdlc(&dir)
        .args(["platform", "deploy", "auth-service", "badenv"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("invalid value"));
}

#[test]
fn platform_no_config_errors() {
    let dir = TempDir::new().unwrap();
    // plain init — no --platform flag
    sdlc(&dir).arg("init").assert().success();

    sdlc(&dir)
        .args(["platform", "list"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("no platform commands configured"));
}

#[test]
fn platform_deploy_shows_run_directive() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "deploy", "auth-service", "staging"])
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "sdlc does not execute platform scripts",
        ))
        .stderr(predicates::str::contains("deploy.sh"));
}

#[test]
fn platform_dev_subcommand_shows_run_directive() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "dev", "start"])
        .assert()
        .success()
        .stderr(predicates::str::contains(
            "sdlc does not execute platform scripts",
        ))
        .stderr(predicates::str::contains("dev-start.sh"));
}

#[test]
fn platform_dev_unknown_subcommand_errors() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "dev", "bogus"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("unknown subcommand"));
}

#[test]
fn platform_init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);
    // Running again must not fail or duplicate config
    init_with_platform(&dir);

    let config = std::fs::read_to_string(dir.path().join(".sdlc/config.yaml")).unwrap();
    let count = config.matches("platform:").count();
    assert_eq!(count, 1, "platform: section should appear exactly once");
}

// ---------------------------------------------------------------------------
// sdlc milestone update
// ---------------------------------------------------------------------------

#[test]
fn milestone_update_changes_title() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["milestone", "create", "v2", "--title", "v2 Launch"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["milestone", "update", "v2", "--title", "v2.0 Final Launch"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated milestone 'v2'"));

    sdlc(&dir)
        .args(["milestone", "info", "v2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("v2.0 Final Launch"))
        .stdout(predicate::str::contains("v2 Launch").not());
}

// ---------------------------------------------------------------------------
// E2E: Full pipeline from init to specified phase
// ---------------------------------------------------------------------------

#[test]
fn e2e_full_pipeline_init_to_specified() {
    let dir = TempDir::new().unwrap();

    // 1. sdlc init — creates project structure
    init_project(&dir);
    assert!(dir.path().join(".sdlc").is_dir());
    assert!(dir.path().join(".sdlc/config.yaml").exists());

    // 2. Write a VISION.md file to the temp dir root
    std::fs::write(
        dir.path().join("VISION.md"),
        "# Project Vision\n\nBuild a world-class authentication system.\n",
    )
    .unwrap();
    assert!(dir.path().join("VISION.md").exists());

    // 3. sdlc milestone create mvp --title "MVP Release"
    sdlc(&dir)
        .args(["milestone", "create", "mvp", "--title", "MVP Release"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Created milestone 'mvp'"));

    // 4. sdlc feature create auth-login --title "Auth Login"
    sdlc(&dir)
        .args(["feature", "create", "auth-login", "--title", "Auth Login"])
        .assert()
        .success();

    // 5. sdlc feature create user-profile --title "User Profile"
    sdlc(&dir)
        .args([
            "feature",
            "create",
            "user-profile",
            "--title",
            "User Profile",
        ])
        .assert()
        .success();

    // 6. sdlc milestone add-feature mvp auth-login
    sdlc(&dir)
        .args(["milestone", "add-feature", "mvp", "auth-login"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added feature 'auth-login' to milestone 'mvp'",
        ));

    // 7. sdlc milestone add-feature mvp user-profile
    sdlc(&dir)
        .args(["milestone", "add-feature", "mvp", "user-profile"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Added feature 'user-profile' to milestone 'mvp'",
        ));

    // 8. sdlc milestone review mvp --json — verify both features show in review
    let review_out = sdlc(&dir)
        .args(["--json", "milestone", "review", "mvp"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let review_json: serde_json::Value = serde_json::from_slice(&review_out).unwrap();
    assert_eq!(review_json["milestone"], "mvp");
    assert_eq!(review_json["feature_count"], 2);
    let features = review_json["features"].as_array().unwrap();
    assert_eq!(features.len(), 2);
    let feature_names: Vec<&str> = features
        .iter()
        .map(|f| f["feature"].as_str().unwrap())
        .collect();
    assert!(feature_names.contains(&"auth-login"));
    assert!(feature_names.contains(&"user-profile"));
    // Both should be in draft phase and not blocked
    for f in features {
        assert_eq!(f["phase"], "draft");
        assert_eq!(f["blocked"], false);
    }

    // 9. For auth-login: draft spec -> approve spec -> transition to specified
    sdlc(&dir)
        .args(["artifact", "draft", "auth-login", "spec"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["artifact", "approve", "auth-login", "spec"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["feature", "transition", "auth-login", "specified"])
        .assert()
        .success();

    // 10. Verify auth-login is now at "specified" phase via sdlc next --for auth-login --json
    let next_auth = sdlc(&dir)
        .args(["next", "--for", "auth-login", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_auth_json: serde_json::Value = serde_json::from_slice(&next_auth).unwrap();
    assert_eq!(next_auth_json["current_phase"], "specified");

    // 11. Verify user-profile is still at "draft" via sdlc next --for user-profile --json
    let next_profile = sdlc(&dir)
        .args(["next", "--for", "user-profile", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_profile_json: serde_json::Value = serde_json::from_slice(&next_profile).unwrap();
    assert_eq!(next_profile_json["current_phase"], "draft");
    assert_eq!(next_profile_json["action"], "create_spec");

    // 12. sdlc milestone review mvp --json — verify auth-login shows "specified",
    //     user-profile shows "draft"
    let review2_out = sdlc(&dir)
        .args(["--json", "milestone", "review", "mvp"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let review2_json: serde_json::Value = serde_json::from_slice(&review2_out).unwrap();
    let features2 = review2_json["features"].as_array().unwrap();
    let auth_entry = features2
        .iter()
        .find(|f| f["feature"] == "auth-login")
        .unwrap();
    let profile_entry = features2
        .iter()
        .find(|f| f["feature"] == "user-profile")
        .unwrap();
    assert_eq!(auth_entry["phase"], "specified");
    assert_eq!(profile_entry["phase"], "draft");

    // 13. sdlc state — verify overall state includes both features
    sdlc(&dir)
        .arg("state")
        .assert()
        .success()
        .stdout(predicate::str::contains("auth-login"))
        .stdout(predicate::str::contains("user-profile"));

    // 14. sdlc project status — verify active feature count is 2
    sdlc(&dir)
        .args(["project", "status"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Active features: 2"));
}

// ---------------------------------------------------------------------------
// E2E: Task and comment lifecycle with blocker/resume flow
// ---------------------------------------------------------------------------

#[test]
fn e2e_task_and_comment_lifecycle() {
    let dir = TempDir::new().unwrap();

    // 1. Init + create feature
    init_project(&dir);
    sdlc(&dir)
        .args([
            "feature",
            "create",
            "payments",
            "--title",
            "Payment Integration",
        ])
        .assert()
        .success();

    // 2. Add 3 tasks
    sdlc(&dir)
        .args(["task", "add", "payments", "Integrate Stripe SDK"])
        .assert()
        .success()
        .stdout(predicate::str::contains("T1"));

    sdlc(&dir)
        .args(["task", "add", "payments", "Build checkout form"])
        .assert()
        .success()
        .stdout(predicate::str::contains("T2"));

    sdlc(&dir)
        .args(["task", "add", "payments", "Write payment tests"])
        .assert()
        .success()
        .stdout(predicate::str::contains("T3"));

    // Start T1 and complete it
    sdlc(&dir)
        .args(["task", "start", "payments", "T1"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["task", "complete", "payments", "T1"])
        .assert()
        .success();

    // Verify T1 is completed
    sdlc(&dir)
        .args(["task", "list", "payments"])
        .assert()
        .success()
        .stdout(predicate::str::contains("completed"));

    // 3. Add a blocker comment — verify pipeline stalls (next returns wait_for_approval)
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "payments",
            "Waiting on Stripe API key from finance team",
            "--flag",
            "blocker",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("C1"));

    let next_blocked = sdlc(&dir)
        .args(["next", "--for", "payments", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_blocked_json: serde_json::Value = serde_json::from_slice(&next_blocked).unwrap();
    assert_eq!(next_blocked_json["action"], "wait_for_approval");
    assert!(next_blocked_json["message"]
        .as_str()
        .unwrap()
        .contains("blocker comment"));

    // 4. Resolve the blocker — verify pipeline resumes
    sdlc(&dir)
        .args(["comment", "resolve", "payments", "C1"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved comment [C1]"));

    let next_resumed = sdlc(&dir)
        .args(["next", "--for", "payments", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_resumed_json: serde_json::Value = serde_json::from_slice(&next_resumed).unwrap();
    assert_ne!(next_resumed_json["action"], "wait_for_approval");
    // Should be back to normal pipeline (create_spec for draft phase)
    assert_eq!(next_resumed_json["action"], "create_spec");
    assert_eq!(next_resumed_json["current_phase"], "draft");

    // 5. Search tasks by keyword — verify correct results
    sdlc(&dir)
        .args(["task", "search", "Stripe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Integrate Stripe SDK"))
        .stdout(predicate::str::contains("checkout").not());

    sdlc(&dir)
        .args(["task", "search", "checkout"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Build checkout form"))
        .stdout(predicate::str::contains("Stripe").not());

    // Search scoped to the feature slug
    sdlc(&dir)
        .args(["task", "search", "tests", "--slug", "payments"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Write payment tests"));

    // 6. sdlc project blockers: initially no blocked features (comment-based blockers
    //    stall the pipeline via `next` but do not populate state.blocked / feature.blockers)
    sdlc(&dir)
        .args(["project", "blockers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No blocked features"));

    // Add a new blocker comment and verify next stalls again
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "payments",
            "Legal review pending for PCI compliance",
            "--flag",
            "blocker",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("C2"));

    let next_blocked2 = sdlc(&dir)
        .args(["next", "--for", "payments", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_blocked2_json: serde_json::Value = serde_json::from_slice(&next_blocked2).unwrap();
    assert_eq!(next_blocked2_json["action"], "wait_for_approval");
    assert!(next_blocked2_json["message"]
        .as_str()
        .unwrap()
        .contains("PCI compliance"));

    // Resolve the second blocker and verify pipeline resumes
    sdlc(&dir)
        .args(["comment", "resolve", "payments", "C2"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Resolved comment [C2]"));

    let next_final = sdlc(&dir)
        .args(["next", "--for", "payments", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let next_final_json: serde_json::Value = serde_json::from_slice(&next_final).unwrap();
    assert_eq!(next_final_json["action"], "create_spec");

    // Confirm project blockers is still clean after resolving all comments
    sdlc(&dir)
        .args(["project", "blockers"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No blocked features"));
}

// ---------------------------------------------------------------------------
// sdlc query search
// ---------------------------------------------------------------------------

#[test]
fn query_search_finds_by_title() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "feature",
            "create",
            "auth-login",
            "--title",
            "User Authentication",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["query", "search", "authentication"])
        .assert()
        .success()
        .stdout(predicate::str::contains("auth-login"));
}

#[test]
fn query_search_finds_by_description() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "feature",
            "create",
            "payments",
            "--title",
            "Payment Flow",
            "--description",
            "Stripe checkout integration",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["query", "search", "stripe"])
        .assert()
        .success()
        .stdout(predicate::str::contains("payments"));
}

#[test]
fn query_search_no_match_prints_no_results() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["query", "search", "kubernetes"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results."));
}

#[test]
fn query_search_phase_field_scope() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "feat-a", "--title", "Feature A"])
        .assert()
        .success();

    // All fresh features are in draft phase
    sdlc(&dir)
        .args(["query", "search", "phase:draft"])
        .assert()
        .success()
        .stdout(predicate::str::contains("feat-a"));
}

#[test]
fn query_search_limit_respected() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    for slug in ["feat-a", "feat-b", "feat-c"] {
        sdlc(&dir)
            .args([
                "feature",
                "create",
                slug,
                "--title",
                &format!("Feature {slug}"),
            ])
            .assert()
            .success();
    }

    // Search for a common term with limit 1 — output must contain exactly one slug reference
    let output = sdlc(&dir)
        .args(["query", "search", "phase:draft", "--limit", "1"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let text = String::from_utf8_lossy(&output);
    let slug_count = ["feat-a", "feat-b", "feat-c"]
        .iter()
        .filter(|&&s| text.contains(s))
        .count();
    assert_eq!(slug_count, 1, "only 1 result expected due to --limit 1");
}

#[test]
fn query_search_json_output() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "feature",
            "create",
            "oauth-flow",
            "--title",
            "OAuth Login Flow",
        ])
        .assert()
        .success();

    let output = sdlc(&dir)
        .args(["query", "search", "oauth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON");
    let arr = json.as_array().expect("JSON array");
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["slug"], "oauth-flow");
    assert_eq!(arr[0]["phase"], "draft");
    assert!(arr[0]["score"].as_f64().unwrap() > 0.0);
}

#[test]
fn query_search_empty_project_returns_no_results() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["query", "search", "anything"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No results."));
}

// ---------------------------------------------------------------------------
// sdlc artifact waive
// ---------------------------------------------------------------------------

#[test]
fn artifact_waive_unblocks_pipeline() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "simple-crud"])
        .assert()
        .success();

    // Approve spec and transition to specified
    sdlc(&dir)
        .args(["artifact", "draft", "simple-crud", "spec"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["artifact", "approve", "simple-crud", "spec"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "transition", "simple-crud", "specified"])
        .assert()
        .success();

    // Next should be create_design
    sdlc(&dir)
        .args(["next", "--for", "simple-crud", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_design"));

    // Waive design — simple CRUD needs no architecture doc
    sdlc(&dir)
        .args([
            "artifact",
            "waive",
            "simple-crud",
            "design",
            "--reason",
            "simple CRUD, no arch decisions",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Waived: simple-crud/design"));

    // Next should now skip design and go to create_tasks
    sdlc(&dir)
        .args(["next", "--for", "simple-crud", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_tasks"));
}

#[test]
fn artifact_waive_spec_skips_to_specified() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "pure-refactor"])
        .assert()
        .success();

    // Next should be create_spec
    sdlc(&dir)
        .args(["next", "--for", "pure-refactor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_spec"));

    // Waive spec — this is a pure refactor with no behavioral change
    sdlc(&dir)
        .args([
            "artifact",
            "waive",
            "pure-refactor",
            "spec",
            "--reason",
            "pure refactor, behavior unchanged",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Waived: pure-refactor/spec"));

    // Next should now transition to specified
    sdlc(&dir)
        .args(["next", "--for", "pure-refactor", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("specified"));
}

#[test]
fn artifact_waive_audit_skips_to_qa() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "config-fix"])
        .assert()
        .success();

    // Waive review so we can transition to audit (review is required to enter audit)
    sdlc(&dir)
        .args([
            "artifact",
            "waive",
            "config-fix",
            "review",
            "--reason",
            "trivial config change, review waived",
        ])
        .assert()
        .success();

    // Transition to audit phase
    sdlc(&dir)
        .args(["feature", "transition", "config-fix", "audit"])
        .assert()
        .success();

    // Next should be create_audit
    sdlc(&dir)
        .args(["next", "--for", "config-fix", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_audit"));

    // Waive audit — trivial config change, no security surface
    sdlc(&dir)
        .args([
            "artifact",
            "waive",
            "config-fix",
            "audit",
            "--reason",
            "trivial config change, no security surface",
        ])
        .assert()
        .success();

    // Next should now transition to QA
    sdlc(&dir)
        .args(["next", "--for", "config-fix", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("qa"));
}

#[test]
fn artifact_waive_json_output() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "waive-json-test"])
        .assert()
        .success();

    let output = sdlc(&dir)
        .args([
            "artifact",
            "waive",
            "waive-json-test",
            "design",
            "--reason",
            "no design needed",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let json: serde_json::Value =
        serde_json::from_slice(&output).expect("output should be valid JSON");
    assert_eq!(json["slug"], "waive-json-test");
    assert_eq!(json["artifact"], "design");
    assert_eq!(json["status"], "waived");
    assert_eq!(json["reason"], "no design needed");
}

// ---------------------------------------------------------------------------
// sdlc agent — CLI parsing and error paths (no Claude subprocess)
// ---------------------------------------------------------------------------

#[test]
fn agent_help_works() {
    // Verifies the subcommand is wired and --help doesn't panic.
    Command::cargo_bin("sdlc")
        .unwrap()
        .args(["agent", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("run"));
}

#[test]
fn agent_run_missing_feature_fails() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["agent", "run", "no-such-feature"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("no-such-feature"));
}

// sdlc agent run — short-circuit paths (no Claude subprocess)
// ---------------------------------------------------------------------------

#[test]
fn agent_run_done_feature_exits_without_spawning_claude() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    // Create feature and drive it all the way to Released (done).
    sdlc(&dir)
        .args(["feature", "create", "done-feature"])
        .assert()
        .success();

    // Approve qa_results to unlock merge phase, then transition and merge.
    sdlc(&dir)
        .args(["artifact", "approve", "done-feature", "qa_results"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["feature", "transition", "done-feature", "merge"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["merge", "done-feature"])
        .assert()
        .success();

    // sdlc agent run should detect Done and exit cleanly without spawning Claude.
    sdlc(&dir)
        .args(["agent", "run", "done-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already done"));
}

#[test]
fn agent_run_hitl_gate_exits_without_spawning_claude() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "blocked-feature"])
        .assert()
        .success();

    // Add a blocker comment — this stalls the pipeline at wait_for_approval.
    sdlc(&dir)
        .args([
            "comment",
            "create",
            "blocked-feature",
            "Needs product sign-off before spec is written",
            "--flag",
            "blocker",
        ])
        .assert()
        .success();

    // sdlc agent run should detect the human gate and exit cleanly.
    sdlc(&dir)
        .args(["agent", "run", "blocked-feature"])
        .assert()
        .success()
        .stdout(predicate::str::contains("human gate"));
}
