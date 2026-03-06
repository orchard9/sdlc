# Security Audit: Webhook Payload Inspector UI

## Scope

This audit covers:
- New `POST /api/webhooks/{route}/replay/{id}` endpoint
- Frontend payload query and display logic
- Type extensions to existing webhook route interfaces

## Findings

### A1: Replay endpoint allows arbitrary tool execution via stored payloads -- LOW RISK

**Description:** The replay endpoint dispatches stored payloads through the tool associated with the registered route. An authenticated user could replay any stored payload.

**Assessment:** This is by design -- the endpoint sits behind the existing auth middleware (tunnel auth/cookie gate). Only authenticated users with access to the Ponder dashboard can trigger replays. The tool execution is identical to what the orchestrator tick loop does during normal webhook dispatch.

**Action:** Accepted. No change needed.

### A2: Payload data exposed to frontend without sanitization -- LOW RISK

**Description:** The `query_webhook_data` endpoint returns raw payload bodies (parsed JSON or lossy UTF-8 strings) to the frontend, which renders them in a `<pre>` block.

**Assessment:** Since payloads are displayed in `<pre>` elements (not rendered as HTML), XSS risk is minimal. React automatically escapes content in JSX expressions. The data is from webhook senders that the user themselves configured.

**Action:** Accepted. React's default escaping is sufficient.

### A3: Wide time-range scan in replay could be slow -- LOW RISK

**Description:** The replay endpoint scans 30 days of payloads (up to 10,000) to find a single payload by UUID.

**Assessment:** For store-only routes (the typical use case), payload volumes are low. The scan happens in a `spawn_blocking` task, so it cannot block the async runtime. If this becomes a performance issue, a `get_by_id` method can be added to the backend trait.

**Action:** Accepted. Adequate for current scale.

### A4: Secret tokens not exposed in frontend -- COMPLIANT

**Description:** The backend masks `secret_token` as `"***"` in the `list_routes` response. The frontend only uses this to show a lock icon.

**Assessment:** Correct. No risk of token leakage through the UI.

**Action:** No change needed.

### A5: No CSRF concern on replay endpoint -- COMPLIANT

**Description:** The replay endpoint uses POST method and is behind auth middleware.

**Assessment:** The existing auth mechanism (token/cookie) protects against CSRF. The endpoint requires explicit authentication.

**Action:** No change needed.

## Verdict

**Approved.** No high-risk findings. All findings are documented and accepted. The feature follows existing security patterns and does not introduce new attack surface beyond what authenticated users already have access to.
