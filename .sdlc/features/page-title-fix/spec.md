# Spec: Page Title — sdlc — {project-name} in Browser Tabs

## Problem

The current `index.html` has a hardcoded `<title>Ponder</title>`. Browser tabs always show "Ponder" regardless of which project the server is serving. This makes it impossible to distinguish between multiple open sdlc instances.

## Goal

Inject a dynamic page title at serve time so browser tabs read:
- `sdlc — {project-name}` for a normal project instance (e.g., `sdlc — myapp`)
- `sdlc hub` for a hub mode instance (future, but handled via fallback)
- `sdlc` as a safe fallback when the project name is unavailable

## Implementation

### Where the change goes

The static handler in `crates/sdlc-server/src/embed.rs` serves the embedded `index.html`. Currently it returns the raw embedded bytes. The SPA fallback path (line 25-33) must be modified to:

1. Read the embedded `index.html` bytes
2. Convert to a UTF-8 string
3. Replace `<title>` tag contents with the dynamic title
4. Return the modified HTML with `Content-Type: text/html`

The handler needs access to `AppState` (already on `app_state.root`) to read the project name from `.sdlc/state.yaml` at request time.

### Title logic

```
project_name = State::load(&app.root).map(|s| s.project).unwrap_or_default()

if project_name is empty:
    title = "sdlc"
else:
    title = format!("sdlc — {project_name}")
```

### Replacement strategy

The `index.html` contains `<title>Ponder</title>`. Replace this pattern via a string replacement:

```rust
let html = String::from_utf8_lossy(&content.data).into_owned();
let title = compute_title(&app.root);
let html = replace_title_tag(&html, &title);
```

Where `replace_title_tag` replaces the contents between `<title>` and `</title>` with the computed value. If no `<title>` tag is found, return the HTML unchanged (safe fallback).

### Function signature

The `static_handler` must be updated to accept `State<AppState>` so it can read `app.root`:

```rust
pub async fn static_handler(
    State(app): State<AppState>,
    uri: axum::http::Uri,
) -> Response
```

The existing route registration in `lib.rs` uses `.fallback(proxy::proxy_handler)` rather than registering `static_handler` as a named route — check how `static_handler` is currently called and wire `AppState` accordingly.

### No caching of the title

State is read fresh at each `index.html` request. This is appropriate — `index.html` is fetched rarely (only on full page load), and reading a small YAML file is cheap. No in-memory caching needed.

### Hub mode

The architecture decision calls for `sdlc hub` as the hub title. Hub mode is a future concern; for now, if `state.project` is empty or state.yaml cannot be read, the title falls back to `sdlc`. Hub mode can be added later by passing a flag at startup.

## Files to change

1. `crates/sdlc-server/src/embed.rs` — update `static_handler` to accept `AppState`, inject title
2. `crates/sdlc-server/src/lib.rs` — update the fallback route to pass state (if needed by Axum's fallback API)
3. `frontend/index.html` — update `<title>Ponder</title>` to `<title>sdlc</title>` as the base value (the server will replace it at runtime anyway, but the base value should match the fallback)

## Acceptance criteria

- Browser tab shows `sdlc — myproject` when serving a project with name `myproject`
- Browser tab shows `sdlc` when `.sdlc/state.yaml` is missing or project name is empty
- No `unwrap()` in the implementation — use `unwrap_or` / `unwrap_or_else` / `ok()`
- Existing `static_handler` tests (if any) continue to pass
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
