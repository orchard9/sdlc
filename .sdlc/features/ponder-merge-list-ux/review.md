# Review: ponder-merge-list-ux

## Summary

This feature adds UX for merged ponder entries across all layers: data model, CLI, REST API, and frontend. Merged entries are hidden by default and show redirect banners when viewed.

## Changes Reviewed

### Data Model (crates/sdlc-core/src/ponder.rs)
- `merged_into: Option<String>` and `merged_from: Vec<String>` fields added to `PonderEntry`
- Both use `serde(default)` for backward compatibility
- Constructor initializes to `None` / `Vec::new()`
- Serde roundtrip tests exist and pass
- **Verdict**: Clean, backward-compatible addition

### CLI (crates/sdlc-cli/src/cmd/ponder.rs)
- `sdlc ponder list`: `--all` flag added, merged entries filtered by default
- Status column shows `parked -> <target>` for merged entries when `--all` is set
- JSON output includes `merged_into` field
- `sdlc ponder show`: redirect banner printed before normal output when `merged_into` is set
- JSON output includes both `merged_into` and `merged_from`
- **Verdict**: Clean implementation, follows existing patterns

### REST API (crates/sdlc-server/src/routes/roadmap.rs)
- `GET /api/roadmap`: accepts `?all=true` query param via `ListPondersQuery` struct
- Filters merged entries by default using `.filter()` on the iterator
- Both `merged_into` and `merged_from` included in list response JSON
- `GET /api/roadmap/:slug`: includes `merged_into`, `merged_from`, and `redirect_banner`
- **Verdict**: Clean, additive API change

### Server Error Handling (crates/sdlc-server/src/error.rs)
- Added `PonderMergeError` match arm mapping to `422 UNPROCESSABLE_ENTITY`
- Required for exhaustiveness with the new error variant from ponder-merge-cli
- **Verdict**: Correct status code mapping

### Frontend Types (frontend/src/lib/types.ts)
- `PonderSummary`: `merged_into: string | null`, `merged_from: string[]` added
- `PonderDetail`: same fields plus `redirect_banner: string | null`
- **Verdict**: Correctly mirrors server response

### Frontend API Client (frontend/src/api/client.ts)
- `getRoadmap` accepts optional `all?: boolean` parameter
- Appends `?all=true` query string when set
- **Verdict**: Simple, correct change

### Frontend PonderPage (frontend/src/pages/PonderPage.tsx)
- `showMerged` state toggles merged entry visibility
- `load()` passes `showMerged` to `api.getRoadmap(showMerged)`
- Eye/EyeOff toggle icon in header toolbar
- `EntryRow`: merged entries render with `opacity-50` and arrow indicator showing target
- Non-merged entries render normally (sessions, team, tags)
- Detail view: blue info banner with link to target entry displayed above header when `merged_into` is set
- **Verdict**: Clean UI with good visual hierarchy

## Findings

1. **No issues found.** All changes are additive and backward-compatible. The implementation correctly follows existing patterns across all layers.

## Build Verification

- `SDLC_NO_NPM=1 cargo test --all` -- all tests pass
- `cargo clippy --all -- -D warnings` -- clean
- TypeScript compilation (`npx tsc --noEmit`) -- clean
