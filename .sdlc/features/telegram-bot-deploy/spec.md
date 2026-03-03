# Spec: telegram-bot-deploy

## Feature Title
Hosting setup and deployment (VPS or Raspberry Pi)

## Overview

This feature covers the infrastructure and deployment configuration required to run the Telegram Daily Digest Bot on a self-hosted environment — either a VPS (Virtual Private Server) or a Raspberry Pi. The bot comprises two components: a message-store service (polling Telegram and persisting to SQLite) and a cron-triggered digest sender (query + SMTP email). This spec defines the deployment topology, service management, environment configuration, and operational runbook so the bot runs reliably 24/7.

## Problem Statement

The Telegram bot services (message-store and digest-cron) need a persistent, always-on execution environment. A developer laptop or ephemeral cloud function is insufficient — the polling loop must run continuously and the cron job must fire on schedule every day. The deployment must be simple enough for a single operator to set up, maintain, and recover from failures.

## Goals

- Deploy both bot services (message-store and digest-cron) to a single host
- Services start automatically on boot and restart on failure
- Environment secrets (Telegram bot token, SMTP credentials) are stored safely and injected at runtime
- Deployment is reproducible from a shell script or Ansible playbook
- Operator runbook covers: initial deploy, update, restart, log inspection, and rollback

## Non-Goals

- Container orchestration (Kubernetes, Docker Swarm) — out of scope for this milestone
- Multi-host high availability
- Automatic TLS certificate management (no inbound HTTP required for bot)
- CI/CD pipeline integration (out of scope for this milestone; manual deploy is sufficient)

## Target Environments

### Option A: VPS (Recommended for reliability)
- Ubuntu 22.04 LTS or Debian 12
- Minimum specs: 1 vCPU, 512 MB RAM, 5 GB disk
- Providers: Hetzner, DigitalOcean, Linode, Vultr

### Option B: Raspberry Pi
- Raspberry Pi 3B+ or newer
- Raspberry Pi OS (Bullseye/Bookworm, 64-bit)
- Requires stable internet connection

## Architecture

```
[Host: VPS or Raspberry Pi]
  ├── systemd service: telegram-message-store
  │     runs: ./telegram-message-store
  │     restarts: on-failure, delay 5s
  │     env: .env file (Telegram token, DB path)
  │
  ├── systemd service: telegram-digest-cron
  │     runs: via systemd timer (daily at 07:00 local time)
  │     env: .env file (SMTP creds, DB path, recipient email)
  │
  └── SQLite database: /opt/telegram-bot/data/messages.db
        shared between both services (same host, same filesystem)
```

## Deployment Layout

```
/opt/telegram-bot/
  ├── bin/
  │   ├── telegram-message-store   (compiled binary)
  │   └── telegram-digest-cron     (compiled binary or script)
  ├── data/
  │   └── messages.db              (SQLite, created on first run)
  ├── .env                         (secrets, chmod 600, owned by bot user)
  └── logs/                        (journald via systemd, no separate log dir needed)
```

## Environment Variables

| Variable | Description | Example |
|---|---|---|
| `TELEGRAM_BOT_TOKEN` | Telegram bot API token from @BotFather | `123456:ABC-DEF...` |
| `DATABASE_URL` | Path to SQLite database | `/opt/telegram-bot/data/messages.db` |
| `SMTP_HOST` | SMTP server hostname | `smtp.gmail.com` |
| `SMTP_PORT` | SMTP port (465=TLS, 587=STARTTLS) | `587` |
| `SMTP_USER` | SMTP login username | `user@gmail.com` |
| `SMTP_PASSWORD` | SMTP password or app password | `secret` |
| `DIGEST_RECIPIENT` | Email address to send digest to | `owner@example.com` |
| `DIGEST_TIME` | Cron schedule for digest (systemd timer format) | `07:00` |

## Systemd Units

### telegram-message-store.service
```ini
[Unit]
Description=Telegram Message Store — polling and SQLite persistence
After=network.target

[Service]
Type=simple
User=telegram-bot
EnvironmentFile=/opt/telegram-bot/.env
ExecStart=/opt/telegram-bot/bin/telegram-message-store
Restart=on-failure
RestartSec=5s
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
```

### telegram-digest.service (oneshot)
```ini
[Unit]
Description=Telegram Digest Sender — daily email digest

[Service]
Type=oneshot
User=telegram-bot
EnvironmentFile=/opt/telegram-bot/.env
ExecStart=/opt/telegram-bot/bin/telegram-digest-cron
StandardOutput=journal
StandardError=journal
```

### telegram-digest.timer
```ini
[Unit]
Description=Daily Telegram Digest Timer

[Timer]
OnCalendar=*-*-* 07:00:00
Persistent=true

[Install]
WantedBy=timers.target
```

## Deployment Script

A `deploy.sh` script at repository root handles:
1. Cross-compiling binaries for target architecture (amd64 or arm64)
2. Copying binaries to host via `rsync`
3. Installing systemd units
4. Reloading systemd and restarting services
5. Verifying service status

## Security Considerations

- Dedicated `telegram-bot` system user (no login shell, no sudo)
- `.env` file owned by `telegram-bot`, permissions `600`
- No inbound ports opened (bot uses Telegram long-polling, not webhooks)
- SQLite database owned by `telegram-bot`, not world-readable

## Acceptance Criteria

1. Both services start automatically after `systemctl enable` and host reboot
2. `telegram-message-store` restarts within 10 seconds of a crash
3. `telegram-digest.timer` fires at the configured time and sends the email
4. Secrets are not stored in code, git, or world-readable files
5. A new operator can deploy from scratch following the runbook in under 30 minutes
6. Logs are accessible via `journalctl -u telegram-message-store`

## Out of Scope

- Docker containerization (can be added later)
- Monitoring/alerting (Prometheus, Grafana)
- Webhook-based Telegram updates (long-polling is simpler for this use case)
- Multi-recipient digest distribution lists
