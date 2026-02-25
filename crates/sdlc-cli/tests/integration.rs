#![allow(deprecated)]
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn sdlc(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("sdlc").unwrap();
    cmd.current_dir(dir.path()).env("SDLC_ROOT", dir.path());
    cmd
}

fn init_project(dir: &TempDir) {
    sdlc(dir).arg("init").assert().success();
}

// ---------------------------------------------------------------------------
// sdlc init
// ---------------------------------------------------------------------------

#[test]
fn init_creates_directory_tree() {
    let dir = TempDir::new().unwrap();
    sdlc(&dir).arg("init").assert().success();

    assert!(dir.path().join(".sdlc").is_dir());
    assert!(dir.path().join(".sdlc/features").is_dir());
    assert!(dir.path().join(".sdlc/config.yaml").exists());
    assert!(dir.path().join(".sdlc/state.yaml").exists());
    assert!(dir.path().join(".ai").is_dir());
    assert!(dir.path().join(".ai/index.md").exists());
    assert!(dir.path().join(".ai/patterns").is_dir());
    assert!(dir.path().join(".ai/decisions").is_dir());
    assert!(dir.path().join("AGENTS.md").exists());
    assert!(dir.path().join(".claude/commands/sdlc-next.md").exists());
    assert!(dir.path().join(".claude/commands/sdlc-status.md").exists());
    assert!(dir.path().join(".claude/commands/sdlc-approve.md").exists());
}

#[test]
fn init_is_idempotent() {
    let dir = TempDir::new().unwrap();
    // Run twice — should succeed both times without error
    sdlc(&dir).arg("init").assert().success();
    sdlc(&dir).arg("init").assert().success();
}

#[test]
fn init_appends_sdlc_section_to_existing_agents_md() {
    let dir = TempDir::new().unwrap();
    std::fs::write(dir.path().join("AGENTS.md"), "# Existing content\n").unwrap();
    sdlc(&dir).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    assert!(content.contains("# Existing content"));
    assert!(content.contains("## SDLC"));
}

#[test]
fn init_does_not_duplicate_sdlc_section() {
    let dir = TempDir::new().unwrap();
    sdlc(&dir).arg("init").assert().success();
    sdlc(&dir).arg("init").assert().success();

    let content = std::fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();
    let count = content.matches("## SDLC").count();
    assert_eq!(count, 1, "SDLC section should appear exactly once");
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
fn task_search_finds_match() {
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
fn task_search_scoped_to_slug() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);
    sdlc(&dir)
        .args(["feature", "create", "auth"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "add", "auth", "Fix login bug"])
        .assert()
        .success();
    sdlc(&dir)
        .args(["task", "search", "login", "--slug", "auth"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Fix login bug"));
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
fn platform_deploy_executes_script() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "deploy", "auth-service", "staging"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Deploying auth-service to staging",
        ));
}

#[test]
fn platform_dev_subcommand_dispatches() {
    let dir = TempDir::new().unwrap();
    init_with_platform(&dir);

    sdlc(&dir)
        .args(["platform", "dev", "start"])
        .assert()
        .success()
        .stdout(predicates::str::contains(
            "Starting development environment",
        ));
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
// sdlc config agent
// ---------------------------------------------------------------------------

#[test]
fn config_agent_show_defaults() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["config", "agent", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude_agent_sdk"));
}

#[test]
fn config_agent_show_json() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    let out = sdlc(&dir)
        .args(["--json", "config", "agent", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert!(v.get("default").is_some(), "JSON must have 'default' key");
    assert!(v.get("actions").is_some(), "JSON must have 'actions' key");
}

#[test]
fn config_agent_set_default_xadk() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-default",
            "--type",
            "xadk",
            "--agent-id",
            "sdlc_spec",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["config", "agent", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("xadk"))
        .stdout(predicate::str::contains("sdlc_spec"));
}

#[test]
fn config_agent_set_default_claude() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-default",
            "--type",
            "claude",
            "--model",
            "claude-haiku-4-5",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["config", "agent", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude-haiku-4-5"));
}

#[test]
fn config_agent_set_action_override() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-action",
            "create_spec",
            "--type",
            "xadk",
            "--agent-id",
            "sdlc_spec",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["config", "agent", "show"])
        .assert()
        .success()
        .stdout(predicate::str::contains("create_spec"))
        .stdout(predicate::str::contains("sdlc_spec"));
}

#[test]
fn config_agent_reset_clears_overrides() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-action",
            "create_spec",
            "--type",
            "human",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["config", "agent", "reset"])
        .assert()
        .success()
        .stdout(predicate::str::contains("cleared"));

    let out = sdlc(&dir)
        .args(["--json", "config", "agent", "show"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(
        v["actions"],
        serde_json::json!({}),
        "actions map must be empty after reset"
    );
}

// ---------------------------------------------------------------------------
// sdlc run
// ---------------------------------------------------------------------------

#[test]
fn run_human_backend_exits_zero() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "my-feat"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["config", "agent", "set-default", "--type", "human"])
        .assert()
        .success();

    sdlc(&dir)
        .args(["run", "my-feat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Human action required"));
}

#[test]
fn run_dry_run_claude_prints_command() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "my-feat"])
        .assert()
        .success();

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-default",
            "--type",
            "claude",
            "--model",
            "claude-opus-4-6",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["run", "my-feat", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("claude"))
        .stdout(predicate::str::contains("-p"));
}

#[test]
fn run_dry_run_xadk_prints_command() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "my-feat"])
        .assert()
        .success();

    sdlc(&dir)
        .args([
            "config",
            "agent",
            "set-default",
            "--type",
            "xadk",
            "--agent-id",
            "sdlc_spec",
        ])
        .assert()
        .success();

    sdlc(&dir)
        .args(["run", "my-feat", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("python"))
        .stdout(predicate::str::contains("xadk"))
        .stdout(predicate::str::contains("sdlc_spec"));
}

#[test]
fn run_done_exits_zero_with_message() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "released-feat"])
        .assert()
        .success();

    // Force the feature to Released phase by patching its manifest directly
    let manifest_path = dir
        .path()
        .join(".sdlc/features/released-feat/manifest.yaml");
    let manifest = std::fs::read_to_string(&manifest_path).unwrap();
    let updated = manifest.replace("phase: draft", "phase: released");
    std::fs::write(&manifest_path, updated).unwrap();

    sdlc(&dir)
        .args(["run", "released-feat"])
        .assert()
        .success()
        .stdout(predicate::str::contains("complete — no pending actions"));
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
// Verification Gates
// ---------------------------------------------------------------------------

#[test]
fn next_json_includes_gates_when_configured() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    // Write config with gates for create_spec
    let config_path = dir.path().join(".sdlc/config.yaml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let config_with_gates = format!(
        "{}\ngates:\n  create_spec:\n    - name: lint\n      gate_type:\n        type: shell\n        command: \"echo lint ok\"\n      max_retries: 1\n      timeout_seconds: 30\n",
        config.trim()
    );
    std::fs::write(&config_path, config_with_gates).unwrap();

    let output = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["action"], "create_spec");
    assert!(json["gates"].is_array());
    assert_eq!(json["gates"][0]["name"], "lint");
    assert_eq!(json["gates"][0]["max_retries"], 1);
}

#[test]
fn next_json_omits_gates_when_not_configured() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    let output = sdlc(&dir)
        .args(["next", "--for", "auth", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let json: serde_json::Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(json["action"], "create_spec");
    // gates should not appear in JSON when empty (skip_serializing_if)
    assert!(json.get("gates").is_none());
}

#[test]
fn run_dry_run_shows_gates() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    // Write config with gates for create_spec
    let config_path = dir.path().join(".sdlc/config.yaml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let config_with_gates = format!(
        "{}\ngates:\n  create_spec:\n    - name: build\n      gate_type:\n        type: shell\n        command: \"npm run build\"\n      max_retries: 2\n      timeout_seconds: 120\n    - name: review\n      gate_type:\n        type: human\n        prompt: \"Review the spec before proceeding\"\n      auto: false\n",
        config.trim()
    );
    std::fs::write(&config_path, config_with_gates).unwrap();

    sdlc(&dir)
        .args(["run", "auth", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Gates after create_spec"))
        .stdout(predicate::str::contains("shell: build"))
        .stdout(predicate::str::contains("npm run build"))
        .stdout(predicate::str::contains("human: review"));
}

#[test]
fn config_backward_compat_no_gates() {
    // A config without any gates section should still work
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    // Dry run should work without gates (no "Gates after" output)
    let output = sdlc(&dir)
        .args(["run", "auth", "--dry-run"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let stdout = String::from_utf8(output).unwrap();
    assert!(!stdout.contains("Gates after"));
}

#[test]
fn run_with_passing_gates_exits_zero() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    // Create a mock "claude" binary that exits 0
    let bin_dir = dir.path().join("mock_bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let mock_claude = bin_dir.join("claude");
    std::fs::write(&mock_claude, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Configure gates for create_spec
    let config_path = dir.path().join(".sdlc/config.yaml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let config_with_gates = format!(
        "{}\ngates:\n  create_spec:\n    - name: check\n      gate_type:\n        type: shell\n        command: \"echo gate-passed\"\n      timeout_seconds: 5\n",
        config.trim()
    );
    std::fs::write(&config_path, config_with_gates).unwrap();

    // Run with mock claude in PATH — agent succeeds, gate passes, exit 0
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    sdlc(&dir)
        .args(["run", "auth"])
        .env("PATH", &path)
        .assert()
        .success()
        .stderr(predicate::str::contains("Running verification gates"))
        .stderr(predicate::str::contains("All gates passed"));
}

#[test]
fn run_with_failing_gate_exits_two() {
    let dir = TempDir::new().unwrap();
    init_project(&dir);

    sdlc(&dir)
        .args(["feature", "create", "auth", "--title", "Auth System"])
        .assert()
        .success();

    // Create a mock "claude" binary that exits 0
    let bin_dir = dir.path().join("mock_bin");
    std::fs::create_dir_all(&bin_dir).unwrap();
    let mock_claude = bin_dir.join("claude");
    std::fs::write(&mock_claude, "#!/bin/sh\nexit 0\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&mock_claude, std::fs::Permissions::from_mode(0o755)).unwrap();
    }

    // Configure a failing gate for create_spec
    let config_path = dir.path().join(".sdlc/config.yaml");
    let config = std::fs::read_to_string(&config_path).unwrap();
    let config_with_gates = format!(
        "{}\ngates:\n  create_spec:\n    - name: must-fail\n      gate_type:\n        type: shell\n        command: \"false\"\n      timeout_seconds: 5\n",
        config.trim()
    );
    std::fs::write(&config_path, config_with_gates).unwrap();

    // Run with mock claude in PATH — agent succeeds, gate fails, exit 2
    let path = format!(
        "{}:{}",
        bin_dir.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    sdlc(&dir)
        .args(["run", "auth"])
        .env("PATH", &path)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("Running verification gates"))
        .stderr(predicate::str::contains("must-fail"));
}
