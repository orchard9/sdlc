---
session: 1
timestamp: 2026-03-02T18:30:00Z
orientation:
  current: "Architecture is clear. sdlc-server needs one new primitive (outbound webhook on UAT completion). WhatsApp bot is a standalone script consuming that webhook via Twilio. Depends on uat-artifacts-storage (v19) for media and cloudflare tunnel for public artifact URLs."
  next: "Spawn two features: (1) outbound-uat-webhook — add notify_webhooks config + POST on MilestoneUatCompleted; (2) whatsapp-notify-bot — scaffold the Twilio consumer script + setup guide."
  commit: "Scope is clear, dependencies mapped, architecture decided. Commit signal met."
---

**jordan · Owner**
i want a bot for whats app to send me notifications when a UAT is done and send me any videos and screenshots

---

## Session 1 — Interrogating the Brief

**Facilitator:** First session — no prior context. Let me bring in some thought partners to pressure-test this.

---

### Recruiting: Dana Cho (Product Skeptic)

**Dana:** Before we design anything — what's the actual pain? The web UI has SSE-driven live updates, the UAT history panel shows all runs. Is the problem that you're not at your computer when UAT finishes? Or that you have to actively check?

**Jordan (inferred):** I want to know on my phone, passively, without opening a browser. And I want the screenshots so I can see what passed or failed right there in WhatsApp.

**Dana:** OK, so this is a *mobile push notification* problem with *media delivery*. WhatsApp is the channel because it's already open, has good media rendering, and notifications are reliable. That's a legitimate reason — not just novelty. But let's flag: Twilio WhatsApp sandbox is easy dev path; production needs Meta Business approval (~1 week). Make sure Jordan knows that before committing.

⚑  Decided: The real need is mobile passive notification with media delivery. WhatsApp is the right channel for Jordan's specific context.

---

### Recruiting: Priya Nair (Distributed Systems)

**Priya:** I looked at the codebase. Here's what exists:

1. `SseMessage::MilestoneUatCompleted { slug }` — fires in `spawn_agent_run` when UAT run finishes
2. `AppState.tunnel_url: Arc<RwLock<Option<String>>>` — tunnel URL is in server state, accessible at notification time
3. `uat-artifacts-storage` (v19 milestone) — will store screenshots at `<run_dir>/screenshots/*.png` with a binary serving route

The **missing primitive** is outbound webhook egress on `MilestoneUatCompleted`. The server already receives inbound webhooks (`POST /webhooks/{route}`) but has no mechanism to *fire* them.

**Dana:** And that's exactly the right primitive — not a WhatsApp-specific integration. If Jordan later wants Slack, Discord, or SMS, the same outbound webhook powers all of them.

⚑  Decided: sdlc-server gets a generic outbound webhook mechanism. No WhatsApp SDK, no Twilio dependencies in the server.

---

### What sdlc-server needs

One new config field:
```yaml
# .sdlc/config.yaml
notify_webhooks:
  - https://my-bot.example.com/hooks/uat
```

When `MilestoneUatCompleted` fires, POST this payload to each URL (non-blocking, best-effort — failure must not block anything):

```json
{
  "event": "uat_completed",
  "milestone": "v12-feature",
  "run_id": "20260302-180513-ouu",
  "status": "passed",
  "verdict": "All 8 checks passed",
  "tunnel_url": "https://xyz.trycloudflare.com",
  "artifact_urls": [
    "https://xyz.trycloudflare.com/api/milestones/v12/uat-runs/ouu/artifacts/screenshots/01-login.png"
  ],
  "report_url": "https://xyz.trycloudflare.com/milestones/v12"
}
```

If no tunnel is active: `tunnel_url` is null, `artifact_urls` is empty. Still send — text notification is useful even without media.

---

### The WhatsApp Bot

**Priya:** The bot is not a sdlc crate — it's a standalone ~50-line script Jordan deploys wherever his phone can reach it. Fly.io free tier, Raspberry Pi, anything.

```python
# bot.py — FastAPI endpoint that receives sdlc outbound webhook
@app.post('/hooks/uat')
async def handle_uat(req: Request):
    data = await req.json()
    status_emoji = '✅' if data['status'] == 'passed' else '❌'
    body = f"{status_emoji} UAT {data['status'].upper()}: {data['milestone']}\n{data['verdict']}\nReport: {data['report_url']}"

    client = Client(TWILIO_SID, TWILIO_TOKEN)
    client.messages.create(body=body, from_=WHATSAPP_FROM, to=WHATSAPP_TO)

    for url in data.get('artifact_urls', [])[:3]:
        client.messages.create(media_url=[url], from_=WHATSAPP_FROM, to=WHATSAPP_TO)

    return {'ok': True}
```

**Dana:** Does sdlc provide a setup guide or scaffold for this bot? Or just document "here's the webhook payload, write your own bot"?

**Facilitator:** For the initial feature, document the payload and provide a reference implementation in the feature's `summary.md`. No new sdlc commands needed — this is integration code Jordan owns.

---

### Recruiting: Ben Hartley (Developer Productivity UX)

**Ben:** The notification UX matters. When UAT finishes, Jordan wants to glance at his phone and know three things instantly:

1. **Pass or fail** — emoji + bold status, first line
2. **What milestone** — so he knows which work finished
3. **Screenshots** — visual confirmation without clicking into a browser

WhatsApp's media grouping is great for this: text message first, then 2-3 screenshot images in a cluster. The key screenshots to prioritize: first screenshot (shows the initial state) + last screenshot (shows the final state / any failure).

**Video:** 16MB WhatsApp limit is tight for webm recordings. Skip video inline — send the report URL link instead. Jordan can watch the video in the browser if he wants.

⚑  Decided: No video in WhatsApp notification (size limits + complexity). Text + screenshots + report URL link covers the value.

---

### Dependency Map

```
whatsapp-notify-bot (standalone)
  └── outbound-uat-webhook (sdlc-server feature)
        └── uat-artifacts-storage (v19 milestone) — for screenshot artifact_urls
              └── tunnel active — for public artifact URLs
```

**Important:** The outbound webhook fires regardless of whether screenshots exist — `artifact_urls` is just empty if v19 isn't shipped yet. This means the feature can ship independently and get immediate value (text notification) before v19 artifacts land.

---

### Open Questions

?  **Media URL timing:** WhatsApp/Twilio fetches media at send time. If the tunnel URL is ephemeral (new URL per server restart), a notification sent after the tunnel rotates will have broken image links. Mitigation: the bot should download and re-upload media on receipt rather than passing URLs to Twilio. This makes the bot slightly more complex but resilient.

?  **Twilio sandbox → production:** Sandbox requires Jordan to send "join <keyword>" to the Twilio WhatsApp number from his phone. Production requires Meta Business verification (~1 week). Should we document this prominently?

?  **Multiple phone numbers:** Could Jordan want to notify teammates? Not in scope for MVP — single recipient configured in bot env.

---

### Feature Scope

**Feature 1: outbound-uat-webhook** (sdlc-server)
- Add `notify_webhooks: Vec<String>` to sdlc config
- On `MilestoneUatCompleted`: read config, build JSON payload, POST to each URL asynchronously
- Log webhook dispatch (info) and failures (warn) — never propagate errors
- Include: event, milestone, run_id, status, verdict, tunnel_url (nullable), artifact_urls (may be empty), report_url

**Feature 2: whatsapp-notify-bot** (standalone — reference implementation)
- Python FastAPI script + Dockerfile
- Receives outbound webhook, sends WhatsApp via Twilio
- Optional: downloads and re-uploads media (resilient to tunnel rotation)
- Setup guide: Twilio sandbox setup, env vars, deployment options (Fly.io, local)

**Dependencies:** Feature 1 can ship now. Feature 2 depends on Feature 1. Both are independent of v19 (screenshots are a nice-to-have enhancement, not a blocker).

---

### Summary

The idea is well-shaped. The key insight is separating the **notification trigger** (sdlc-server outbound webhook — a generic primitive worth building) from the **WhatsApp bot** (Jordan's integration code). This is one small Rust addition and one standalone script.

⚑  Decided: outbound webhook primitive goes in sdlc-server
⚑  Decided: WhatsApp bot is standalone Twilio consumer, not a sdlc crate
⚑  Decided: No video inline — screenshots + report URL
?  Open: Should bot download+re-upload media to avoid tunnel-rotation link-rot?
?  Open: Twilio sandbox vs production — need to document the Meta approval path clearly
