# Design: Page Title Injection

## Overview

This is a backend-only feature. The change is entirely in `crates/sdlc-server/src/embed.rs` with a minor update to `crates/sdlc-server/src/lib.rs` to pass state to the fallback handler, plus a cosmetic update to `frontend/index.html`.

## Current architecture

```
Request: GET /
  → Axum fallback → embed::static_handler(uri)
  → FrontendAssets::get("index.html")
  → Returns raw bytes with <title>Ponder</title>
```

`static_handler` currently has no access to `AppState` — it takes only `uri: axum::http::Uri`.

## Target architecture

```
Request: GET /
  → Axum fallback → embed::static_handler(State(app), uri)
  → FrontendAssets::get("index.html")
  → Compute title from app.root (reads .sdlc/state.yaml)
  → String-replace <title>...</title> in HTML bytes
  → Return modified HTML
```

## Axum fallback with state

Axum's `.fallback()` handler supports extractors including `State<T>`. The handler signature:

```rust
pub async fn static_handler(
    State(app): State<AppState>,
    uri: axum::http::Uri,
) -> Response
```

The route registration in `lib.rs` changes from:
```rust
.fallback(embed::static_handler)
```
to:
```rust
.fallback(embed::static_handler)
```
No change needed — Axum resolves extractors from the router's state automatically when `AppState` is present on the router (it is, via `.with_state(app_state)`).

## Title computation

```rust
fn compute_title(root: &std::path::Path) -> String {
    let project = sdlc_core::state::State::load(root)
        .ok()
        .map(|s| s.project)
        .unwrap_or_default();

    if project.is_empty() {
        "sdlc".to_string()
    } else {
        format!("sdlc \u{2014} {project}")  // em-dash: —
    }
}
```

## HTML injection

The `index.html` contains `<title>Ponder</title>` (or after the cosmetic update, `<title>sdlc</title>`). Replace tag contents with a simple string replacement:

```rust
fn inject_title(html: &str, title: &str) -> String {
    // Replace the entire <title>...</title> block
    if let Some(start) = html.find("<title>") {
        if let Some(end) = html[start..].find("</title>") {
            let tag_end = start + end + "</title>".len();
            return format!(
                "{}<title>{}</title>{}",
                &html[..start],
                title,
                &html[tag_end..]
            );
        }
    }
    // No <title> tag found — return unchanged
    html.to_string()
}
```

This approach:
- Does not require regex
- Does not require any additional dependencies
- Is safe — if parsing fails, returns original HTML
- Works with minified HTML (no whitespace dependency)

## Static asset pass-through

Only the SPA fallback path (serving `index.html`) applies title injection. Static assets (JS, CSS, fonts, images) are returned unchanged — the existing path for those does not apply injection.

```
/main.js, /assets/*, /vite.svg → returned as-is (no injection)
/ or /any-spa-path → index.html with title injected
```

## `frontend/index.html` update

Change `<title>Ponder</title>` to `<title>sdlc</title>` so the base/fallback value is consistent with the server fallback behavior. The server replaces this at runtime anyway.

## Error handling

- `State::load` failure → `ok()` → `None` → `project = ""` → title = `"sdlc"` (safe fallback)
- `String::from_utf8_lossy` → always succeeds (lossy conversion)
- Missing `<title>` tag → return original HTML unchanged

No panics. No `unwrap()`. No `SdlcError` propagation — this is a best-effort title injection on a UI route.

## Files changed

| File | Change |
|------|--------|
| `crates/sdlc-server/src/embed.rs` | Add `State<AppState>` parameter, add `compute_title` and `inject_title` helpers |
| `frontend/index.html` | Change `<title>Ponder</title>` to `<title>sdlc</title>` |

No changes needed to `lib.rs` — Axum's fallback handler already resolves state automatically.
