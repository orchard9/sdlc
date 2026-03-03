# Audit: telegram-bot-deploy

## Feature: Hosting Setup and Deployment (VPS or Raspberry Pi)

---

## Audit Scope

This audit covers the security, operational, and correctness properties of the deployment package:
- Systemd unit files (`telegram-message-store.service`, `telegram-digest.service`, `telegram-digest.timer`)
- Environment template (`deploy/.env.example`)
- Deploy script (`deploy/deploy.sh`)
- Operator runbook (`deploy/RUNBOOK.md`)
- Cross-compilation config (`.cargo/config.toml`)
- `.gitignore` secret exclusion

---

## Security Audit

### A1: Secrets Handling

**Finding:** The `.env.example` file contains no real credentials — all values are placeholder strings.

**Evidence:**
- `TELEGRAM_BOT_TOKEN=your_bot_token_here`
- `SMTP_PASSWORD=your_app_password_here`
- Programmatic check confirmed no real token patterns (regex `[0-9]+:[A-Za-z0-9_-]{35,}`) found.

**`.gitignore` coverage:**
```
.env
.env.*
!.env.example
```
This excludes all `.env` variants while keeping the template tracked.

**Verdict:** PASS — no secrets are present in version-controlled files.

---

### A2: Service User Isolation

**Finding:** The service units run as `telegram-bot` system user — no login shell, no sudo.

**Evidence from unit files:**
```ini
User=telegram-bot
Group=telegram-bot
NoNewPrivileges=true
```

**Evidence from deploy.sh:**
```bash
useradd --system --no-create-home --shell /usr/sbin/nologin telegram-bot
```

The user cannot:
- Log in via SSH
- Escalate to root via sudo
- Access other users' home directories

**Verdict:** PASS — principle of least privilege applied correctly.

---

### A3: File System Permissions

**Finding:** Service units include systemd-level filesystem restrictions.

```ini
ProtectSystem=strict       # Root and /usr are read-only
PrivateTmp=true            # Isolated /tmp namespace
ReadWritePaths=/opt/telegram-bot/data  # Only data dir is writable
NoNewPrivileges=true       # No setuid/setgid escalation
```

**`.env` file permissions** set by deploy.sh:
- Not set by the script — documented as operator responsibility in RUNBOOK.md §1c
- **Finding (minor):** deploy.sh does not enforce `.env` permissions (600) after copying. This is left to the operator.

**Action:** Accept with task — add a check/warn in `deploy.sh` for `.env` permissions on the host. Tracked as future improvement.

**Verdict:** MOSTLY PASS — filesystem restrictions are strong; `.env` permission enforcement could be improved.

---

### A4: Network Exposure

**Finding:** The bot uses long-polling (outbound only). No inbound ports are opened.

**Evidence from spec:** "No inbound ports opened (bot uses Telegram long-polling, not webhooks)"

**Evidence from unit files:** No `BindToDevice`, `ListenStream`, or socket-activation directives. The service makes outbound connections to `api.telegram.org` and the SMTP host only.

**Verdict:** PASS — minimal attack surface; no inbound exposure.

---

### A5: Supply Chain — Build Process

**Finding:** The deploy script builds from local source using `cross` or native `cargo build`. No downloading of pre-built binaries from external sources.

**Verdict:** PASS — build-from-source ensures binary integrity.

---

### A6: Credential Rotation

**Finding:** The `.env` file is on the host filesystem, not in a secrets manager. Rotation requires SSH access to the host + service restart.

**No key rotation automation** is provided. This is acceptable for the current scale (single bot, single operator) but would be a gap if scaled to multiple hosts.

**Action:** Accept — tracked as a future concern. For this milestone (single-host deployment), manual rotation is adequate. Add a RUNBOOK.md note about credential rotation.

**Verdict:** ACCEPTABLE — no action required for current scope.

---

## Operational Audit

### A7: Service Recovery

**Finding:** `telegram-message-store` has `Restart=on-failure; RestartSec=5s`.

**Analysis:**
- A crash loop (repeated failures) will cause increasing restart attempts at 5s intervals. systemd will eventually hit its restart limit (`StartLimitIntervalSec` / `StartLimitBurst` defaults). Default is 5 restarts per 10 seconds before entering failed state.
- For temporary Telegram API outages, the 5-second backoff is appropriate.
- For permanent failures (bad token, database corrupt), the service will enter `failed` state and require manual intervention.

**Verdict:** PASS — behavior is correct. Operator is alerted via `journalctl` logs when the service enters failed state.

---

### A8: Timer Persistence

**Finding:** `Persistent=true` on `telegram-digest.timer` ensures the digest fires once if the host was offline at 07:00.

**Risk:** If the host is offline for multiple days, only one catch-up digest fires when it comes back online (not one per missed day). This is acceptable behavior for a daily digest bot.

**Verdict:** PASS — no unexpected data loss; behavior documented.

---

### A9: Binary Rollback

**Finding:** `deploy.sh` saves the previous binary as `sdlc.bak` before each deploy. Rollback is a manual two-command operation (copy `.bak`, restart service).

**Risk:** Only one generation of backup is kept. Two consecutive bad deploys would leave no rollback option.

**Action:** Accept — for a single-operator bot, one generation of rollback is adequate. This is documented in RUNBOOK.md §9.

---

## Correctness Audit

### A10: Unit File Syntax (Static Analysis)

All three unit files validated against required sections and directives:
- `telegram-message-store.service`: `[Unit]` ✓, `[Service]` ✓, `[Install]` ✓, required directives ✓
- `telegram-digest.service`: `[Unit]` ✓, `[Service]` ✓, required directives ✓
- `telegram-digest.timer`: `[Unit]` ✓, `[Timer]` ✓, `[Install]` ✓, `OnCalendar` ✓, `Persistent` ✓

**Verdict:** PASS

---

### A11: Deploy Script Correctness

- `bash -n` syntax check: PASS
- `--help` flag: PASS
- Missing `--host` error: PASS (clear message + exit 1)
- Unknown flag handling: PASS
- Dry-run mode: PASS (complete command preview)
- Architecture validation: PASS (rejects invalid `--arch` values)
- Cross-compilation fallback chain: PASS

**Verdict:** PASS

---

## Findings Summary

| Finding | Severity | Action |
|---|---|---|
| A1: Secrets handling | — | PASS |
| A2: Service user isolation | — | PASS |
| A3: .env permission enforcement | Low | Future task: add chmod enforcement in deploy.sh |
| A4: Network exposure | — | PASS |
| A5: Supply chain | — | PASS |
| A6: Credential rotation | Low | Accept — documented for future multi-host expansion |
| A7: Service recovery | — | PASS |
| A8: Timer persistence | — | PASS |
| A9: Rollback coverage | Low | Accept — one generation adequate for single-operator |
| A10: Unit file syntax | — | PASS |
| A11: Deploy script correctness | — | PASS |

**Total findings:** 3 low-severity, all accepted with rationale or tracked for future improvement.

---

## Audit Verdict

**APPROVED**

The deployment package is production-quality for a single-operator self-hosted bot. Security surface is minimal (no inbound ports, dedicated system user, filesystem restrictions, secrets excluded from git). All acceptance criteria from the spec are met. The three low-severity findings are accepted with documented rationale — none are blockers for this milestone.

---

## Tasks Added for Future Improvement

1. Add `.env` permission check in `deploy.sh` (chmod 600 verification/enforcement after copy)
2. Document credential rotation procedure in RUNBOOK.md
