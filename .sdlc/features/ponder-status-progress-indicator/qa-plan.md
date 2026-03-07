# QA Plan: Ponder Status Step Indicator

## Q1: Full variant renders correctly for each status
- Navigate to a ponder entry detail view with status `exploring`
- Verify: "Exploring" is highlighted violet, "Converging" and "Committed" are dimmed
- Change status to `converging` — verify "Exploring" shows check mark, "Converging" highlighted amber
- Change status to `committed` — verify "Exploring" and "Converging" show check marks, "Committed" highlighted emerald
- Change status to `parked` — verify all steps are muted, "Parked" badge appears

## Q2: Compact variant renders correctly in list rows
- Navigate to the roadmap/ponder list page
- Verify each entry shows a compact dot indicator matching its status
- Exploring: first dot filled violet with ring, others hollow
- Converging: first dot filled, second amber with ring, third hollow
- Committed: all three dots filled
- Parked: all dots muted

## Q3: No regressions on existing ponder functionality
- Status change modal still works to transition between statuses
- Tab filtering (All/Exploring/Converging/Committed/Parked) still works
- Entry detail view still shows all other content (team, artifacts, sessions)

## Q4: Build verification
- `cd frontend && npm run build` completes without errors
- `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings` passes
