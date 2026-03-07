# Security Audit: Tunnel Preflight Check

## Scope

New `GET /api/tunnel/preflight` endpoint + frontend changes in `NetworkPage.tsx`.

## Findings

### A1: Endpoint exposes filesystem paths [ACCEPTED]
The preflight response includes `path` (e.g. `/opt/homebrew/bin/orch-tunnel`) and checked location paths. This is diagnostic information about the server's local environment. **Risk: low.** The endpoint is only accessible to authenticated users (same auth gate as all `/api/*` routes). In local mode there is no auth, but the user is already on the same machine. In tunnel mode, the tunnel auth middleware gates access. Filesystem paths of system binaries are not sensitive.

### A2: Version string reflects `--version` output [ACCEPTED]
The version is captured by executing `<binary> --version` and returning stdout. The binary path comes from `which::which` or known fallback locations — not from user input. No command injection vector exists. The output is trimmed and returned as a string. **Risk: none.**

### A3: No auth bypass [ACCEPTED]
The route is registered inside the standard axum router, so it inherits the same auth middleware as other `/api/*` routes. No special bypass was added. **Risk: none.**

### A4: `spawn_blocking` prevents DoS via slow shells [ACCEPTED]
`read_login_shell_path()` has a 3-second timeout, and the entire `check_orch_tunnel()` call is wrapped in `spawn_blocking`. A malicious or broken shell config cannot block the tokio runtime. **Risk: none.**

### A5: Frontend graceful degradation [ACCEPTED]
If the preflight endpoint fails (network error, server error), the frontend falls back to allowing tunnel start (old behavior). This means a broken preflight cannot be weaponized to permanently disable tunnels. **Risk: none.**

## Verdict

No security issues found. The endpoint is read-only, inherits existing auth, and exposes only non-sensitive diagnostic data about binary availability. All findings accepted with documented rationale.
