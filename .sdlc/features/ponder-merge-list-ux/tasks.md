# Tasks: ponder-merge-list-ux

## Task 1: Add merged_into and merged_from fields to PonderEntry
Add `merged_into: Option<String>` and `merged_from: Vec<String>` to `PonderEntry` struct in `crates/sdlc-core/src/ponder.rs` with serde defaults. Add unit test for serde roundtrip.

## Task 2: CLI ponder list -- filter merged entries and add --all flag
Update `PonderSubcommand::List` to accept `--all` flag. In `fn list()`, filter out merged entries by default. When merged entries are shown with `--all`, display status as `parked -> <target>`. Include `merged_into` in JSON output.

## Task 3: CLI ponder show -- redirect banner for merged entries
In `fn show()`, when `merged_into` is set, print a redirect banner before the normal output. Include `merged_into` and `merged_from` in JSON output.

## Task 4: REST GET /api/roadmap -- filter merged entries and add ?all query param
Add `ListPondersQuery` struct with `all` bool param. Filter out merged entries by default. Include `merged_into` and `merged_from` in each entry's JSON response.

## Task 5: REST GET /api/roadmap/:slug -- include merge fields and redirect banner
Include `merged_into`, `merged_from`, and `redirect_banner` fields in the show response JSON.

## Task 6: Frontend types and API client updates
Add `merged_into` and `merged_from` to `PonderSummary` and `PonderDetail` TypeScript interfaces. Update `getRoadmap` API call to accept optional `all` parameter.

## Task 7: Frontend PonderPage -- filter merged entries and add toggle
Filter merged entries from default list view. Add "Show merged" toggle. Render merged entries with reduced opacity and arrow indicator linking to target.

## Task 8: Frontend detail view -- redirect banner for merged entries
When viewing a merged entry, display an info banner at the top with a link to the target entry.
