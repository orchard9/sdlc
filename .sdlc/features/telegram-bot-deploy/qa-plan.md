# QA Plan: telegram-bot-deploy

## Feature: Hosting Setup and Deployment (VPS or Raspberry Pi)

---

## Test Strategy

This is a deployment/infrastructure feature. QA focuses on:
1. **Artifact completeness** — all required files exist and are correct
2. **Script correctness** — deploy.sh logic, argument handling, idempotency
3. **Unit file validity** — systemd unit syntax and semantics
4. **Security compliance** — secrets handling, file permissions
5. **Documentation quality** — runbook completeness and accuracy

Actual end-to-end deployment to a live host is verified via manual smoke test (TC-5).

---

## Test Cases

### TC-1: Systemd Unit File Syntax

**Objective:** Verify all three systemd unit files are syntactically valid.

**Method:** Static analysis / manual review

**Steps:**
1. Open `deploy/telegram-message-store.service`
2. Verify `[Unit]`, `[Service]`, `[Install]` sections present
3. Verify `Type=simple`, `Restart=on-failure`, `RestartSec=5s`, `EnvironmentFile=`, `ExecStart=` set correctly
4. Open `deploy/telegram-digest.service`
5. Verify `Type=oneshot`, no `Restart=`, `EnvironmentFile=` set
6. Open `deploy/telegram-digest.timer`
7. Verify `OnCalendar=*-*-* 07:00:00`, `Persistent=true`, `[Install]` with `WantedBy=timers.target`

**Pass criteria:** All sections and directives match spec. No missing required fields.

---

### TC-2: `.env.example` Completeness

**Objective:** Verify the env template covers all required variables.

**Steps:**
1. Open `deploy/.env.example`
2. Confirm presence of all 7 variables: `TELEGRAM_BOT_TOKEN`, `DATABASE_URL`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`, `DIGEST_RECIPIENT`
3. Confirm all values are placeholder strings (not real credentials)
4. Confirm each variable has an inline comment explaining its purpose

**Pass criteria:** All 7 variables present, no real secrets, all commented.

---

### TC-3: `.gitignore` Secret Exclusion

**Objective:** Verify `.env` is excluded from git tracking.

**Steps:**
1. Check `.gitignore` for `.env` entry
2. Create a `/tmp/test.env` file locally, copy to project root as `.env`
3. Run `git status` — verify `.env` does NOT appear as untracked
4. Run `git status` — verify `.env.example` DOES appear (not ignored)
5. Remove the test `.env` file

**Pass criteria:** `.env` excluded, `.env.example` tracked.

---

### TC-4: deploy.sh Argument Parsing and Help

**Objective:** Verify the deploy script handles arguments correctly.

**Steps:**
1. Run `bash deploy/deploy.sh --help` — verify usage text is printed, script exits 0
2. Run `bash deploy/deploy.sh` with no arguments — verify it either prompts for required args or exits with a clear error message
3. Run `bash -n deploy/deploy.sh` — verify no syntax errors (bash dry-run)
4. Check that `--host`, `--user`, `--arch` flags are all handled

**Pass criteria:** `--help` works, no syntax errors, required-arg validation present.

---

### TC-5: Manual Smoke Test Deploy (End-to-End)

**Objective:** Verify complete deploy to a live host works end-to-end.

**Prerequisites:**
- SSH access to a VPS or Raspberry Pi running Ubuntu 22.04 / Raspberry Pi OS
- Rust + `cross` installed on the build machine
- `.env` file configured with real credentials on the target host

**Steps:**
1. Run `deploy/deploy.sh --host <hostname> --user <user> --arch amd64`
2. SSH to host, run `systemctl status telegram-message-store`
3. Verify service is `active (running)`
4. Run `systemctl list-timers --all | grep digest` — verify timer is enabled
5. Run `systemctl start telegram-digest.service` (manual trigger)
6. Check `journalctl -u telegram-digest.service` — verify email was sent (or SMTP connection logged)
7. Reboot host, wait 60 seconds, SSH back
8. Verify `telegram-message-store` is `active (running)` without manual start
9. Verify timer is still listed in `systemctl list-timers`

**Pass criteria:** Service runs after reboot, timer active, manual digest trigger works.

---

### TC-6: File Permissions Security Check

**Objective:** Verify the deployment enforces correct file permissions for secrets.

**Steps:**
1. After TC-5 deploy, SSH to host
2. Run `stat /opt/telegram-bot/.env` — verify permissions are `600`
3. Run `stat /opt/telegram-bot/.env` — verify owner is `telegram-bot:telegram-bot`
4. Verify `telegram-bot` user has no login shell: `grep telegram-bot /etc/passwd`
5. Verify `telegram-bot` has no sudo entry: `sudo -l -U telegram-bot`

**Pass criteria:** `.env` is `600`, owned by `telegram-bot`, user has `nologin` shell, no sudo.

---

### TC-7: RUNBOOK.md Completeness

**Objective:** Verify the operator runbook covers all required sections.

**Steps:**
1. Open `RUNBOOK.md`
2. Confirm presence of sections: Prerequisites, Initial Setup, First Deploy, Verify Deployment, Update, Restart, Check Logs, Manual Digest, Rollback, Troubleshooting
3. Verify "Initial Setup" covers: user creation, directory creation, permissions
4. Verify "Check Logs" shows exact `journalctl` command
5. Verify "Rollback" describes keeping a `.bak` binary and swapping it

**Pass criteria:** All 10 sections present, commands are accurate and copy-pasteable.

---

### TC-8: Idempotent Deploy

**Objective:** Verify running deploy.sh twice on an already-deployed host does not produce errors.

**Prerequisites:** TC-5 completed (host already deployed)

**Steps:**
1. Run `deploy/deploy.sh --host <hostname> --user <user> --arch amd64` again
2. Observe output — no errors or warnings about "already exists"
3. Run `systemctl status telegram-message-store` — service still active

**Pass criteria:** Second deploy exits 0, service remains running.

---

## Pass/Fail Criteria Summary

| Test Case | Type | Weight |
|---|---|---|
| TC-1: Systemd unit syntax | Static review | Required |
| TC-2: .env.example completeness | Static review | Required |
| TC-3: .gitignore secret exclusion | Automated check | Required |
| TC-4: deploy.sh argument handling | Static + run | Required |
| TC-5: End-to-end deploy smoke test | Manual | Required |
| TC-6: File permissions security | Manual (post-deploy) | Required |
| TC-7: RUNBOOK.md completeness | Static review | Required |
| TC-8: Idempotent deploy | Manual | Required |

**All test cases are required to pass for QA approval.**

---

## Environment Requirements

| Requirement | Details |
|---|---|
| Build machine | Rust stable, `cross` CLI, Docker (for cross-compilation) |
| Target host | Ubuntu 22.04 LTS or Raspberry Pi OS (Bullseye/Bookworm 64-bit) |
| Network | Build machine has SSH access to target host |
| Credentials | Real Telegram bot token and SMTP credentials for TC-5 |
