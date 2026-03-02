# Security Audit: Orchestrator Actions Page

## Scope

Frontend-only feature: `ActionsPage.tsx`, `recurrence.ts`, type additions, SseContext extension, sidebar nav, App router. No new Rust server code is introduced by this feature.

## Attack Surface Analysis

### 1. API Calls from the Frontend

**Surface:** The page calls `DELETE /api/orchestrator/actions/:id` and `DELETE /api/orchestrator/webhooks/routes/:id` without confirmation dialogs.

**Risk:** Low. The server's auth middleware (token/cookie gate, local bypass) already gates all `/api/*` endpoints. Deletes require authentication. The lack of a confirmation dialog is a UX choice, not a security gap — any authenticated user can delete. This is consistent with other delete patterns in the UI (e.g., secrets, tool interactions).

**Decision:** Accept. Delete without confirmation is the stated design intent and matches the rest of the UI.

---

### 2. JSON textarea for Tool Input

**Surface:** `ScheduleActionModal` has a `<textarea>` where users type JSON for `tool_input`. The value is serialized via `JSON.parse` + `JSON.stringify` before being sent to the server.

**Risk:** Low. Invalid JSON is silently treated as `{}` (fallback). The server receives a JSON value field — it does not execute it on the server side; it is stored and passed to the tool at dispatch time. No XSS vector: the textarea value is serialized to a JS object, then re-stringified — it is never inserted into the DOM as HTML.

**Decision:** Accept. The current behavior (fallback to `{}` on parse failure) is acceptable for v1. A future improvement could show a parse error to the user, but this is UX, not security.

---

### 3. Input Template in Add Route Modal

**Surface:** `AddRouteModal` accepts a freeform `input_template` string. This template contains `{{payload}}` placeholders that the orchestrator substitutes at dispatch time.

**Risk:** Low. The template is stored verbatim and processed server-side. The frontend does not interpret or render the template — it is treated as an opaque string. The server-side substitution is bounded by the webhook payload, which comes from authenticated external callers (webhook senders know the token or path). No frontend XSS: the template is displayed in a `<td>` via React's text nodes, not `dangerouslySetInnerHTML`.

**Decision:** Accept.

---

### 4. `formatRecurrence` display in table cells

**Surface:** `formatRecurrence(secs)` generates strings like "1h", "30m". These are inserted into React `<td>` children as plain text.

**Risk:** None. The function returns only digit + unit character strings. No HTML injection possible.

**Decision:** Accept.

---

### 5. SSE event dispatch — `action` event type

**Surface:** The SseContext now dispatches `action`-typed SSE events. Malformed payloads are caught and silently discarded (`catch { /* malformed */ }`). The `onActionEvent` callback triggers `refetchActions()` — a GET to the server.

**Risk:** None. A malformed SSE payload causes a no-op. Even if an attacker could inject events into the SSE stream, the worst case is triggering an extra `GET /api/orchestrator/actions` fetch. No writes are triggered by SSE events.

**Decision:** Accept.

---

### 6. Relative time display and tooltip

**Surface:** `relativeTime(iso)` and `futureRelativeTime(iso)` take ISO strings from API responses and generate relative time strings. The absolute timestamp is used as a `title` attribute.

**Risk:** None. The API-sourced ISO string is processed through `new Date(iso).getTime()` — any non-date value returns NaN, resulting in "NaN ago" in the relative display (ugly but not dangerous). The `title` attribute is text, not HTML.

**Decision:** Accept.

---

### 7. Action ID in API paths

**Surface:** Action and route IDs from API responses are embedded in DELETE/PATCH paths via `encodeURIComponent(id)`.

**Risk:** None. All IDs are UUIDs from the server. `encodeURIComponent` is applied. Even if a malicious ID were somehow injected, the server validates UUIDs at the handler level.

**Decision:** Accept.

---

## Summary

No security findings. This feature is frontend-only, adds no new server endpoints, and all inputs are either validated before transmission, treated as opaque data, or displayed via React's safe text rendering. The existing server auth middleware gates all API calls.

**Verdict: PASS**
