#![allow(deprecated)]
/// Integration tests for `sdlc telegram` subcommands.
///
/// These tests verify the CLI's behavior at the boundary — particularly that
/// it fails gracefully with a clear error message when the bot token is absent.
/// No live Telegram network calls are made.
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

fn sdlc_with_home(project: &TempDir, home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("sdlc").unwrap();
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
