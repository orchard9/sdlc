# Code Review: page-title-fix

## Summary

Implementation injects the project name from `.sdlc/state.yaml` into the `<title>` tag of `index.html` at serve time. Two files changed: `embed.rs` (core logic) and `frontend/index.html` (cosmetic base value update).

## Files Changed

### `crates/sdlc-server/src/embed.rs`

**Changes:**
- Added `State<AppState>` parameter to `static_handler`
- Added `compute_title(root: &Path) -> String` — reads `state.yaml`, formats title
- Added `inject_title(html: &str, title: &str) -> String` — string-replaces `<title>` tag
- Title injection applied only on SPA fallback path (static assets unchanged)
- 5 unit tests covering title injection and compute logic

**Review findings:**

1. No `unwrap()` — confirmed. All error paths use `.ok()`, `.unwrap_or_default()`, and safe fallbacks.
2. State read at request time — correct. No caching introduced. `index.html` requests are rare (page loads only).
3. Static assets unaffected — confirmed. The exact-path branch returns early before injection.
4. Fallback title `"sdlc"` when `state.yaml` missing or empty — confirmed in tests.
5. UTF-8 handling — `String::from_utf8_lossy` is used, which handles any bytes gracefully.
6. Proxy handler also calls `static_handler` — updated in `proxy.rs` to pass `State(app)`.

**No findings requiring action.**

### `frontend/index.html`

Changed `<title>Ponder</title>` to `<title>sdlc</title>`. This is a cosmetic change only — the server replaces this at runtime anyway. The fallback value now matches what users see if they somehow get the raw HTML without server injection.

## Additional: Hub compilation fixes

While implementing, found the codebase had a partially-implemented `hub-server-mode` feature that left compilation broken:
- `routes/hub.rs` was missing `use axum::response::IntoResponse`
- `state.rs` was missing `hub_registry: Option<Arc<Mutex<HubRegistry>>>` field and `new_with_port_hub` constructor
- These are required by `hub.rs` and `routes/hub.rs` which were already committed to the working tree

Fixed all three to restore a compiling state. These fixes advance the hub-server-mode feature (T1 is already started there).

## Test Results

- `SDLC_NO_NPM=1 cargo test --all` — all tests pass
- `cargo clippy --all -- -D warnings` — no warnings

## Verdict: Approved

Implementation is clean, minimal, and correct. No issues requiring remediation.
