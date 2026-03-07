# QA Plan: Milestone Detail — MilestonePreparePanel Integration

## Test Scenarios

### 1. Panel renders on milestone detail
- Navigate to a milestone detail page that has an active wave plan
- Verify the MilestonePreparePanel appears between header and features list
- Confirm progress bar, wave plan, and action buttons are visible

### 2. Panel hides when no data
- Navigate to a milestone with no wave plan and not in verifying state
- Confirm the panel renders nothing (no empty box or placeholder)

### 3. Verifying state shows Run UAT
- Navigate to a milestone where all features are released
- Confirm the panel shows "All features released" with a Run UAT button

### 4. SSE auto-refresh
- Trigger a run completion event
- Confirm the panel refreshes without manual reload

### 5. Build passes
- `SDLC_NO_NPM=1 cargo test --all` passes
- `cargo clippy --all -- -D warnings` passes
