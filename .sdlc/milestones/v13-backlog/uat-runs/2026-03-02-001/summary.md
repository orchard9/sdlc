# UAT Run — v13-backlog: Project-level backlog: capture and promote out-of-scope concerns
**Date:** 2026-03-02T09:50:00Z
**Verdict:** PassWithTasks
**Tests:** 26/33 (CLI core + REST API pass; dashboard UI and guidance docs missing)
**Tasks created:** backlog-cli#T11, backlog-cli#T12, backlog-server#T5, backlog-dashboard#T10, backlog-guidance#T9

## Pre-flight: CLI Registration
The `backlog` command was implemented in `crates/sdlc-cli/src/cmd/backlog.rs` and `crates/sdlc-core/src/backlog.rs` but not registered in `cmd/mod.rs` or `main.rs`. Server routes existed in `crates/sdlc-server/src/routes/backlog.rs` but not wired in `routes/mod.rs` or `lib.rs`. Both issues were fixed inline before testing.

## Results by Scenario

### Scenario 1: Agent captures concern immediately upon discovery — PASS
- `sdlc backlog add ... --kind concern --evidence ... --source-feature ...` → B1 recorded with ID, title, source_feature confirmation
- `sdlc backlog list` shows B1 with status `open`, kind `concern`, id `B1`
- `sdlc backlog show B1` confirms title, kind, evidence, source_feature, created_at

### Scenario 2: kind and evidence fields are present and useful — PASS
- Added B2 (idea), B3 (debt) successfully
- `sdlc backlog list` shows all 3 items with kind column
- `sdlc backlog list --source-feature run-events-api` shows only B2

### Scenario 3: Auto-infer source_feature — PASS WITH TASK
- Item recorded without `--source-feature` → inferred `dashboard-empty-states` from state.yaml ✓
- Missing: no stderr/stdout warning "No source feature provided — inferred from state.yaml: <slug>" → Task backlog-cli#T11

### Scenario 4: Park with required reason — PASS WITH TASK
- `sdlc backlog park B2` (no --reason) → rejected with helpful error ✓
- `sdlc backlog park B2 --reason "Revisit after v14 ships"` → succeeds ✓
- `sdlc backlog list` (default) hides B2 ✓
- `sdlc backlog list --all` shows B2 with status `parked` ✓
- Missing: `list --all` doesn't show park_reason column → Task backlog-cli#T12

### Scenario 5: Promote to feature in milestone — PASS
- `sdlc backlog promote B1 --slug auth-race-fix --milestone v13-backlog` → "Promoted B1 → feature: auth-race-fix / Added to milestone: v13-backlog" ✓
- `sdlc backlog show B1` shows `status: promoted`, `promoted_to: auth-race-fix` ✓
- `sdlc feature list` includes `auth-race-fix` in draft phase ✓
- `sdlc milestone info v13-backlog` includes `auth-race-fix` ✓

### Scenario 6: REST API — PASS WITH TASK
- `GET /api/backlog` returns 4 items ✓
- `GET /api/backlog?status=open` returns 2 open items ✓
- `GET /api/backlog?source_feature=run-events-api` returns run-events-api item ✓
- `POST /api/backlog` creates B5 with ID ✓
- `POST /api/backlog/B3/park` succeeds ✓
- `POST /api/backlog/B5/promote` requires `title` field not in spec → Task backlog-server#T5

### Scenario 7: Dashboard legibility — FAILED (tasks filed)
- `backlog-dashboard` feature in draft phase, no spec — full UI not implemented
- Missing: BacklogPage, empty state, item cards, park modal, promote button, staleness indicator, filter dropdown, toast
- Task: backlog-dashboard#T10

### Scenario 8: Guidance enforces immediate capture — FAILED (tasks filed)
- `sdlc-run` template has no backlog capture instruction or example
- `guidance.md` has 11 sections, §12 Session Close Protocol absent
- Task: backlog-guidance#T9

## Failures
| Area | Classification | Resolution |
|---|---|---|
| Auto-infer warning message missing | Code bug (minor) | Task backlog-cli#T11 created |
| list --all missing park_reason column | Code bug (minor) | Task backlog-cli#T12 created |
| REST promote requires explicit title | Code bug (API ergonomics) | Task backlog-server#T5 created |
| Dashboard UI not implemented | Feature not complete | Task backlog-dashboard#T10 created |
| Guidance docs missing §12 + sdlc-run template | Feature not complete | Task backlog-guidance#T9 created |
