# QA Plan: page-title-fix

## Automated checks

- `SDLC_NO_NPM=1 cargo test --all` — all existing tests pass
- `cargo clippy --all -- -D warnings` — no clippy warnings

## Unit tests in embed.rs

Add tests for the helper functions:

### `inject_title` tests
- Input with `<title>Ponder</title>` → replaces with new title
- Input with no `<title>` tag → returns HTML unchanged
- Input with `<title>sdlc</title>` → replaces correctly

### `compute_title` tests
- With a valid `.sdlc/state.yaml` containing `project: myapp` → returns `"sdlc — myapp"`
- With an empty project name → returns `"sdlc"`
- With missing state.yaml → returns `"sdlc"`

## Manual verification (local server)

1. Start `sdlc ui` against a project with `project: myapp` in `.sdlc/state.yaml`
2. Open browser and verify tab title reads `sdlc — myapp`
3. Navigate to a SPA route (e.g., `/features`) and reload — verify title persists
4. Verify static assets still load (check browser console for 404s)

## Edge cases

- Hub mode: project name empty → title = `sdlc` (acceptable fallback)
- State.yaml parse error → title = `sdlc` (graceful degradation)
- Multiple tab instances with different projects — each shows correct name
