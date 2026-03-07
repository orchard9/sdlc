# Code Review: Tunnel Preflight Check

## Summary

The implementation adds a `GET /api/tunnel/preflight` endpoint and gates the Network page's tunnel buttons on whether `orch-tunnel` is discoverable. The backend reuses the existing `check_orch_tunnel()` function (added by the sibling `robust-orch-tunnel-lookup` feature) and wraps it in a route handler with `spawn_blocking`. The frontend lifts preflight state to the page level and shares it between both tunnel sections.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-server/src/routes/tunnel.rs` | Added `PreflightResponse`, `tunnel_preflight` handler, 2 tests |
| `crates/sdlc-server/src/lib.rs` | Registered `/api/tunnel/preflight` route |
| `frontend/src/lib/types.ts` | Added `TunnelPreflightResult`, `CheckedLocation` types |
| `frontend/src/api/client.ts` | Added `getTunnelPreflight()` method |
| `frontend/src/pages/NetworkPage.tsx` | Rewrote to add preflight gating, `PreflightWarning` component, `TunnelDisclosure` now shows version info |

## Findings

### F1: Graceful degradation on preflight failure [ACCEPTED]
The frontend catches preflight API errors and falls back to `installed: true`, preserving the old behavior. This is the right choice — a broken preflight endpoint should not block tunnel usage.

### F2: `TunnelToggleButton` now accepts `disabled` prop [ACCEPTED]
The button component gained an optional `disabled` prop separate from `toggling`. When the tunnel is already active (stop case), `disabled` is not applied — only the start action is gated. Correct behavior.

### F3: `PreflightResponse` uses `#[serde(flatten)]` [ACCEPTED]
The response flattens `TunnelCheckResult` fields to the top level, so `installed`, `path`, `version`, `checked` etc. appear directly in the JSON alongside `install_hint`. Test `preflight_response_serializes_flat` verifies this. Good API ergonomics.

### F4: Single preflight fetch shared between sections [ACCEPTED]
Preflight is fetched once at the `NetworkPage` level and passed as props. This avoids duplicate requests and ensures consistent state between the two tunnel sections.

### F5: `spawn_blocking` for synchronous check [ACCEPTED]
`check_orch_tunnel()` does blocking I/O (`which`, `Command::output`, `is_file`). The handler correctly wraps it in `spawn_blocking` to avoid blocking the tokio runtime.

## Verdict

All findings are accepted. The implementation is clean, well-tested, and follows established patterns. No issues to fix or track.
