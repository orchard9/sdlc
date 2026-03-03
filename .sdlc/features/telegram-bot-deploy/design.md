# Design: telegram-bot-deploy

## Hosting Setup and Deployment (VPS or Raspberry Pi)

---

## 1. Deployment Architecture

The bot stack consists of two Rust binaries colocated on a single host, sharing a SQLite database on the local filesystem. No network communication between services is needed — they share state via the database file.

```
┌─────────────────────────────────────────────────────────────┐
│  Host (VPS: Ubuntu 22.04 / RPi: Raspberry Pi OS 64-bit)    │
│                                                             │
│  ┌─────────────────────────────────┐                        │
│  │  systemd service                │                        │
│  │  telegram-message-store         │  ← runs continuously  │
│  │  • long-polls Telegram API      │    restarts on crash   │
│  │  • writes to SQLite             │                        │
│  └──────────────┬──────────────────┘                        │
│                 │                                           │
│         /opt/telegram-bot/data/messages.db                  │
│                 │                                           │
│  ┌──────────────┴──────────────────┐                        │
│  │  systemd timer (daily 07:00)    │                        │
│  │  telegram-digest                │  ← fires once/day     │
│  │  • reads SQLite                 │    oneshot, exits     │
│  │  • sends email via SMTP         │                        │
│  └─────────────────────────────────┘                        │
│                                                             │
│  /opt/telegram-bot/                                         │
│  ├── bin/telegram-message-store                             │
│  ├── bin/telegram-digest-cron                               │
│  ├── data/messages.db                                       │
│  └── .env  (chmod 600)                                      │
└─────────────────────────────────────────────────────────────┘
          │
          ▼ outbound only
   Telegram Bot API (api.telegram.org)
   SMTP Server (smtp.gmail.com or similar)
```

---

## 2. Repository Structure Changes

Add the following files at the repository root:

```
telegram-digest-bot/          (project root or workspace crate)
├── deploy/
│   ├── deploy.sh             — one-shot deploy script
│   ├── telegram-message-store.service
│   ├── telegram-digest.service
│   ├── telegram-digest.timer
│   └── .env.example          — template with all required variables
├── RUNBOOK.md                — operator runbook
└── (existing source files)
```

---

## 3. Systemd Unit Design

### Service: `telegram-message-store`

Long-running daemon. Uses `Restart=on-failure` with a 5-second backoff to handle transient network errors or API rate limits.

Key design choices:
- `Type=simple` — binary exits only on error; systemd tracks PID directly
- `EnvironmentFile` — secrets injected from `/opt/telegram-bot/.env` at startup
- `StandardOutput=journal` — all stdout/stderr captured by journald, accessible via `journalctl`
- `RestartSec=5s` — avoids tight crash loops during Telegram API outages

### Timer: `telegram-digest.timer` + Service: `telegram-digest.service`

Systemd timer pair replaces cron. Timer triggers the oneshot service.

Key design choices:
- `OnCalendar=*-*-* 07:00:00` — fires daily at 07:00 local time
- `Persistent=true` — if host was offline at 07:00, fires once when it comes back online (catch-up)
- `Type=oneshot` — service exits after sending; timer re-fires next day
- No `Restart=` on the oneshot — if digest fails, it logs and the timer retries next day

---

## 4. User and Permissions Design

A dedicated system user `telegram-bot` owns all files and runs both services:

```bash
# Create system user (no home dir, no login shell)
useradd --system --no-create-home --shell /usr/sbin/nologin telegram-bot

# Directory ownership
chown -R telegram-bot:telegram-bot /opt/telegram-bot
chmod 750 /opt/telegram-bot
chmod 600 /opt/telegram-bot/.env
chmod 755 /opt/telegram-bot/bin/*
```

The `telegram-bot` user has no sudo access, no SSH login, and no write access outside `/opt/telegram-bot/`.

---

## 5. Deploy Script Design (`deploy.sh`)

The script is idempotent — running it multiple times is safe.

```
deploy.sh [--host HOST] [--user USER] [--arch amd64|arm64]

Steps:
1. cargo build --release --target <arch>-unknown-linux-musl
2. rsync binaries to HOST:/opt/telegram-bot/bin/
3. rsync systemd units to HOST:/etc/systemd/system/
4. ssh HOST: systemctl daemon-reload
5. ssh HOST: systemctl enable --now telegram-message-store
6. ssh HOST: systemctl enable --now telegram-digest.timer
7. ssh HOST: systemctl status telegram-message-store telegram-digest.timer
```

Musl static linking is used so the binaries run on any Linux without glibc version concerns. This is important for Raspberry Pi (musl armv7 or aarch64 target).

---

## 6. Environment File Design

`.env.example` (committed to git — no real secrets):

```bash
# Telegram Bot Token from @BotFather
TELEGRAM_BOT_TOKEN=your_bot_token_here

# SQLite database path
DATABASE_URL=/opt/telegram-bot/data/messages.db

# SMTP configuration
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USER=your_email@gmail.com
SMTP_PASSWORD=your_app_password_here

# Digest recipient
DIGEST_RECIPIENT=recipient@example.com
```

The live `.env` file on the host is never committed to git. Initial setup: operator copies `.env.example` to `/opt/telegram-bot/.env` and fills in real values.

---

## 7. Cross-Compilation Strategy

| Host Architecture | Rust Target | Notes |
|---|---|---|
| VPS (x86_64) | `x86_64-unknown-linux-musl` | Standard; musl-cross toolchain |
| RPi 4 (aarch64) | `aarch64-unknown-linux-musl` | Raspberry Pi OS 64-bit |
| RPi 3 (armv7) | `armv7-unknown-linux-musleabihf` | Raspberry Pi OS 32-bit |

Local cross-compilation uses `cross` (Docker-based) or `cargo-zigbuild` for simplicity:
```bash
cross build --release --target aarch64-unknown-linux-musl
```

---

## 8. RUNBOOK.md Outline

Operator runbook covers:

1. **Initial Setup** — provision host, create user, install binaries, configure `.env`
2. **First Deploy** — run `deploy.sh`, verify services, send test digest
3. **Update** — `deploy.sh` reruns build + rsync + restart
4. **Check Logs** — `journalctl -u telegram-message-store -f`
5. **Manual Digest** — `systemctl start telegram-digest.service`
6. **Restart Services** — `systemctl restart telegram-message-store`
7. **Rollback** — keep previous binary in `bin/telegram-message-store.bak`, swap and restart
8. **Verify Timer** — `systemctl list-timers --all | grep digest`

---

## 9. Design Decisions and Trade-offs

| Decision | Rationale |
|---|---|
| systemd over Docker | Lower overhead, native on both VPS and RPi, no Docker daemon required |
| systemd timer over crontab | Better logging, catch-up on missed runs, integrated with journald |
| Musl static linking | No glibc version dependency — same binary works on old/new distros and RPi |
| `/opt/telegram-bot/` prefix | Standard Linux convention for self-contained third-party apps |
| Long-polling over webhooks | No inbound port required; simpler firewall config; fine for single-bot use |
| SQLite on shared filesystem | No network DB needed; both services colocated; no connection pooling issues |

---

## 10. Acceptance Criteria Mapping

| Spec AC | Implementation |
|---|---|
| Services start on boot | `systemctl enable` in deploy.sh |
| Restart on crash within 10s | `Restart=on-failure; RestartSec=5s` |
| Timer fires at configured time | `OnCalendar=*-*-* 07:00:00; Persistent=true` |
| Secrets not in git | `.env.example` in git, `.env` on host only |
| Deploy in under 30 min | `deploy.sh` + `RUNBOOK.md` |
| Logs via journalctl | `StandardOutput=journal` in service units |
