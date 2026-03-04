# Security Audit: Knowledge research — web-capable agent with rewritten prompt

## Scope

Single function change in `crates/sdlc-server/src/routes/knowledge.rs`:
1. Adding `WebSearch` and `WebFetch` to the allowed tools list for the research agent.
2. Rewriting the research agent prompt.

## Security surface

### External HTTP access via WebSearch/WebFetch

**Finding:** The research agent can now make outbound HTTP requests to arbitrary URLs.

**Analysis:**
- This is intentional and consistent with the existing `ask_knowledge` endpoint (same file), which already has WebSearch/WebFetch enabled.
- The `POST /api/knowledge/:slug/research` endpoint is protected by the same auth middleware as all other `/api/` routes (token/cookie gate, local bypass — see `auth.rs`). An unauthenticated caller cannot trigger the research endpoint.
- The agent runs in a sandboxed subprocess. External HTTP access goes through the Claude SDK's tool invocation layer, not a raw `reqwest`/`curl` call that could be SSRF-injected.
- The slug and topic values are interpolated into the prompt string (not into SQL or shell commands), so SQL injection and shell injection are not applicable.

**Action:** Accept — consistent with existing pattern; no new attack surface.

### Prompt injection via `topic` parameter

**Finding:** The `body.topic` value is interpolated directly into the prompt string.

**Analysis:**
- The topic comes from a JSON body field. A malicious caller could provide a crafted topic string to manipulate agent behavior.
- This is the same risk that already exists in the current code and in `ask_knowledge` (which also interpolates a user-supplied `question` into the prompt).
- The endpoint is authenticated. The agent has no ability to exfiltrate secrets beyond what it can read in the project directory (which is the same level of access the authenticated user already has).

**Action:** Accept — pre-existing pattern, within the authenticated threat model. Track as a general prompt injection hardening opportunity if desired.

### No new persistent state changes

The change does not modify the data model, add new routes, or change how results are stored. The agent writes to `content.md` and logs a session — both pre-existing behaviors.

## Verdict

No new security risks introduced. The change is a straightforward extension of the existing `ask_knowledge` pattern to the `research_knowledge` endpoint.
