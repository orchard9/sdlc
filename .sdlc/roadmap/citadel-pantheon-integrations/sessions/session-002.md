---
session: 2
timestamp: 2026-03-03T03:30:00Z
orientation:
  current: "Citadel and Pantheon are fully mapped. Citadel = observability (Rust, 50+ crates, REST API, annotation + webhook). Pantheon = developer hub (Go, Discord/web/CLI, task/incident/runbook management, tool registry). Clear two-tier integration architecture: server-to-server webhooks (automatic) + agent tool registry (on-demand). Key open question: is this work in Pantheon's app platform, or sdlc .sdlc/tools/?"
  next: "Jordan to clarify ownership: Pantheon app platform integration vs. sdlc dev-driver tools — then decide which approach, sketch the ToolDefinition schema or shell tool shape, and start implementation."
  commit: "Ownership decision made + concrete data flow confirmed + one prototype tool shape (JSON schema or shell script) drafted"
---

**jordan · Owner**
Read through citadel and pantheon at ~/Workspace/orchard9/citadel and pantheon

---

## Session 2: Deep System Exploration

### What we learned

Both projects were fully explored from source code. This session closes the "what are these systems?" question from Session 1.

---

### Citadel: Enterprise Observability Platform

**Rust, 52+ workspace crates.** A Datadog competitor built on the principle of "never drop a log." Three tenets: zero data loss (quarantine-first writes), zero config (auto-detection), AI first (intelligent log clustering).

Key architectural layers:
- **Quarantine Journal** — fsync-guaranteed write before response (append-only)
- **Compression/Mapping** — MinHash/LSH template extraction, 50x compression
- **WAL Batcher** — 200K+ appends/sec group commit
- **Tiered Storage** — NVMe (7 days) → SSD (30 days) → S3 (long-term)
- **Episode Engine** — correlated log groups (error, request, anomaly, deployment)
- **Error Tracker** — stack trace fingerprinting across Java/Python/JS/Go/Rust

**Authentication:** X-Tenant-ID (trusted internal callers) + JWT Bearer (external) + API keys (`ck_<env>_<org>_<random>`, Argon2id hashed).

**The three operations Jordan wants are directly supported by the Citadel API:**

1. **Fetch logs** — `GET/POST /api/v1/query` (CPL query language), `POST /v2/search` (aggregations), `GET /api/v1/errors` (fingerprinted error groups), `GET /api/v1/tenants/{id}/episodes` (correlated clusters)
2. **Annotate logs** — `POST /api/v1/annotations` with `annotation_type` ("note" | "bug" | "root_cause" | "false_positive" | "incident") and crucially: `author_type: "ai_agent"` — Citadel *explicitly designed* annotations for agent use
3. **Create tickets based on logs** — Citadel has NO ticket creation. It fires webhooks on error thresholds. Ticket creation must go to Pantheon.

⚑  Decided: "Create tickets based on logs" = call Pantheon's task/incident API. This is a two-system operation, not a Citadel-only operation.

---

### Pantheon: Developer Hub Platform

**Go, Chi router.** Discord-first developer hub for humans + AI agents collaborating on operational work. Three surfaces: Discord bot, web UI (Next.js), CLI (Cobra with device auth).

Core domain: Orgs → Teams → Projects → Tasks. Plus: Incidents, Runbooks, AgentProfiles, memory (Synap vector DB), K8s cluster tools.

**Task statuses:** backlog → todo → in_progress → review → blocked → done | wont_do | cancelled

**Relevant capability: App Platform / Tool Registry**

Pantheon already has a first-class tool registration system:
- `ToolDefinition` — JSON schema for an external API call
- `ToolCredential` — encrypted API credentials scoped to a tool + org
- Tool execution — agent invokes tool, Pantheon handles auth + HTTP
- `ApprovalGate` — human approval required for high-risk tool calls

This is NOT a coincidence. Pantheon's tool registry exists precisely to add new integrations like Citadel. The current tool categories (tasks, incidents, runbooks, K8s, deployments) have NO observability / log inspection capability. **Citadel fills this gap.**

**Incident management:**
```
POST /api/v1/organizations/{orgSlug}/incidents/
     severity: critical | high | medium | low
     status lifecycle: investigating → identified → mitigating → resolved
```

Agent already auto-creates incidents when it detects "site is down", "500 errors" in chat. Citadel logs would feed this signal directly.

---

### Integration Architecture

**⚑  Decided: Two-tier integration, not one.**

**Tier 1: Server-to-server webhook (automatic, configuration-only)**

Citadel already fires HMAC-SHA256 webhooks when error thresholds are exceeded (`POST /api/v1/webhooks/deployments`). Pantheon needs a new handler:

```
POST /api/v1/webhooks/citadel  →  parse payload  →  create Pantheon incident
```

Citadel's webhook → Pantheon's incident API. No agent involvement. Fires automatically when Citadel detects error spikes. This is the "production blew up, create an incident" path.

Effort: Small. Pantheon already has an Alertmanager webhook handler as a pattern.

**Tier 2: Agent tools (on-demand, via App Platform)**

Register Citadel as a Pantheon App with 3 tools:

```
Tool: citadel_query_logs
  params: { query: string (CPL), time_range: string, limit: int }
  calls:  GET /api/v1/query
  auth:   Citadel API key (from ToolCredential)
  returns: log entries, episode context

Tool: citadel_annotate_log
  params: { log_id: string, content: string, annotation_type: string }
  calls:  POST /api/v1/annotations
  auth:   Citadel API key (from ToolCredential)
  author_type: "ai_agent"   ← Citadel explicitly supports this
  returns: annotation ID + timestamp

Tool: citadel_create_incident_from_logs  (better as a Runbook)
  logic:  1. query_logs for relevant window
          2. annotate key entries ("linked to incident-{id}")
          3. create Pantheon incident with log summary + episode link
```

?  Open: Tool 3 spans two APIs (Citadel annotation + Pantheon incident creation). Should this be a compound Pantheon tool (agent does both calls), or a Pantheon Runbook with two steps?

**Lean: Runbook.** Pantheon already has multi-step runbooks. A runbook "Create incident from Citadel logs" could be: Step 1 (command: citadel_query_logs) → Step 2 (command: citadel_annotate_log) → Step 3 (command: create_incident). This fits the existing model cleanly.

⚑  Decided: "Create ticket from logs" is modeled as a Pantheon Runbook, not a monolithic tool.

---

### The Full Loop (Target State)

```
1. Service logs → Citadel ingest
2. Citadel detects error spike → HMAC webhook → Pantheon incident created (auto, Tier 1)
3. Developer in Discord: "query citadel for logs around incident-456"
4. Pantheon agent: calls citadel_query_logs("level:error service:auth time:1h")
5. Agent receives log entries + episode context
6. Agent calls citadel_annotate_log on key entries ("linked to incident-456")
7. Agent updates Pantheon incident timeline with log summary
8. Agent creates Pantheon task: "fix NullPointerException in auth service"
9. Developer fixes bug → deploys → Citadel deployment episode created → loop closed
```

This is a complete observability → action loop. Citadel provides signal; Pantheon provides resolution.

---

### CPL (Citadel Processing Language)

Citadel has its own query DSL for log search. Agents will need CPL fluency. The integration skill / Pantheon agent instructions must include:
- Basic CPL syntax: `level:error`, `service:auth`, `time:1h`, boolean operators
- Common patterns: error queries, service-specific queries, time windowing
- Episode-aware queries: correlating logs to a specific `trace_id` or `correlation_id`

This is a skill/training concern alongside the API integration concern.

---

### Ownership Question

?  Open (needs Jordan's input): Where does this work happen?

| Path | Where | Effort |
|---|---|---|
| **Pantheon webhook handler** | New `/webhooks/citadel` route in Pantheon | Small |
| **Pantheon App Platform** | Citadel as registered App in Pantheon tool registry | Medium |
| **sdlc .sdlc/tools/** | Shell scripts wrapping Citadel REST API | Small |
| **Both (recommended)** | Webhook (Tier 1 auto) + App Platform (Tier 2 agent) | Medium |

Strong lean: **Both** — webhook handler for automatic incident creation + App Platform for agent-driven log queries and annotation. The phrase "first integrations" implies establishing a pattern in Pantheon's app platform, not one-off shell scripts.

The sdlc route (shell scripts) is viable if Jordan wants dev-driver-accessible tools, but it's lower leverage than a Pantheon-native integration that Discord users can access from chat.

---

### Assessment against Session 1 commit signal

> "Clear integration target definitions + one user story per tool + API availability assessment"

- ✅ **Integration target definitions** — both systems fully understood from source code
- ✅ **API availability assessment** — both have documented REST APIs with well-defined auth
- ✅ **User stories**:
  1. **Fetch logs**: "As a Pantheon agent, I want to query Citadel for error logs around an incident so I can give developers the context they need in Discord"
  2. **Annotate logs**: "As a Pantheon agent, I want to annotate Citadel log entries with incident/task links so the observability and task layers stay connected"
  3. **Create ticket from logs**: "As a Pantheon agent, I want to run a runbook that queries logs → annotates key entries → creates a Pantheon incident, so production issues can be triaged in one Discord command"

**Session 1 commit signal is met.** Recommending status → `converging`.

New commit signal for Session 3: **Ownership decision made + first ToolDefinition JSON schema or Pantheon webhook handler spec drafted.**

---

### Scrapbook artifacts captured this session
- `citadel-api-surface.md` — Citadel REST API reference for the 3 integration operations
- `pantheon-api-surface.md` — Pantheon task/incident/tool-registry API reference
- `integration-architecture.md` — Two-tier architecture with data flow and ownership options
