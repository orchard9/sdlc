# QA Plan: Spike REST Routes

## Automated Tests (Rust integration tests in sdlc-server)

All tests use `build_router_for_test` with a `TempDir` — no real file system side effects.

### GET /api/spikes

| Scenario | Expected |
|---|---|
| No `.sdlc/spikes/` directory exists | 200 + `[]` |
| Directory exists but empty | 200 + `[]` |
| One spike with ADOPT verdict | 200 + array with one entry, `verdict: "ADOPT"` |
| Multiple spikes with different dates | 200 + sorted by date descending |
| Spike with no findings.md | 200 + entry with null verdict, null date, title = slug |

### GET /api/spikes/:slug

| Scenario | Expected |
|---|---|
| Valid slug with findings.md | 200 + entry fields + `findings` string |
| Unknown slug | 404 + `{ "error": "..." }` |
| Slug present, no findings.md | 200 + nulls for parsed fields, `findings: ""` |

### POST /api/spikes/:slug/promote

| Scenario | Expected |
|---|---|
| ADAPT spike, no body | 200 + `{ "ponder_slug": "<slug>" }` |
| ADAPT spike, body with override slug | 200 + `{ "ponder_slug": "<override>" }` |
| ADOPT spike | 422 + `{ "error": "only ADAPT spikes can be promoted" }` |
| REJECT spike | 422 + `{ "error": "only ADAPT spikes can be promoted" }` |
| Spike with no verdict | 422 |
| Unknown slug | 404 |

## Build Quality Checks

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no warnings
- No `unwrap()` in new library or server code
- All file I/O through existing patterns (`spawn_blocking`)

## Manual Smoke Test (optional, for local validation)

```bash
# With a real spike in .sdlc/spikes/
curl http://localhost:3000/api/spikes
curl http://localhost:3000/api/spikes/my-spike
curl -X POST http://localhost:3000/api/spikes/my-spike/promote \
  -H "Content-Type: application/json" -d '{}'
```
