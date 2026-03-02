# QA Plan: sdlc knowledge librarian run — maintenance pass + harvest hooks wired

## Scope

Verify the CLI subcommand, core library function, server endpoints, SSE events, and hook wiring all work correctly. No frontend UI is in scope.

---

## 1. Unit Tests (automated, `cargo test`)

### 1.1 `librarian_harvest_workspace` — new investigation entry

- Set up a TempDir with `.sdlc/investigations/<slug>/manifest.yaml` (status: complete).
- Call `knowledge::librarian_harvest_workspace(root, "investigation", slug)`.
- Assert: `HarvestResult.created == true`.
- Assert: entry exists at `.sdlc/knowledge/<entry_slug>/entry.yaml` with `origin: harvested`, `harvested_from: "investigation/<slug>"`.
- Assert: a `MaintenanceAction` with `action_type: "harvest"` is appended to maintenance log.

### 1.2 `librarian_harvest_workspace` — idempotent on second call

- Repeat the call from 1.1 on the same workspace.
- Assert: `HarvestResult.created == false`.
- Assert: no duplicate maintenance log entries (or a second "harvest" action with the update timestamp).

### 1.3 `librarian_harvest_workspace` — ponder workspace

- Set up a TempDir with `.sdlc/ponders/<slug>/manifest.yaml`.
- Call `knowledge::librarian_harvest_workspace(root, "ponder", slug)`.
- Assert: `HarvestResult.created == true`, `harvested_from == "ponder/<slug>"`.

### 1.4 `HarvestResult` serialization

- Assert `serde_json::to_value(HarvestResult { entry_slug: "..".into(), created: true })` produces `{ "entry_slug": "..", "created": true }`.

---

## 2. CLI Integration Tests (`crates/sdlc-cli/tests/integration.rs`)

### 2.1 `sdlc knowledge librarian run` (maintain mode, default)

- Run `sdlc knowledge librarian run` in a temp project dir.
- Assert exit code 0.
- Assert stdout contains text describing the maintenance checks (URL health, harvest pending, etc.).

### 2.2 `sdlc knowledge librarian run --mode maintain`

- Same as 2.1 with explicit flag. Assert exit code 0.

### 2.3 `sdlc knowledge librarian run --mode harvest --type investigation --slug <slug>`

- Set up a temp project dir with a completed investigation workspace.
- Run `sdlc knowledge librarian run --mode harvest --type investigation --slug <slug>`.
- Assert exit code 0.
- Assert stdout confirms a knowledge entry was created.

### 2.4 `sdlc knowledge librarian harvest --type investigation --slug <slug>`

- Same scenario as 2.3 using the direct `harvest` subcommand alias.
- Assert exit code 0.

### 2.5 `sdlc knowledge librarian run --mode harvest` with missing `--type`/`--slug`

- Assert non-zero exit code and error message indicating missing arguments.

### 2.6 JSON output — `sdlc --json knowledge librarian harvest --type ponder --slug <slug>`

- Assert stdout is valid JSON with keys `type`, `slug`, `created`, `entry_slug`.

---

## 3. Hook Wiring Tests

### 3.1 `sdlc investigate update --status complete` triggers harvest (smoke)

- Set up a temp dir with an investigation workspace.
- Run `sdlc investigate update <slug> --status complete`.
- Assert exit code 0 (status update succeeds regardless of whether `sdlc` binary is in PATH for the subprocess call).
- Assert investigation manifest shows `status: complete`.
- Assert: if `sdlc` binary is available in PATH, a knowledge entry with `harvested_from: "investigation/<slug>"` is created.

### 3.2 Hook failure does not abort the status update

- In an environment where the `sdlc` subprocess would fail (e.g., wrong PATH or knowledge dir absent), run `sdlc investigate update <slug> --status complete`.
- Assert exit code 0 — the status update still succeeds.

### 3.3 `sdlc ponder update --status complete` triggers harvest (smoke)

- Same pattern as 3.1 for ponder workspaces.

---

## 4. Server Endpoint Tests (`crates/sdlc-server/tests/integration.rs`)

### 4.1 `POST /api/knowledge/maintain` — happy path

- Start test server.
- POST to `/api/knowledge/maintain` with empty body.
- Assert HTTP 200.
- Assert response JSON has `run_id` (non-empty string) and `status: "started"`.

### 4.2 `POST /api/knowledge/maintain` — conflict when already running

- Start a maintain run, then immediately POST again.
- Assert HTTP 409 Conflict.

### 4.3 `POST /api/knowledge/harvest` — valid body

- POST `{ "type": "investigation", "slug": "test-slug" }` to `/api/knowledge/harvest`.
- Assert HTTP 200 with `run_id` and `status: "started"`.

### 4.4 `POST /api/knowledge/harvest` — invalid type

- POST `{ "type": "unknown", "slug": "test" }`.
- Assert HTTP 400 Bad Request.

### 4.5 Routes registered before `/api/knowledge/:slug`

- Confirm `POST /api/knowledge/maintain` is not matched by the parameterized slug route (i.e., "maintain" is not treated as a slug). Assert 200, not 404 or 405.

---

## 5. SSE Event Tests

### 5.1 `KnowledgeMaintenanceStarted` serialization

- Assert the `SseMessage::KnowledgeMaintenanceStarted` variant serializes to `{ "type": "KnowledgeMaintenanceStarted" }` (or equivalent canonical form matching the project's SSE serialization convention).

### 5.2 `KnowledgeMaintenanceCompleted` serialization

- Assert `SseMessage::KnowledgeMaintenanceCompleted { actions_taken: 5 }` serializes to `{ "type": "KnowledgeMaintenanceCompleted", "actions_taken": 5 }`.

### 5.3 `KnowledgeMaintenanceStarted` is emitted on `POST /api/knowledge/maintain`

- Subscribe to SSE stream, POST to `/api/knowledge/maintain`, assert `KnowledgeMaintenanceStarted` event is received before `RunStarted`.

---

## 6. Build & Lint

| Check | Command | Pass Criteria |
|---|---|---|
| Build | `cargo build --all` | Zero errors |
| Tests | `SDLC_NO_NPM=1 cargo test --all` | All tests green |
| Clippy | `cargo clippy --all -- -D warnings` | Zero warnings |

---

## 7. Acceptance Criteria Checklist

- [ ] `sdlc knowledge librarian run` exits 0 and prints maintenance instructions.
- [ ] `sdlc knowledge librarian run --mode harvest --type investigation --slug <slug>` exits 0 and creates a knowledge entry.
- [ ] `sdlc knowledge librarian harvest --type ponder --slug <slug>` exits 0 and creates a knowledge entry.
- [ ] `sdlc investigate update --status complete` succeeds and (when `sdlc` is in PATH) triggers harvest.
- [ ] `sdlc ponder update --status complete` succeeds and (when `sdlc` is in PATH) triggers harvest.
- [ ] `POST /api/knowledge/maintain` returns `{ run_id, status: "started" }`.
- [ ] `POST /api/knowledge/harvest` with valid body returns `{ run_id, status: "started" }`.
- [ ] `POST /api/knowledge/harvest` with invalid type returns 400.
- [ ] SSE emits `KnowledgeMaintenanceStarted` when maintenance run starts.
- [ ] SSE emits `KnowledgeMaintenanceCompleted` when maintenance run finishes.
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes.
- [ ] `cargo clippy --all -- -D warnings` clean.
