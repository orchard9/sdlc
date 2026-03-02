## Architecture

**What it does**: Telegram bot collects messages from group chats, sends a daily email digest of active groups.

**Stack**:
- Python (python-telegram-bot or aiogram) + SQLite
- BotFather-registered bot added manually to each group
- Cron job daily at fixed time (e.g. 8am)
- Any SMTP sender (Gmail app password, SendGrid, Resend)

**Flow**:
1. Bot receives messages via polling/webhook → store in SQLite (group_id, sender, text, timestamp)
2. Cron at 8am: query last 24h per group
3. Groups with ≥1 message → included in digest email
4. Groups with 0 messages → omitted (no noise)
5. Single email sent: group name + message count + last-active time

**Email format (v1 — keep it simple)**:
```
Daily Telegram Digest — {date}
───────────────────────────────
{Group Name} — {N} messages, last active {time}
{Group Name} — {N} messages, last active {time}
───────────────────────────────
{N} groups were silent today.
```

**Constraints**:
- Bot must be added manually to each group (no MTProto/userbot — ToS risk)
- 24h window = fixed daily digest, not rolling per-message delay
- Private groups only (Jordan controls which groups get the bot)

**Hosting**: VPS or Raspberry Pi. ~200 lines of Python. One-time setup.