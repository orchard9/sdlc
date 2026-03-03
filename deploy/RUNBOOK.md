# Telegram Digest Bot — Operator Runbook

This runbook covers everything you need to deploy, operate, and maintain the Telegram Digest Bot on a VPS or Raspberry Pi.

---

## Prerequisites

Before you begin, ensure you have:

- **SSH access** to the target host (key-based auth strongly recommended)
- **Rust toolchain** installed locally (`rustup` — stable channel)
- **`cross` CLI** for cross-compilation (skip if deploying to amd64 natively):
  ```bash
  cargo install cross --git https://github.com/cross-rs/cross
  ```
  `cross` requires Docker to be running.
- **A Telegram bot token** from [@BotFather](https://t.me/BotFather)
- **SMTP credentials** (e.g., Gmail App Password) for sending digest emails
- **Target host** running Ubuntu 22.04 LTS, Debian 12, or Raspberry Pi OS (Bullseye/Bookworm)

---

## 1. Initial Host Setup

These steps run once on a fresh host. Run them as root or with sudo.

### 1a. Create the service user

```bash
# SSH to the host
ssh ubuntu@yourhost.example.com

# Create system user (no login shell, no home directory)
useradd --system --no-create-home --shell /usr/sbin/nologin telegram-bot
```

### 1b. Create the directory structure

```bash
mkdir -p /opt/telegram-bot/{bin,data}
chown -R telegram-bot:telegram-bot /opt/telegram-bot
chmod 750 /opt/telegram-bot
chmod 750 /opt/telegram-bot/bin /opt/telegram-bot/data
```

### 1c. Configure secrets

```bash
# Copy the env template from the repo (run on your local machine):
scp deploy/.env.example ubuntu@yourhost.example.com:/opt/telegram-bot/.env

# SSH to the host and fill in real values:
ssh ubuntu@yourhost.example.com
nano /opt/telegram-bot/.env

# Set secure permissions:
chmod 600 /opt/telegram-bot/.env
chown telegram-bot:telegram-bot /opt/telegram-bot/.env
```

Required variables in `/opt/telegram-bot/.env`:

| Variable | Description |
|---|---|
| `TELEGRAM_BOT_TOKEN` | Token from @BotFather |
| `DATABASE_URL` | `/opt/telegram-bot/data/messages.db` |
| `SMTP_HOST` | SMTP server (e.g., `smtp.gmail.com`) |
| `SMTP_PORT` | `587` (STARTTLS) or `465` (SSL) |
| `SMTP_USER` | Your email address |
| `SMTP_PASSWORD` | App password (not your main password) |
| `DIGEST_RECIPIENT` | Email to send the daily digest to |

---

## 2. First Deploy

From your local machine (repository root):

```bash
# Deploy to a VPS (amd64):
./deploy/deploy.sh --host yourhost.example.com --user ubuntu --arch amd64

# Deploy to Raspberry Pi 4 (arm64):
./deploy/deploy.sh --host raspberrypi.local --user pi --arch arm64

# Deploy to Raspberry Pi 3 (armv7):
./deploy/deploy.sh --host raspberrypi.local --user pi --arch armv7
```

The script will:
1. Build the `sdlc` binary for the target architecture
2. Upload it to the host
3. Install the systemd unit files
4. Enable and start both services

---

## 3. Verify Deployment

After the first deploy, confirm everything is running:

```bash
# SSH to the host
ssh ubuntu@yourhost.example.com

# Check the message store service is active and running:
systemctl status telegram-message-store

# Expected: Active: active (running)

# Check the digest timer is enabled:
systemctl list-timers --all | grep digest

# Expected: shows telegram-digest.timer with next trigger time

# Check recent logs:
journalctl -u telegram-message-store -n 20
```

### Send a test digest manually

```bash
# Trigger the digest right now (don't wait for the timer):
systemctl start telegram-digest.service

# Check the digest logs to confirm email was sent:
journalctl -u telegram-digest -n 30
```

---

## 4. Update (Deploy New Version)

Re-run the deploy script. It will build a new binary, upload it, and restart the service. The previous binary is saved as `sdlc.bak` for quick rollback.

```bash
./deploy/deploy.sh --host yourhost.example.com --user ubuntu --arch amd64
```

The message-store service will be briefly unavailable during the restart (typically under 2 seconds). The timer is not affected.

---

## 5. Restart Services

```bash
ssh ubuntu@yourhost.example.com

# Restart the message store (e.g., after config change):
systemctl restart telegram-message-store

# Reload the timer after editing the unit file:
systemctl daemon-reload
systemctl restart telegram-digest.timer
```

---

## 6. Check Logs

```bash
# Live tail of message store:
journalctl -u telegram-message-store -f

# Last 50 lines of digest sender:
journalctl -u telegram-digest -n 50

# All logs since a specific time:
journalctl -u telegram-message-store --since "2026-01-01 00:00:00"

# Errors only:
journalctl -u telegram-message-store -p err -n 20
```

---

## 7. Manual Digest Trigger

```bash
# Manually trigger the digest now:
systemctl start telegram-digest.service

# View what happened:
journalctl -u telegram-digest -n 50
```

---

## 8. Check Timer Status

```bash
# List all timers and when they fire next:
systemctl list-timers --all | grep digest

# Verify timer is enabled:
systemctl is-enabled telegram-digest.timer

# Expected output: enabled
```

---

## 9. Rollback

If a new binary causes issues, roll back to the previous version:

```bash
ssh ubuntu@yourhost.example.com

# Swap in the backup binary:
cp /opt/telegram-bot/bin/sdlc.bak /opt/telegram-bot/bin/sdlc

# Restart the service:
systemctl restart telegram-message-store

# Verify it's running:
systemctl status telegram-message-store
```

---

## 10. Troubleshooting

### Service fails to start

```bash
journalctl -u telegram-message-store -n 50 --no-pager
```

Common causes:
- `.env` file missing or has wrong permissions → `chmod 600 /opt/telegram-bot/.env`
- Invalid bot token → check `TELEGRAM_BOT_TOKEN` in `.env`
- Binary missing or wrong architecture → re-run `deploy.sh` with correct `--arch`

### Timer not firing

```bash
# Check the timer is enabled:
systemctl is-enabled telegram-digest.timer

# Check for timer errors:
journalctl -u telegram-digest.timer -n 20

# Re-enable if needed:
systemctl enable --now telegram-digest.timer
```

### Digest not sending email

```bash
# View digest logs:
journalctl -u telegram-digest -n 50

# Common causes:
# - Wrong SMTP credentials in .env
# - Gmail requires App Password (not account password)
# - SMTP_PORT mismatch (try 587 for STARTTLS, 465 for SSL)
# - DIGEST_RECIPIENT address invalid
```

### Check message count in database

```bash
# Quick count of stored messages:
sqlite3 /opt/telegram-bot/data/messages.db "SELECT COUNT(*) FROM messages;"

# View recent messages:
sqlite3 /opt/telegram-bot/data/messages.db "SELECT message_id, username, text, datetime(date, 'unixepoch') FROM messages ORDER BY date DESC LIMIT 10;"
```

### Services not starting after reboot

```bash
# Verify services are enabled:
systemctl is-enabled telegram-message-store
systemctl is-enabled telegram-digest.timer

# If not enabled:
systemctl enable telegram-message-store
systemctl enable telegram-digest.timer
```

---

## Architecture Reference

```
[telegram-message-store]  runs continuously, polls Telegram API
         ↓ writes
[/opt/telegram-bot/data/messages.db]  SQLite database
         ↑ reads
[telegram-digest]  runs once daily at 07:00, sends email
```

Both services run as the `telegram-bot` system user with no login shell or sudo access. Secrets are in `/opt/telegram-bot/.env` (mode 600).
