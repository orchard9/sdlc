# Plan: Project Navigator + Credential Pool

## Context

From ponder `project-navigator`. Two sessions, full architecture locked. Problem:
Jordan runs multiple sdlc instances locally and can't tell them apart or navigate
between them. In the cluster, there's no listing of deployed projects. Credential pool
is the parallel infrastructure concern that makes fleet agent runs possible.

These are two milestones: the navigator is UI + protocol, the pool is fleet ops.

---

## Milestone 1: v37-project-hub

**Vision:** One bookmark — `localhost:9999` locally, `sdlc.threesix.ai` in the cluster
— shows every live sdlc project as a navigable card. Start a project and it appears.
Kill it and it disappears. Filter by name. Click to navigate. No config to maintain.

### Features

#### hub-server-mode
Hub mode for sdlc-server. `sdlc serve --hub` (or running from a directory with no
`.sdlc/`) starts the hub instead of a project workspace. Hub mode:
- Exposes `POST /api/hub/heartbeat` — accepts `{ name, url, active_milestone,
  feature_count, agent_running }`, updates in-memory registry, refreshes `last_seen`
- Background sweep task every 15s: >90s → mark offline, >5min → evict
- Persists registry to `~/.sdlc/hub-state.yaml` (restart cache, not source of truth)
- Serves the hub React UI at `/`
- SSE stream at `/api/hub/events` — emits `ProjectUpdated`, `ProjectRemoved` events
  so the UI stays live without polling

Tasks:
- Add `HubMode` variant to server startup (detect `--hub` flag or absent `.sdlc/`)
- Implement `POST /api/hub/heartbeat` route + registry struct
- Implement sweep background task (tokio interval, 15s)
- Implement `hub-state.yaml` persistence (load on start, write on change)
- Implement `GET /api/hub/projects` — returns current registry as JSON
- Implement `GET /api/hub/events` — SSE stream for registry changes
- Wire hub mode into `build_router` behind feature flag

#### hub-heartbeat-client
Each project sdlc-server instance sends heartbeats to the hub when one is configured.

- Read hub URL from env var `SDLC_HUB_URL` (unset = skip heartbeats, no error)
- On server start: spawn background task, `POST SDLC_HUB_URL/api/hub/heartbeat` every 30s
- Payload: project name (from `.sdlc/state.yaml`), self URL (from `SDLC_BASE_URL` env),
  active milestone slug, open feature count, whether any agent run is currently active
- On failure: log warn, continue — heartbeat is best-effort, never blocks the server

Tasks:
- Read `SDLC_HUB_URL` and `SDLC_BASE_URL` from env at startup
- Spawn heartbeat task: build payload from AppState, HTTP POST, 5s timeout, retry on next tick
- Include `agent_running: bool` derived from active runs in AppState
- Wire into `new_with_port` (production only, not `new_for_test`)

#### hub-ui
React listing page served in hub mode.

- Route: `/` in hub mode renders the Projects listing (not the normal dashboard)
- Filter text box — client-side, filters on name and slug as you type, shows count
- Project cards: name, URL, green/yellow dot (last_seen age), active milestone,
  feature count, "agent running" badge, chevron → navigates to project URL
- Live via SSE: subscribes to `/api/hub/events`, updates cards without page refresh
- Empty state: shows hub config hint
- Page title: `sdlc hub`

Tasks:
- Create `HubPage.tsx` — top-level page, filter input, card list, SSE subscription
- Create `ProjectCard.tsx` — name, URL, status dot, milestone, feature count, agent badge
- Status dot: green (<30s), yellow (30–90s), nothing (removed by sweep before render)
- Wire hub routing: when `HUB_MODE=true` env injected at build, serve `HubPage` at `/`
  (or detect from a `/api/hub/mode` endpoint, whichever is simpler)
- Empty state component

#### page-title-fix
Every project sdlc-server instance sets a meaningful browser tab title.

- `<title>sdlc — {project_name}</title>` where `project_name` comes from `.sdlc/state.yaml`
- Injected into the HTML template served by sdlc-server (not hardcoded in React)
- Falls back to `sdlc` if project name unavailable

Tasks:
- Read project name from AppState at request time
- Inject into `index.html` template via Axum response middleware or build-time substitution
- Test: verify title changes between projects

---

## Milestone 2: v38-credential-pool

**Vision:** Any project pod in the cluster can run a Claude agent without filesystem
credentials. Tokens are stored in shared Postgres, checked out round-robin so multiple
concurrent runs never overload a single account.

### Features

#### credential-pool-core
The Rust credential pool: connection, schema, checkout logic.

- New file: `crates/sdlc-server/src/credential_pool.rs`
- `struct ClaudeCredential { id: i64, account_name: String, token: String }`
- `struct CredentialPool { pool: PgPool }` — max 5 connections
- `CredentialPool::new(database_url: &str) -> Result<Self, sqlx::Error>`
- `CredentialPool::initialize_schema()` — `CREATE TABLE IF NOT EXISTS claude_credentials`
  with columns: id BIGSERIAL PK, account_name TEXT, token TEXT, is_active BOOL DEFAULT
  true, last_used_at TIMESTAMPTZ DEFAULT '1970-01-01', use_count BIGINT DEFAULT 0;
  partial index on (last_used_at ASC) WHERE is_active
- `CredentialPool::checkout() -> Result<Option<ClaudeCredential>>` — SELECT FOR UPDATE
  SKIP LOCKED ORDER BY last_used_at ASC LIMIT 1, then UPDATE last_used_at + use_count
  in same transaction. Runtime sqlx API only (no compile-time macros).
- Wire into AppState: `pub credential_pool: Arc<OnceLock<Arc<CredentialPool>>>`,
  init in `build_base_state`, async init in `new_with_port` if `DATABASE_URL` is set.
  Warn on failure, never panic. Add `pub mod credential_pool;` to `lib.rs`.
- Add sqlx dependency: `sqlx = { version = "0.8", default-features = false, features =
  ["runtime-tokio-rustls", "postgres", "chrono"] }` to `sdlc-server/Cargo.toml`

Tasks:
- Create `credential_pool.rs` with struct, new, initialize_schema, checkout
- Add `Arc<OnceLock<Arc<CredentialPool>>>` to AppState
- Async init in `new_with_port`: spawn task, connect, schema init, OnceLock::set
- Add sqlx to Cargo.toml
- Add pub mod to lib.rs
- Unit test: mock pool returns credential; no-pool path returns None

#### credential-pool-runs
Inject checked-out token into every agent run.

- Add `async fn checkout_claude_token(app: &AppState) -> Option<String>` to `runs.rs`
- Change `sdlc_query_options` signature: add `claude_token: Option<String>` param;
  if Some, insert `CLAUDE_CODE_OAUTH_TOKEN` into env map
- Same change propagated to `sdlc_ponder_query_options` and
  `sdlc_guideline_query_options` (they delegate to `sdlc_query_options`)
- All ~25 call sites in `runs.rs`: add `let claude_token = checkout_claude_token(&app).await;`
  before each `sdlc_query_options` call

Tasks:
- Implement `checkout_claude_token`
- Update `sdlc_query_options` signature + env injection
- Update `sdlc_ponder_query_options` and `sdlc_guideline_query_options`
- Update all call sites (they are all async, so await is valid)
- Integration test: token set → env var present in QueryOptions

#### credential-pool-helm
Helm chart changes to provision credentials from GCP Secret Manager.

- New template: `external-secret-postgres.yaml` — ExternalSecret syncing
  `k3sf-postgres-sdlc` (field: `database_url`) → k8s Secret `postgres-sdlc-credentials`
  (key: `DATABASE_URL`). Uses same ClusterSecretStore as gitea (`gcp-secret-manager`).
- `values.yaml`: add `postgres.externalSecret.gsmKey: k3sf-postgres-sdlc`
- `deployment.yaml`: add env var `DATABASE_URL` from secretKeyRef
  `postgres-sdlc-credentials.DATABASE_URL` to sdlc-server container

Tasks:
- Write `external-secret-postgres.yaml`
- Add postgres block to `values.yaml`
- Add DATABASE_URL env injection to `deployment.yaml`
- Ops prerequisite (not a code task): create GCP Secret `k3sf-postgres-sdlc` with
  `database_url` field pointing to cluster Postgres
