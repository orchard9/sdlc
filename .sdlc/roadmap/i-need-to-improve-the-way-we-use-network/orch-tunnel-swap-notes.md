# orch-tunnel swap — implementation notes

## Tunnel.rs changes
- `find_cloudflared()` → `find_orch_tunnel()` using `which::which("orch-tunnel")`
- Spawn args: `orch-tunnel http <port> --name <project-name>` (gets name from config.yaml project.name)
- URL extraction: orch-tunnel outputs to stdout; URL pattern is `https://{name}.tunnel.threesix.ai`
- Error messages: update to reference `brew install orch-tunnel` not cloudflared
- TunnelError variants: NotFound, Timeout, ExitedEarly — same structure, different messages

## URL stability bonus
Named tunnels (`--name sdlc`) mean the URL is stable across restarts. This is a significant improvement
over cloudflared quick tunnels which generate random hostnames every start.

## Default behavior change
`sdlc ui` should start tunnel by default (orch-tunnel, not cloudflared).
`--no-tunnel` flag to opt out.
If orch-tunnel binary not found or fails to connect: warn and continue in local-only mode.

## Files to change
- crates/sdlc-server/src/tunnel.rs — main swap
- crates/sdlc-cli/src/cmd/ui.rs — flip --tunnel to --no-tunnel logic
- crates/sdlc-server/src/routes/tunnel.rs — update error messages
- docs/orch-tunnel-reference.md — update default behavior docs