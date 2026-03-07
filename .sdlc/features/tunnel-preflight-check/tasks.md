# Tasks: Tunnel Preflight Check

## T1: Add `check_orch_tunnel()` and types to `tunnel.rs`

Add `PreflightResult`, `CheckedLocation` structs and `check_orch_tunnel()` function to `crates/sdlc-server/src/tunnel.rs`. The function tries `which::which`, checks fallback paths, runs `--version` on found binary, and returns a complete `PreflightResult`.

## T2: Add `GET /api/tunnel/preflight` route

Add `tunnel_preflight` handler in `crates/sdlc-server/src/routes/tunnel.rs` that calls `check_orch_tunnel()` via `spawn_blocking`. Register the route in `crates/sdlc-server/src/lib.rs`.

## T3: Add preflight API client method and TypeScript types

Add `PreflightResult` type to `frontend/src/lib/types.ts` and `getTunnelPreflight()` to the API client.

## T4: Gate tunnel buttons on preflight status in NetworkPage

Both `SdlcTunnelSection` and `AppTunnelSection` call preflight on mount. When not installed: disable button, show warning banner with install instructions and checked locations. When installed: show version info subtly, buttons enabled.

## T5: Add unit tests for `check_orch_tunnel()` and preflight route

Test that `check_orch_tunnel()` returns a valid `PreflightResult` (installed or not). Test the route handler returns JSON with expected shape.
