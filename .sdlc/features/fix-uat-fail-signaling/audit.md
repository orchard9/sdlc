# Security Audit: UAT Fail Endpoint and Skill Template for Explicit Failure Signaling

## Surface Area

This feature adds:
- `POST /api/milestone/{slug}/uat/fail` — a new unauthenticated (local-only) HTTP endpoint
- One new SSE variant emitted on the `milestone_uat` channel
- A TypeScript union type extension (no runtime behavior)
- Skill template text changes (no runtime code)

## Threat Analysis

### Authentication and Authorization

**Finding:** The new endpoint follows the same auth model as all other `sdlc-server` routes. The server's `auth.rs` middleware applies a token/cookie gate to all remote (tunneled) requests and bypasses the gate for local connections (localhost:7777). This is the intended design — the server is a local developer tool, not a public service.

**Risk:** None. The endpoint cannot be called remotely without a valid tunnel token. Locally, any process that can reach localhost:7777 can call it — this is the same posture as every other mutation endpoint on the server.

**Action:** Accept. No change needed.

### Input Validation

**Finding:** The handler calls `validate_slug(&slug)` before any other work. `validate_slug` delegates to `sdlc_core::paths::validate_slug` which enforces a strict character allowlist (alphanumeric + hyphens only). An invalid slug returns an error before the milestone load is attempted.

**Risk:** None. Path traversal and injection via the slug parameter are prevented.

**Action:** Accept.

### Information Disclosure

**Finding:** If the milestone does not exist, the handler propagates the `Milestone::load` error through `AppError`, which returns an HTTP 500 with the error message. This leaks the fact that the slug does not correspond to a known milestone.

**Risk:** Low. The server is a local developer tool. The information (slug existence) is not sensitive. This is consistent with how all other milestone endpoints behave on unknown slugs.

**Action:** Accept. Tracking a future improvement to return 404 instead of 500 for not-found resources is out of scope for this feature.

### Denial of Service

**Finding:** The endpoint performs a blocking `Milestone::load` (file I/O) in `spawn_blocking`. It does not write any files, does not enqueue agent runs, and does not allocate unbounded memory. The only side effect is a single SSE broadcast.

**Risk:** None. A caller could flood the endpoint with requests, but the only consequence is SSE noise. There is no state modification that could be corrupted by repeated calls, and the server has no rate limiting on any endpoint (consistent with its local-only model).

**Action:** Accept.

### SSE Channel Integrity

**Finding:** The `MilestoneUatFailed` event is emitted on the `milestone_uat` channel — the same channel as `MilestoneUatCompleted`. Frontend consumers that handle this channel must now be prepared for both `type` values.

**Risk:** Existing consumers that destructure the event and only handle `milestone_uat_completed` will receive events with `type: "milestone_uat_failed"` that they do not match. TypeScript's type system now includes both types in the union, so consumers that switch on `type` will get exhaustiveness warnings if they don't handle the new variant. This is the intended behavior — the type system nudges consumers to handle both cases.

**Action:** Accept. No existing consumer is broken — unhandled event types are silently ignored in JavaScript SSE handlers.

### Skill Template Content

**Finding:** The skill templates are static instruction strings embedded in a Rust binary. They contain the URL `http://localhost:7777/api/milestone/<slug>/uat/fail` as a template for agents to fill in. The `<slug>` placeholder must be replaced by the agent before calling the endpoint — a well-behaved agent will not call it with the literal placeholder text.

**Risk:** None. Template text cannot cause security issues; it is executed only when an agent follows the instructions and constructs a real curl command with a real slug.

**Action:** Accept.

## Summary

No security findings require remediation. The endpoint is consistent with the existing security posture of `sdlc-server` (local-only, validated input, no file writes, no state corruption). All findings are accepted with rationale documented above.

**Verdict: Approved.**
