# Tasks: backlog-server

The feature has four sequential tasks. T1 creates the handler file, T2 registers the module, T3 registers the routes in the router, T4 writes integration tests. T1 must complete before T2 and T3, which can be done together. T4 depends on T1-T3.

## T1 — Create crates/sdlc-server/src/routes/backlog.rs with list, create, park, promote handlers

Implement four async Axum handlers in a new file `crates/sdlc-server/src/routes/backlog.rs`:

- `list_backlog` — `GET /api/backlog` with `?status` and `?source_feature` query params
- `create_backlog_item` — `POST /api/backlog` returning 201
- `park_backlog_item` — `POST /api/backlog/:id/park`
- `promote_backlog_item` — `POST /api/backlog/:id/promote`

Each handler wraps `BacklogStore` calls in `tokio::task::spawn_blocking`. All return `Result<Json<serde_json::Value>, AppError>`.

Status string query param parsed to `BacklogStatus` with match; unknown values return 400 via `SdlcError::InvalidSlug`.

`park_reason` missing from body returns 400 (Axum JSON extraction failure). Empty `park_reason` string returns 422 (BacklogStore validation).

Use `serde_json::to_value(&item)` to serialize `BacklogItem` to JSON value.

## T2 — Add pub mod backlog to routes/mod.rs

One-line addition at the end of the alphabetical module list in `crates/sdlc-server/src/routes/mod.rs`.

## T3 — Register GET /api/backlog, POST /api/backlog, POST /api/backlog/:id/park, POST /api/backlog/:id/promote in lib.rs

Add four `.route(...)` calls to `build_router_from_state` in `crates/sdlc-server/src/lib.rs`. Place them after the knowledge routes block, before investigations. Group as:

```rust
// Backlog
.route("/api/backlog", get(routes::backlog::list_backlog).post(routes::backlog::create_backlog_item))
.route("/api/backlog/{id}/park", post(routes::backlog::park_backlog_item))
.route("/api/backlog/{id}/promote", post(routes::backlog::promote_backlog_item))
```

## T4 — Write route integration tests

Add tests in `crates/sdlc-server/tests/integration.rs` covering:

1. `GET /api/backlog` on empty project returns `[]`
2. `POST /api/backlog` creates item with ID `B1`
3. `GET /api/backlog` after create returns the item
4. `GET /api/backlog?status=open` returns only open items
5. `GET /api/backlog?source_feature=feat-x` returns only matching items
6. `POST /api/backlog/{id}/park` transitions item to parked with reason
7. `POST /api/backlog/{id}/park` with empty `park_reason` returns 422
8. `POST /api/backlog/{id}/park` on promoted item returns 422
9. `POST /api/backlog/{id}/promote` transitions item to promoted
10. `POST /api/backlog/{id}/promote` already-promoted returns 422
11. `POST /api/backlog/B99/park` returns 404
12. `POST /api/backlog/B99/promote` returns 404
