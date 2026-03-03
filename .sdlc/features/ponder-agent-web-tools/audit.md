# Security Audit: ponder-agent-web-tools

## Scope

This audit covers the changes in `crates/sdlc-server/src/routes/runs.rs`:
- New `sdlc_ponder_query_options` helper function
- Updated `start_ponder_chat` to use it
- Updated ponder session prompt to mention web tools

## Surface Analysis

### Tool Expansion

**WebSearch** — built-in Claude tool, handled by the Claude agent runtime. No new network code in this repository. The agent calls this tool; the Claude runtime executes it. No additional authentication, secrets, or API keys are introduced.

**WebFetch** — built-in Claude tool, same pattern as WebSearch. No new network code in this repository.

**Playwright MCP** — spawned as a subprocess via `npx @playwright/mcp@latest`. This is the identical configuration already used in `start_milestone_uat`. No new subprocess patterns are introduced.

### Authorization

Ponder sessions are already gated by the sdlc-server authentication layer (`auth.rs` — token/cookie gate, local bypass). Adding web tools to an already-authenticated session does not change who can initiate a session. The tool expansion only affects what the agent can do during an authenticated session.

### Information Disclosure

Web tools allow the agent to fetch external URLs and search the web. This is intentional. The agent operates under `PermissionMode::BypassPermissions` already (unchanged). The ponder agent already has full filesystem access via `Bash`, `Read`, `Write`, etc. Web access does not expand the trust boundary — any actor who can already start a ponder session could already exfiltrate data via other means.

### Supply Chain

The Playwright MCP package (`@playwright/mcp@latest`) is the same package pinned in the existing UAT path. No new transitive dependencies are introduced by this feature — the package is already referenced in `start_milestone_uat`. Risk profile is unchanged from the existing UAT surface.

### Prompt Injection

The ponder prompt now includes tool names and brief descriptions. These are static strings with no user-controlled interpolation except for `{slug}` (validated by `validate_slug()` before reaching the prompt). The web tools mention is not exploitable via prompt injection.

### Resource Consumption

WebSearch and WebFetch are bounded by the agent's turn limit (`max_turns: 100`). Playwright browser instances are bounded by session lifecycle. No new unbounded resource consumption patterns are introduced.

## Findings

None. The changes are additive and low-risk:
- No new authentication surface
- No new secrets or credentials
- No new subprocess patterns (Playwright already used in UAT)
- No user-controlled input in the new code paths
- Supply chain risk unchanged

## Verdict

**APPROVED** — no security concerns.
