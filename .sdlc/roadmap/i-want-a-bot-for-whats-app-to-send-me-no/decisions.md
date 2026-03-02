# Decision Log

## ⚑  Decided: outbound webhook is the right primitive

The sdlc-server should emit an outbound webhook on UAT completion — not a WhatsApp-specific integration. Reasoning:

- Keeps sdlc-server free of messaging-platform dependencies (no Twilio SDK, no API keys)
- Same primitive enables Slack, Discord, PagerDuty, email — one feature covers all Jordan's future notification needs
- The WhatsApp bot is a 50-line script Jordan can run anywhere (Raspberry Pi, Fly.io, local machine)
- Dana Cho: 'One generic outbound hook > one per-platform integration forever'

## ⚑  Decided: standalone bot, not embedded in sdlc-server

The WhatsApp bot is a separate process that receives the webhook and calls Twilio. It is NOT a new crate in this monorepo. Reasoning:

- Credentials (Twilio SID/token, phone numbers) must not be in sdlc config
- The bot may run on a different machine (phone number must be reachable by WhatsApp/Twilio servers)
- Deployment flexibility: can run on Fly.io free tier, Raspberry Pi, or a VPS

## ⚑  Decided: Twilio WhatsApp API is the path

- **Dev**: Twilio Sandbox — no Meta Business approval, 5-minute setup, receives text + media
- **Production**: Requires Meta Business account + approved message templates
- Alternative considered: whatsapp-web.js (personal account, unofficial) — rejected (TOS risk, session fragility)
- Alternative considered: CallMeBot — limited media support

## ?  Open: What if the tunnel isn't running?

Two options:
1. Send text-only notification (degrade gracefully, omit artifact_urls)
2. Don't send notification if no tunnel (artifacts aren't reachable anyway)

**Lean toward option 1**: A text notification with pass/fail verdict + report URL (even if non-public) is still useful.

## ?  Open: Should sdlc-cli offer a ready-made bot script?

`sdlc notify setup-whatsapp` could scaffold a bot.py + instructions. Or just document it in a guide. Low priority — Jordan can write 50 lines.

## ?  Open: Video delivery

WhatsApp supports video up to 16MB. Playwright webm files may exceed this. Options:
1. Skip video in WhatsApp notification, link to report URL instead
2. Transcode to compressed mp4 (adds ffmpeg dependency)
3. Upload to temporary link service

**Lean toward option 1 for initial scope**: screenshots cover 90% of the value; video via browser link is sufficient.