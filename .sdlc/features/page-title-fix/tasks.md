# Tasks: page-title-fix

## Task 1: Update embed.rs — inject dynamic title into index.html

Update `crates/sdlc-server/src/embed.rs`:
- Add `State<AppState>` parameter to `static_handler`
- Add `compute_title(root: &Path) -> String` helper that reads `.sdlc/state.yaml` and returns `"sdlc — {project}"` or `"sdlc"` fallback
- Add `inject_title(html: &str, title: &str) -> String` helper that replaces `<title>...</title>` content
- Apply injection only in the SPA fallback path (when serving `index.html`)
- Static assets (JS, CSS, images) return unchanged as before

## Task 2: Update frontend/index.html base title

Change `<title>Ponder</title>` to `<title>sdlc</title>` in `frontend/index.html` so the static fallback value is consistent with the server fallback behavior.

## Task 3: Verify build and tests pass

Run:
- `SDLC_NO_NPM=1 cargo test --all` — confirm no test failures
- `cargo clippy --all -- -D warnings` — confirm no warnings
