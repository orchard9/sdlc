# Tasks: telegram-bot-deploy

## Task Breakdown

---

### T1: Add deploy directory with systemd unit files

Create the `deploy/` directory in the project repository with all systemd service and timer unit files.

**Deliverables:**
- `deploy/telegram-message-store.service`
- `deploy/telegram-digest.service`
- `deploy/telegram-digest.timer`

**Acceptance:** All three unit files are syntactically correct (validate with `systemd-analyze verify` if available) and match the spec.

---

### T2: Create `.env.example` template

Create a `.env.example` file at `deploy/.env.example` (or project root) listing all required environment variables with placeholder values and inline comments.

**Deliverables:**
- `deploy/.env.example` with all variables from spec: `TELEGRAM_BOT_TOKEN`, `DATABASE_URL`, `SMTP_HOST`, `SMTP_PORT`, `SMTP_USER`, `SMTP_PASSWORD`, `DIGEST_RECIPIENT`

**Acceptance:** File is committed to git. No real secrets present. Comments explain each variable.

---

### T3: Write deploy.sh script

Create an idempotent `deploy/deploy.sh` shell script that:
1. Accepts `--host`, `--user`, `--arch` arguments
2. Runs `cross build --release --target <arch>-unknown-linux-musl` (or `cargo build` for native)
3. Rsyncs binaries to the host under `/opt/telegram-bot/bin/`
4. Copies systemd unit files to host `/etc/systemd/system/` via ssh
5. Runs `systemctl daemon-reload`, `systemctl enable --now` for service and timer
6. Prints service status

**Acceptance:** Script is executable (`chmod +x`), has a `--help` flag, and all steps are documented in comments. Idempotent: re-running on an already-deployed host does not produce errors.

---

### T4: Write RUNBOOK.md operator guide

Create `RUNBOOK.md` at the project root (or `deploy/RUNBOOK.md`) covering:
1. Prerequisites (SSH access, Rust toolchain, `cross` installed)
2. Initial host setup (create user, create directories, set permissions)
3. First deploy procedure (copy `.env`, run `deploy.sh`)
4. Verify deployment (check service status, check timer, send test digest)
5. Day-to-day operations (update, restart, check logs, manual digest trigger)
6. Rollback procedure (keep `.bak` binary, swap and restart)
7. Troubleshooting (common errors and fixes)

**Acceptance:** A new operator with SSH access to the host can deploy the bot in under 30 minutes following only this runbook.

---

### T5: Add cross-compilation targets to Cargo.toml / Makefile

Ensure the build configuration supports the three target architectures:
- `x86_64-unknown-linux-musl` (VPS)
- `aarch64-unknown-linux-musl` (RPi 4, 64-bit)
- `armv7-unknown-linux-musleabihf` (RPi 3, 32-bit)

Either add a `.cargo/config.toml` with target-specific linker config, or document the `cross` tool usage in `deploy.sh`.

**Acceptance:** `cross build --release --target aarch64-unknown-linux-musl` succeeds (or is clearly documented as the build command).

---

### T6: Add `.gitignore` entries for secrets

Ensure `.env` (and any variant like `.env.local`, `.env.production`) is in `.gitignore` at the project root.

**Deliverables:**
- `.gitignore` entry: `.env`
- Verify `.env.example` is NOT ignored (it must be committed)

**Acceptance:** `git status` does not show `.env` as an untracked file after creating one locally.

---

### T7: Validate unit files and smoke-test deploy script

Manual validation step:
1. Run `systemd-analyze verify deploy/telegram-message-store.service` (if on a Linux host)
2. Run `deploy.sh --help` to verify argument parsing
3. Dry-run rsync to confirm file list is correct

Document any issues found as comments in the source files.

**Acceptance:** No `systemd-analyze` errors. `deploy.sh --help` prints usage. Script structure confirmed correct.

---

## Task Order

```
T6 (gitignore) → parallel with all others (independent)
T1 (systemd units) → T7 (validate)
T2 (.env.example) → T4 (runbook references it)
T3 (deploy.sh) → T5 (cross-compile) → T7 (validate script)
T4 (runbook) — after T1, T2, T3 complete
```

## Estimated Effort

| Task | Effort |
|---|---|
| T1: systemd units | 30 min |
| T2: .env.example | 15 min |
| T3: deploy.sh | 1 hour |
| T4: RUNBOOK.md | 45 min |
| T5: cross-compile config | 30 min |
| T6: .gitignore | 5 min |
| T7: validate | 30 min |
| **Total** | **~3.5 hours** |
