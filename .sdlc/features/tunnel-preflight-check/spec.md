# Spec: Tunnel Preflight Check

## Problem

When a user clicks "Start tunnel" in the Network page and `orch-tunnel` is not installed (or not discoverable on the process PATH), the request fails with a raw error message after a noticeable delay. The user has no advance warning that the binary is missing, and the error message is not actionable — it does not tell them exactly how to fix the problem.

## Solution

Add a preflight check that runs **before** the user can attempt to start a tunnel. This consists of three parts:

### 1. `GET /api/tunnel/preflight` endpoint

A new endpoint that checks whether `orch-tunnel` is reachable and returns structured JSON:

```json
{
  "installed": true,
  "path": "/opt/homebrew/bin/orch-tunnel",
  "version": "0.3.1",
  "source": "process_path",
  "checked_locations": [
    { "location": "process PATH", "found": true }
  ]
}
```

When not found:
```json
{
  "installed": false,
  "path": null,
  "version": null,
  "source": null,
  "checked_locations": [
    { "location": "process PATH", "found": false },
    { "location": "/opt/homebrew/bin/orch-tunnel", "found": false },
    { "location": "/usr/local/bin/orch-tunnel", "found": false },
    { "location": "~/.cargo/bin/orch-tunnel", "found": false }
  ],
  "install_hint": "brew install orch-tunnel  OR  gh release download --repo orchard9/tunnel"
}
```

The endpoint calls the discovery logic from `tunnel.rs` (enhanced by the sibling feature `robust-orch-tunnel-lookup`). If that feature has not landed yet, it uses the existing `find_orch_tunnel()` and additionally runs `<binary> --version` to capture the version string.

### 2. UI gating on the Network page

Both the Ponder Tunnel and App Tunnel sections call `GET /api/tunnel/preflight` on mount. When `installed` is `false`:

- The "Start tunnel" button is **disabled** with a tooltip/label explaining why.
- An inline banner replaces the generic `TunnelDisclosure` with specific install instructions and the list of locations that were checked.

When `installed` is `true`, the UI behaves exactly as it does today — no extra friction.

### 3. Actionable error formatting

If a tunnel start fails despite a passing preflight (race condition, binary crashes, network issue), the error message displayed in the UI is reformatted to be actionable:

| Error type | User-facing message |
|---|---|
| Binary not found | "orch-tunnel is not installed. Install with: `brew install orch-tunnel`" |
| Timeout | "orch-tunnel did not connect within N seconds. Check your network and try again." |
| Unexpected exit | "orch-tunnel exited unexpectedly. Try running manually: `orch-tunnel http <port> --name <name>`" |

These messages are already present in `TunnelError` variants but are returned as raw strings. The preflight endpoint provides structured data so the frontend can render them with proper formatting (code blocks, copy buttons).

## Scope boundaries

- This feature does **not** change how `find_orch_tunnel()` discovers the binary — that is `robust-orch-tunnel-lookup`.
- This feature does **not** add a CLI command — the preflight is a server-side HTTP endpoint consumed by the frontend.
- The preflight check is **read-only** — it never starts or modifies tunnel state.

## Acceptance criteria

1. `GET /api/tunnel/preflight` returns correct JSON when orch-tunnel is installed and when it is not.
2. The Network page disables both "Start tunnel" buttons when preflight reports `installed: false`.
3. When orch-tunnel is missing, the UI shows install instructions with the checked locations.
4. When orch-tunnel is present, the UI shows no extra friction — buttons work as before.
5. Error messages from failed tunnel starts are formatted with actionable instructions.
6. The preflight endpoint has a unit test covering both the found and not-found cases.
