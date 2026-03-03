# Citadel API Surface — Integration-Relevant Endpoints

## Identity
Enterprise observability platform (Rust, 50+ crates). Competing with Datadog.
Hosted at `citadel-staging.orchard9.ai`. Free tier: 500MB/day, 7-day retention.

## Authentication
- **X-Tenant-ID header**: Trusted internal callers (log shippers, agents)
- **JWT Bearer**: External callers (webhooks, serverless, agents from untrusted networks)
- **API Key format**: `ck_<env>_<org>_<random>` — hashed with Argon2id

## Log Fetching
```
GET/POST /api/v1/query           # Search logs with CPL (Citadel Processing Language)
POST     /v2/search              # Advanced search with aggregations
GET      /api/v1/tail            # Live tail (SSE streaming)
GET      /api/v1/errors          # List error groups (fingerprinted)
GET      /api/v1/error-analysis  # "What broke overnight?" analysis (AI, paid)
GET      /api/v1/tenants/{id}/episodes  # Correlated log clusters
```

## Log Annotation
```
GET  /api/v1/annotations/search          # Search annotations
POST /api/v1/annotations                 # CREATE annotation on a log
     body: { annotation_type, content, author_type }
     annotation_type values: "note" | "bug" | "root_cause" | "false_positive" | "incident"
     author_type: "user" | "ai_agent"   ← explicitly designed for agent annotation
```

## Error Status Management
```
GET/POST /api/v1/errors/{fingerprint}/update
     body: { status: "Resolved" | "Ignored" | "Muted" }
```

## Webhook / Push Integration
```
POST /api/v1/webhooks/deployments   # Register webhook to fire on error thresholds
GET  /api/v1/webhooks/deployments   # List registered webhooks
     signature: HMAC-SHA256 (GitHub/Stripe pattern)
```

## Client SDK (Rust)
`citadel-client` crate exposes:
- `CitadelClient::query(QueryParams)` — historical log search
- `CitadelClient::tail()` — live streaming via WebSocket
Query language: CPL (Citadel Processing Language) — custom DSL

## No Native Ticket Creation
Citadel does NOT create tickets/issues. It relies on webhooks to push to external systems.
Agent tools must pull from Citadel and push to Pantheon separately.