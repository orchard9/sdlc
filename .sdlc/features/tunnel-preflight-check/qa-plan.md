# QA Plan: Tunnel Preflight Check

## Unit Tests

### UT-1: `check_orch_tunnel()` returns valid PreflightResult
- Call `check_orch_tunnel()` — result should have `installed` boolean, `checked_locations` non-empty.
- If orch-tunnel is on the test machine PATH, `installed` should be `true` with a path and version.
- `checked_locations` should always contain at least the "process PATH" entry.

### UT-2: `tunnel_preflight` route returns JSON
- Call `GET /api/tunnel/preflight` via the test app.
- Assert 200 status with JSON body containing `installed`, `checked_locations` fields.

### UT-3: PreflightResult serialization
- Construct a `PreflightResult` with known values, serialize to JSON, verify shape.
- Construct with `installed: false`, verify `install_hint` is present.

## Integration Tests

### IT-1: Network page shows preflight warning when not installed
- Mock `/api/tunnel/preflight` to return `{ installed: false, ... }`.
- Verify "Start tunnel" buttons are disabled.
- Verify warning banner with install instructions is visible.

### IT-2: Network page enables buttons when installed
- Mock `/api/tunnel/preflight` to return `{ installed: true, version: "0.3.1", ... }`.
- Verify "Start tunnel" buttons are enabled.
- Verify version info is shown.

### IT-3: Preflight loading state
- Delay the preflight response.
- Verify buttons show loading/disabled state during the check.

## Manual Verification

### MV-1: Real binary check
- With orch-tunnel installed: open Network page, confirm buttons are enabled and version is shown.
- Temporarily rename orch-tunnel binary: refresh page, confirm buttons are disabled with install instructions.
- Rename back: refresh page, confirm buttons re-enable.
