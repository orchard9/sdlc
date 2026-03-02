# Architecture: UAT → WhatsApp Notification

## The pieces

```
sdlc-server                    WhatsApp Bot (standalone)
─────────────                  ──────────────────────────
UAT run completes              receives POST
  → MilestoneUatCompleted      extracts artifact_urls
  → fire outbound webhook  →   fetches screenshots from tunnel URL
     payload: {                sends WhatsApp via Twilio API
       event, milestone,         - text: verdict + summary
       run_id, status,           - images: screenshots (inline)
       tunnel_url,               - video link (if present)
       artifact_urls,          }
       report_url
     }
```

## What sdlc-server needs (new primitive)

**Outbound webhook on UAT completion** — when `MilestoneUatCompleted` fires:
1. Read configured webhook URLs from `.sdlc/config.yaml` (new field: `notify_webhooks: ["https://..."]}`)
2. Build payload (see below)
3. POST to each URL (non-blocking, best-effort — failure must not block the run)

```yaml
# .sdlc/config.yaml (new optional field)
notify_webhooks:
  - https://my-bot.example.com/hooks/uat
```

## Payload shape

```json
{
  "event": "uat_completed",
  "milestone": "v12-feature",
  "run_id": "20260302-180513-ouu",
  "status": "passed",
  "verdict": "All 8 checks passed",
  "tunnel_url": "https://xyz.trycloudflare.com",
  "artifact_urls": [
    "https://xyz.trycloudflare.com/api/milestones/v12/uat-runs/20260302-ouu/artifacts/screenshots/01-login.png"
  ],
  "report_url": "https://xyz.trycloudflare.com/milestones/v12"
}
```

## WhatsApp bot (standalone ~50 lines)

**Approach:** Twilio WhatsApp API (sandbox for dev, production needs Meta approval)

```python
# bot.py — FastAPI endpoint that receives sdlc webhook
from fastapi import FastAPI, Request
from twilio.rest import Client
import httpx, os

TWILIO_SID = os.environ['TWILIO_SID']
TWILIO_TOKEN = os.environ['TWILIO_TOKEN']
WHATSAPP_FROM = 'whatsapp:+14155238886'  # Twilio sandbox
WHATSAPP_TO = os.environ['WHATSAPP_TO']  # your number

app = FastAPI()

@app.post('/hooks/uat')
async def handle_uat(req: Request):
    data = await req.json()
    status_emoji = '✅' if data['status'] == 'passed' else '❌'
    body = f"{status_emoji} UAT {data['status'].upper()}: {data['milestone']}\n{data['verdict']}\nReport: {data['report_url']}"
    
    client = Client(TWILIO_SID, TWILIO_TOKEN)
    # Send text message
    client.messages.create(body=body, from_=WHATSAPP_FROM, to=WHATSAPP_TO)
    # Send screenshots (up to 3)
    for url in data.get('artifact_urls', [])[:3]:
        client.messages.create(media_url=[url], from_=WHATSAPP_FROM, to=WHATSAPP_TO)
    
    return {'ok': True}
```

## Dependencies

1. **Tunnel must be active** — artifact URLs are only routable via cloudflare tunnel; notification is text-only if no tunnel
2. **uat-artifacts-storage** (v19 milestone) — screenshots must be stored and served; otherwise artifact_urls is empty
3. **Twilio sandbox** — free, no Meta approval needed for dev; production requires Meta Business approval

## Key constraint: Media requires public URLs

WhatsApp API (both Twilio and Meta) fetches media at send time. The artifact URLs must be publicly reachable. The cloudflare tunnel solves this IF it's running. If no tunnel: send text-only notification (still useful).

## What does NOT belong in sdlc-server

- Twilio credentials / WhatsApp API calls — these are the bot's concern
- Message formatting — the bot owns this
- Media re-hosting — Twilio handles this by fetching the URL

The server only needs to fire the webhook.