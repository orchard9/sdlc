# QA Plan: ponder-merge-list-ux

## Test 1: Data model serde roundtrip
- Create a PonderEntry with `merged_into: Some("target")` and `merged_from: vec!["source"]`
- Serialize to YAML, deserialize back
- Verify fields preserved
- Also test deserialization of YAML without these fields (backward compat): both default to None/empty

## Test 2: CLI ponder list hides merged entries
- Create two ponder entries: "entry-a" and "entry-b"
- Set entry-a's `merged_into` to "entry-b" (via direct manifest edit in test fixture)
- Run `sdlc ponder list`: verify entry-a is NOT in output
- Run `sdlc ponder list --all`: verify entry-a IS in output with status indicator

## Test 3: CLI ponder show redirect banner
- Load a ponder entry with `merged_into` set
- Run `sdlc ponder show <slug>`: verify redirect banner is printed
- Run `sdlc ponder show <slug> --json`: verify `merged_into` field in JSON

## Test 4: REST list filters merged entries
- `GET /api/roadmap`: verify merged entries excluded from response
- `GET /api/roadmap?all=true`: verify merged entries included with `merged_into` field populated

## Test 5: REST show includes merge fields
- `GET /api/roadmap/:slug` for a merged entry: verify `merged_into`, `merged_from`, `redirect_banner` fields present
- `GET /api/roadmap/:slug` for a non-merged entry: verify `merged_into` is null, `redirect_banner` is null

## Test 6: Frontend types compile
- Verify TypeScript compilation succeeds with new fields on PonderSummary and PonderDetail

## Test 7: Frontend list filtering
- PonderPage filters out entries with `merged_into` set by default
- "Show merged" toggle includes them with visual indicator

## Test 8: Frontend detail banner
- Viewing a merged entry displays info banner with link to target
- Viewing a non-merged entry shows no banner

## Build verification
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
- `cd frontend && npm run build` succeeds
