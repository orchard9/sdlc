# QA Plan: backlog-server

## Verification Method

Primary: Rust integration tests via `cargo test` — `crates/sdlc-server/tests/integration.rs`.
Secondary: Clippy lint pass — `cargo clippy --all -- -D warnings`.
No frontend or Playwright tests needed — this is a pure server-side API feature.

## Test Command

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-server -- backlog 2>&1
```

All backlog-related tests must pass. Full suite must not regress.

## Test Cases

### TC-1: List empty backlog
- `GET /api/backlog` on a freshly initialized project
- Expected: `200 OK`, body = `[]`

### TC-2: Create backlog item
- `POST /api/backlog` with `{ "title": "auth.rs: race condition", "kind": "concern" }`
- Expected: `201 Created`, body contains `id: "B1"`, `status: "open"`, `kind: "concern"`

### TC-3: List after create
- `POST /api/backlog` to add an item, then `GET /api/backlog`
- Expected: `200 OK`, body is array with the created item

### TC-4: Status filter — open
- Add two items, park one, `GET /api/backlog?status=open`
- Expected: only the un-parked item returned

### TC-5: Status filter — parked
- Add two items, park one, `GET /api/backlog?status=parked`
- Expected: only the parked item returned

### TC-6: Source feature filter
- Add two items with different `source_feature`, `GET /api/backlog?source_feature=feat-x`
- Expected: only the feat-x item returned

### TC-7: Park item — success
- Create item B1, `POST /api/backlog/B1/park` with `{ "park_reason": "revisit after v14" }`
- Expected: `200 OK`, body has `status: "parked"`, `park_reason: "revisit after v14"`

### TC-8: Park item — empty reason
- Create item B1, `POST /api/backlog/B1/park` with `{ "park_reason": "" }`
- Expected: `422 Unprocessable Entity`

### TC-9: Park promoted item
- Create B1, promote B1, then `POST /api/backlog/B1/park`
- Expected: `422 Unprocessable Entity`

### TC-10: Promote item — success
- Create B1, `POST /api/backlog/B1/promote` with `{ "slug": "auth-race-fix" }`
- Expected: `200 OK`, body has `status: "promoted"`, `promoted_to: "auth-race-fix"`

### TC-11: Promote already-promoted item
- Create B1, promote B1, then `POST /api/backlog/B1/promote` again
- Expected: `422 Unprocessable Entity`

### TC-12: Park unknown ID
- `POST /api/backlog/B99/park` with `{ "park_reason": "reason" }`
- Expected: `404 Not Found`

### TC-13: Promote unknown ID
- `POST /api/backlog/B99/promote` with `{ "slug": "some-feature" }`
- Expected: `404 Not Found`

### TC-14: Create with all optional fields
- `POST /api/backlog` with `title`, `kind`, `description`, `evidence`, `source_feature`
- Expected: `201`, all fields persisted in response

### TC-15: Promote with no slug
- `POST /api/backlog/B1/promote` with `{}` (empty body)
- Expected: `200`, item promoted, `promoted_to` absent or empty

## Regression Check

Run full integration test suite to confirm no existing routes are broken:

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-server 2>&1
```

All tests must pass.

## Lint Check

```bash
cargo clippy -p sdlc-server -- -D warnings 2>&1
```

Zero warnings allowed.

## Acceptance Criteria

- All 15 test cases pass
- Full integration suite passes (zero regressions)
- Clippy reports zero warnings for `sdlc-server`
- `routes/backlog.rs` uses `spawn_blocking` for all I/O
- No `unwrap()` in handler code
- `201` returned for create (not `200`)
- `404` returned for unknown IDs (not `422` or `500`)
- `422` returned for invalid transitions (not `400`)
