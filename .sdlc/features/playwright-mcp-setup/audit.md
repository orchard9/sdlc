# Security Audit: playwright-mcp-setup

## Scope

This audit covers:
1. `.mcp.json` — new MCP server registration file at the project root.
2. Changes to `start_milestone_uat` in `runs.rs` — Playwright MCP tools added to UAT agent's `allowed_tools` and `mcp_servers`.
3. CLAUDE.md documentation update.

## Threat Model

The attack surface introduced is:
- A new subprocess (`npx @playwright/mcp@latest`) that is spawned by the Claude agent when UAT runs are triggered.
- Browser automation capability given to the UAT Claude agent.

## Security Analysis

### 1. Supply chain — `npx @playwright/mcp@latest`

**Risk level: Low.**

- `@playwright/mcp` is maintained by Microsoft as part of the Playwright project, one of the most widely used browser automation libraries. It has an established security track record.
- Using `@latest` introduces version drift risk. However, the package is not part of the build or production runtime — it is only invoked by the UAT agent at test time, in a developer or CI environment.
- Mitigation: If pinning is required for compliance, replace `@playwright/mcp@latest` with a specific version (e.g., `@playwright/mcp@0.1.0`) in both `.mcp.json` and `runs.rs`.

### 2. Browser sandbox

**Risk level: Low.**

- Playwright runs a browser in a sandboxed subprocess. It does not have elevated OS privileges.
- The UAT agent's browser cannot access the host filesystem directly — it can only interact with pages the agent navigates to.
- Playwright's default mode (no `--no-sandbox`) maintains Chromium's sandbox.

### 3. Scope of UAT agent capabilities

**Risk level: Low.**

- Playwright tools are added **only** to the UAT agent (`start_milestone_uat`). All other agent endpoints are unchanged.
- The UAT agent already has `Bash`, `Read`, `Write`, `Edit` from `sdlc_query_options`. Adding browser tools does not expand its OS-level capabilities.
- The agent runs in the same trust context as the existing sdlc agent infrastructure — inside the developer's or CI machine, not exposed to the internet.

### 4. `.mcp.json` exposure

**Risk level: Negligible.**

- `.mcp.json` contains only the command to start the Playwright MCP server. It contains no credentials, tokens, or secrets.
- The file is committed to the repository. Its content is safe to be public.

### 5. Prompt injection via browser content

**Risk level: Medium (inherent to browser automation, not new here).**

- Any Claude agent that navigates web pages could potentially encounter prompt injection attempts embedded in page content.
- This is a general risk for any agentic browser automation, not unique to this feature.
- Mitigation: The UAT agent's scope is limited to acceptance test scenarios (reading specific pages, not arbitrary browsing). The risk is bounded.

## Findings

| ID | Severity | Finding | Recommendation |
|---|---|---|---|
| A1 | Low | `@latest` version tag for `@playwright/mcp` | Consider pinning to a specific version for CI reproducibility |
| A2 | Medium (generic) | Browser automation agents can encounter prompt injection via page content | Acceptable for acceptance testing in controlled environments; document for operators |

## Conclusion

No blocking security issues. The change introduces standard browser automation tooling in a well-scoped, development/CI-only context. The two findings are informational — A1 is a standard trade-off for npm-based tooling, and A2 is inherent to all browser-using agents.

**Audit result: PASS with informational findings.**
