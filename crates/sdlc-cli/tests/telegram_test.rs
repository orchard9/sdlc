#![allow(deprecated)]
/// Integration tests for `sdlc telegram` subcommands.
///
/// These tests verify the CLI's behavior at the boundary — particularly that
/// it fails gracefully with a clear error message when the bot token is absent.
/// No live Telegram network calls are made.
use assert_cmd::Command;
use predicates::prelude::*;
use sdlc_core::telegram::MessageStore;
use tempfile::TempDir;

fn sdlc_with_home(project: &TempDir, home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("ponder").unwrap();
    cmd.current_dir(project.path())
        .env("SDLC_ROOT", project.path())
        .env("HOME", home.path());
    cmd
}

fn init_project(project: &TempDir) {
    let home = TempDir::new().unwrap();
    sdlc_with_home(project, &home)
        .arg("init")
        .assert()
        .success();
}

/// `sdlc telegram status` must fail with a clear error when no bot token is
/// configured (neither env var nor config.yaml).
#[test]
fn test_status_missing_token() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    init_project(&dir);

    sdlc_with_home(&dir, &home)
        .args(["telegram", "status"])
        // Ensure the env var is not set for this invocation
        .env_remove("TELEGRAM_BOT_TOKEN")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("TELEGRAM_BOT_TOKEN")
                .or(predicate::str::contains("bot token")),
        );
}

/// `sdlc telegram poll` must also fail with a clear error when no token is set.
#[test]
fn test_poll_missing_token() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    init_project(&dir);

    sdlc_with_home(&dir, &home)
        .args(["telegram", "poll"])
        .env_remove("TELEGRAM_BOT_TOKEN")
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("TELEGRAM_BOT_TOKEN")
                .or(predicate::str::contains("bot token")),
        );
}

/// `sdlc telegram digest --dry-run` must read messages from the SQLite DB that
/// `sdlc telegram poll` populates, not from the live Telegram API.
#[test]
fn test_digest_dry_run_reads_from_db() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    init_project(&dir);

    // Seed the SQLite DB with 3 messages within the last 24h
    let db_path = dir.path().join(".sdlc/telegram/messages.db");
    let store = MessageStore::open(&db_path).unwrap();
    let now_ts = chrono::Utc::now().timestamp();
    for i in 0i64..3 {
        let msg = serde_json::json!({
            "chat": { "id": -100123456789i64, "title": "Test Chat" },
            "from": { "id": i + 1, "username": format!("user{i}"), "first_name": format!("User{i}") },
            "date": now_ts - 3600 - i * 100,
            "text": format!("message {i}")
        });
        store.insert_message(i + 1, &msg).unwrap();
    }
    drop(store);

    sdlc_with_home(&dir, &home)
        .args(["telegram", "digest", "--dry-run", "--json"])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .env("RESEND_API_KEY", "re_test_key")
        .env("RESEND_FROM", "from@test.com")
        .env("RESEND_TO", "to@test.com")
        .env_remove("TELEGRAM_CHAT_IDS")
        .assert()
        .success()
        .stdout(predicate::str::contains("\"total_messages\""))
        .stdout(predicate::str::contains("\"dry_run\""));
}

/// `sdlc telegram digest` must fail with a clear error when RESEND_API_KEY is not set.
#[test]
fn test_digest_missing_resend_api_key() {
    let dir = TempDir::new().unwrap();
    let home = TempDir::new().unwrap();

    init_project(&dir);

    sdlc_with_home(&dir, &home)
        .args(["telegram", "digest", "--dry-run", "--json"])
        .env("TELEGRAM_BOT_TOKEN", "test-token")
        .env_remove("RESEND_API_KEY")
        .assert()
        .failure()
        .stderr(predicate::str::contains("RESEND_API_KEY"));
}
