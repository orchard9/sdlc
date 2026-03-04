# Hub Mode Architecture Decisions

_Updated to reflect heartbeat-based registration and credential pool._

## Core model

sdlc-server gains a "hub mode" — when started without a project directory (or with `--hub`
flag), it renders a project navigator instead of a project workspace.

Hub mode is a first-class server mode, not a bolt-on feature.

## Registration: heartbeat protocol

There is no explicit register/deregister. Each project instance sends a heartbeat to the
hub every 30 seconds. First beat = registration. Silence = deregistration.

**Endpoint:** `POST /api/hub/heartbeat`

**Payload:**
```json
{
  "name": "payments-api",
  "url": "http://localhost:3001",
  "active_milestone": "v12-checkout-flow",
  "feature_count": 3,
  "agent_running": false
}
```

**Hub sweep** (every 15s):
- `last_seen > 90s` → mark offline (grey dot)
- `last_seen > 5min` → remove from listing

Hub persists state to `~/.sdlc/hub-state.yaml` (local) so restarts don't show an empty
page while pods re-register. The file is a cache, not the source of truth.

## Card data

Each card shows:
- Project name + URL/port — from heartbeat payload (instant once first beat received)
- Reachability dot — green (beat <30s), yellow (beat 30–90s), removed (>5min)
- Active milestone name — from heartbeat payload
- Feature count — from heartbeat payload
- "agent running" badge — from heartbeat payload

All data arrives via heartbeat — no per-card client-side fetches needed.

## Local topology

```
  $ sdlc serve (payments-api/ :3001)    $ sdlc serve (auth-service/ :3004)
  ┌───────────────────┐                 ┌───────────────────┐
  │  sdlc-server      │                 │  sdlc-server      │
  │  every 30s ───────┼─────┐           │  every 30s ───────┼─────┐
  └───────────────────┘     │           └───────────────────┘     │
                            ▼                                     ▼
                   ┌─────────────────────────────────────────────────┐
                   │  sdlc hub  :9999                                │
                   │  POST /api/hub/heartbeat                        │
                   │  sweep every 15s                                │
                   │  persists → ~/.sdlc/hub-state.yaml              │
                   └────────────────────┬────────────────────────────┘
                                        │
                                 localhost:9999
                                 ┌──────┴───────┐
                                 │  Projects    │
                                 │  ● payments  │──► :3001
                                 │  ● auth      │──► :3004
                                 └──────────────┘
```

## Cluster topology

```
  k3s cluster
  ┌────────────────────────────────────────────────────────────────┐
  │  ns: sdlc-payments-api                                         │
  │  ┌───────────────────────────────────────┐                     │
  │  │  sdlc-server  │  git-sync             │                     │
  │  │  beats every 30s to hub ClusterIP ────┼──────────┐          │
  │  └───────────────────────────────────────┘          │          │
  │                                                     │          │
  │  ns: sdlc-auth-service                              │          │
  │  ┌───────────────────────────────────────┐          │          │
  │  │  sdlc-server  │  git-sync             │          │          │
  │  │  beats every 30s to hub ClusterIP ────┼──────────┤          │
  │  └───────────────────────────────────────┘          │          │
  │                                                     ▼          │
  │  ns: sdlc-hub                                                   │
  │  ┌─────────────────────────────────────────────────────────┐   │
  │  │  sdlc-server (hub mode)  │  git-sync                    │   │
  │  │  ClusterIP: reachable from all namespaces                │   │
  │  │  ingress: sdlc.threesix.ai                               │   │
  │  └─────────────────────────────────────────────────────────┘   │
  │                                                                 │
  │  ns: databases                                                  │
  │  ┌──────────────────────────────────────────────────────────┐  │
  │  │  postgres-0 (shared cluster postgres)                    │  │
  │  │  db: appdb                                               │  │
  │  │  table: claude_credentials (credential pool)             │  │
  │  └──────────────────────────────────────────────────────────┘  │
  └────────────────────────────────────────────────────────────────┘
```

No Woodpecker changes needed for hub registration. Pod comes up → beats start → appears
in listing. Pod deleted → beats stop → disappears after 5min timeout.

## Hub location

| Environment | URL | Auth |
|---|---|---|
| Local | `localhost:9999` | None (local passthrough) |
| Cluster | `sdlc.threesix.ai` | Google OAuth (personal, org-restricted) |

## What hub mode does NOT do

- No cross-project agents or aggregate dashboards
- No project management from the hub
- No live Kubernetes API calls
- No groups, favorites, or sorting beyond default order (latest beat first)

## Page title fix (zero-cost companion)

Every sdlc project instance sets `<title>sdlc — {project-name}</title>`.
Distinct browser tabs immediately, without the hub.

---

# Credential Pool Architecture

## Problem

In a containerized fleet, `~/.claude/` credentials can't live on pod filesystems.
Each project pod shells out to `claude` CLI via `Command::new("claude")`. Claude Code
uses subscription OAuth tokens. We need a shared, round-robin credential pool backed by
the cluster's existing Postgres so multiple pods can run agents concurrently.

## Design

```
spawn_agent_run
  → checkout_claude_token(app)     ← async, SELECT FOR UPDATE SKIP LOCKED
  → sdlc_query_options(root, turns, token)
  → QueryOptions { env: {"CLAUDE_CODE_OAUTH_TOKEN": token} }
  → ClaudeProcess::spawn → cmd.env(k, v) → claude subprocess
```

## Postgres table

```sql
CREATE TABLE IF NOT EXISTS claude_credentials (
  id            BIGSERIAL PRIMARY KEY,
  account_name  TEXT NOT NULL,
  token         TEXT NOT NULL,
  is_active     BOOL NOT NULL DEFAULT true,
  last_used_at  TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01',
  use_count     BIGINT NOT NULL DEFAULT 0
);
CREATE INDEX ON claude_credentials (last_used_at ASC) WHERE is_active;
```

## Round-robin checkout

```sql
-- In a transaction:
SELECT id, account_name, token
FROM claude_credentials
WHERE is_active
ORDER BY last_used_at ASC
LIMIT 1
FOR UPDATE SKIP LOCKED;

UPDATE claude_credentials
SET last_used_at = NOW(), use_count = use_count + 1
WHERE id = $1;
```

`SKIP LOCKED` ensures two concurrent `spawn_agent_run` calls never block each other —
they each grab a different credential row.

## Graceful degradation

If `DATABASE_URL` is unset, pool init is skipped.
If pool is empty or DB unreachable → `warn!("no active Claude credentials")` → run
without token injection (ambient auth fallback). Never panics. Never blocks a run.

## Rust components

| File | Role |
|---|---|
| `crates/sdlc-server/src/credential_pool.rs` | `CredentialPool`, `ClaudeCredential`, checkout logic |
| `crates/sdlc-server/src/state.rs` | `Arc<OnceLock<Arc<CredentialPool>>>` field on `AppState` |
| `crates/sdlc-server/src/routes/runs.rs` | `checkout_claude_token()`, injects into `sdlc_query_options` |

## Helm / secrets

```
GCP Secret Manager: k3sf-postgres-sdlc
  field: database_url = "postgresql://appuser:<pass>@postgres.databases.svc.cluster.local:5432/appdb"

ExternalSecret → k8s Secret "postgres-sdlc-credentials"
  → deployment.yaml env: DATABASE_URL
```

## Connection pool

`sqlx::PgPool` with `max_connections: 5` — small, because checkout transactions are
sub-millisecond and shared across all agent runs on the pod.

## Adding credentials (ops)

```sql
INSERT INTO claude_credentials (account_name, token)
VALUES ('you@example.com', '<token from claude setup-token>');
```

Multiple rows = automatic round-robin across accounts.
