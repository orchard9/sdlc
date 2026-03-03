# Citadel ↔ Pantheon Integration Architecture

## The Core Observation
These two systems are designed to be complementary:
- **Citadel** = observability layer (what happened, when, why — log data)
- **Pantheon** = action layer (what to do about it — tasks, incidents, runbooks)

Citadel has no ticket creation. Pantheon has no log access. The gap is the integration.

## Two Tiers of Integration

### Tier 1: Server-to-Server (Push, Automatic)
Citadel deployment webhooks → Pantheon incident creation API

```
[Citadel] error threshold exceeded
     ↓ HMAC-signed POST
[Pantheon] POST /api/v1/organizations/{orgSlug}/incidents/
     severity: mapped from Citadel error level
     title: error fingerprint + count
     body: Citadel episode link + top affected traces
```

No custom code — just configuration. Citadel already has:
- `POST /api/v1/webhooks/deployments` — register webhook URL
- HMAC-SHA256 signing (GitHub pattern) — Pantheon can verify

Pantheon already has:
- Alertmanager webhook ingestion (same pattern, just needs Citadel variant)
- Incident creation from webhook data

⚑  Decided: Tier 1 is config-only, not a code feature. Needs a Pantheon webhook handler
  that understands Citadel's payload format.

### Tier 2: Agent Tools (Pull, On-Demand)
Pantheon agent gets Citadel as a registered App with 3 tools:

```
Tool 1: citadel_query_logs
  params: { query: string (CPL), time_range: string, limit: int }
  calls:  GET /api/v1/query (Citadel)
  auth:   Citadel API key from ToolCredential
  returns: log entries + episode context

Tool 2: citadel_annotate_log
  params: { log_id: string, content: string, annotation_type: string }
  calls:  POST /api/v1/annotations (Citadel)
  auth:   Citadel API key from ToolCredential
  author_type: "ai_agent" (Citadel explicitly supports this)
  returns: annotation ID + timestamp

Tool 3: citadel_create_pantheon_item
  params: { logs: [LogEntry], title: string, type: "task" | "incident", severity? }
  logic:  1. Annotate each log with link to new item (Citadel)
          2. Create task or incident (Pantheon)
          3. Return created item URL
  NOTE: This tool is actually TWO API calls (Citadel + Pantheon), meaning it
        might be better as a runbook pattern than a single tool call.
```

### Alternative: Citadel Webhook → Pantheon Without Custom Agent Tool
Citadel already fires webhooks. If Pantheon adds a Citadel webhook handler:
`POST /api/v1/webhooks/citadel` → parses Citadel payload → creates incident/task
The agent tools become optional (agents query Citadel via the App Platform when needed).

## Ownership Question

| Approach | Where | Effort | Best for |
|---|---|---|---|
| **Pantheon App Platform** | Citadel as registered app in Pantheon | Medium | Interactive agent use |
| **Pantheon webhook handler** | New handler in Pantheon routes | Small | Automatic incident creation |
| **sdlc .sdlc/tools/** | Shell scripts wrapping REST calls | Small | Dev-driver / sdlc workflow |
| **Standalone library** | Go/Rust citadel-client shared lib | Large | Long-term, reusable |

?  Open: Is this work happening in Pantheon (new app integration) or in sdlc (dev-driver tools)?
  The framing "first integrations" suggests Pantheon's app platform is the target.

## Data Flow (Full Loop)
```
1. Service logs → Citadel (ingest)
2. Citadel detects error spike → webhook → Pantheon (auto-incident)
3. Pantheon agent: "query citadel for logs around this incident"
4. Agent calls citadel_query_logs → gets relevant log entries + episodes
5. Agent calls citadel_annotate_log on key entries ("linked to incident-456")
6. Agent summarizes findings → updates Pantheon incident timeline
7. Agent creates Pantheon task: "fix NullPointerException in auth service"
8. Developer fixes bug → deployment → Citadel deployment episode → closes loop
```

## CPL (Citadel Processing Language)
Citadel has its own query DSL. Agents will need to know how to write CPL queries.
Example agent instruction: "Query format: level:error service:auth time:1h"
This is a training/skill concern — the integration skill must include CPL basics.

## Credentials Management
- Citadel API key → stored in Pantheon ToolCredential (org-scoped, encrypted)
- Pantheon API key → if sdlc tools, stored in `sdlc secrets env` (age-encrypted)
- Both are org/project scoped — multi-tenant by default

## Next Step
Decide: Pantheon app platform integration vs. sdlc tools.
Then: Sketch the ToolDefinition JSON schema for Citadel (if Pantheon route)
OR: Write the 3 shell scripts as .sdlc/tools/ entries (if sdlc route).