# Security Audit: Fix Missing `_shared/agent.ts` in Tool Installation

## Surface Area

This change adds one new constant (`TOOL_SHARED_AGENT_TS`) to the init template system and registers it as a managed shared file written by `sdlc init`/`sdlc update`. No new routes, no new network calls, no new auth paths.

## Findings

### F1: `include_str!` path traversal risk — ACCEPTED (no risk)

**Concern:** `include_str!("../../../../../.sdlc/tools/_shared/agent.ts")` uses a relative path with `../` sequences.

**Assessment:** `include_str!` is a compile-time macro — the path is resolved at build time relative to the source file. There is no runtime path traversal. The macro will fail to compile if the file doesn't exist. No user input touches this path.

**Action:** Accept. No change needed.

### F2: Content of installed `agent.ts` — ACCEPTED

**Concern:** The installed file contains `fetch()` calls and `process.env` access.

**Assessment:** `agent.ts` is developer tooling, installed into `.sdlc/tools/_shared/` alongside the rest of the tool suite. It requires `SDLC_SERVER_URL` and `SDLC_AGENT_TOKEN` env vars (injected by the server per subprocess). No credentials are hardcoded. The file is unchanged from what was already committed to the repo and already reachable via `sdlc tool run`.

**Action:** Accept. This is by design.

### F3: Overwrite semantics — ACCEPTED

**Concern:** The file is always overwritten on `sdlc update`, meaning a user who customized it would lose changes.

**Assessment:** All other shared files (`types.ts`, `log.ts`, `config.ts`, `runtime.ts`) use the same always-overwrite semantics. Users are expected to put customizations in tool-specific `config.yaml` files, not in `_shared/`. This is consistent with existing behavior.

**Action:** Accept. Documented pattern, no change needed.

## Verdict

No security findings require remediation. The change is a two-line Rust fix with no new attack surface.
