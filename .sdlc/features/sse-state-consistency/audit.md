# Security Audit: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## Scope

Five frontend files, ~45 lines net new. No backend changes. No new API
endpoints, no new SSE channels on the server, no authentication changes.

---

## Attack surface analysis

### New SSE event type: `MilestoneUatSseEvent`

The `milestone_uat` SSE channel already existed on the server before this
feature. This change adds a frontend handler for it — it does not introduce the
channel or change who can receive it.

**Data flow:** server broadcasts `{ type, slug }` → SseContext dispatches →
`UatHistoryPanel.onMilestoneUatEvent` fires → `api.listMilestoneUatRuns(slug)`
is called.

**Findings:**

- The `slug` in the SSE event payload is the server-assigned milestone slug.
  The client uses it only as a comparison guard (`if (event.slug ===
  milestoneSlug)`). It does not reflect the slug back to the server or embed it
  in a DOM-rendered element without sanitisation — `milestoneSlug` comes from
  the React prop, not from the SSE event. No XSS vector.

- The re-fetch triggered by the event calls `api.listMilestoneUatRuns(milestoneSlug)`
  where `milestoneSlug` is the React prop (set at component mount from URL
  params), not the value from the SSE event. An adversary who could craft a
  malformed `slug` in the SSE payload cannot redirect the API call to a
  different milestone. No SSRF or IDOR via SSE payload.

- `JSON.parse` is wrapped in try/catch consistent with all other dispatch
  branches. Malformed event data is silently discarded. No crash surface.

### `SettingsPage` refresh path change

`setError(null)` is called before and on success of `api.getConfig()`. This
only affects the component's local React state. No security implication.

### `useSSE` parameter extension

The new `onMilestoneUatEvent` parameter is optional and positioned last. No
existing call site is affected. No security implication.

---

## Findings

| ID | Severity | Finding | Action |
|---|---|---|---|
| A1 | Info | SSE `slug` payload used only as a guard, not reflected in API calls | Accept — implementation is correct |

No exploitable vulnerabilities identified.

## Verdict

APPROVE — no meaningful security surface introduced. The changes close a
stale-data UX bug through an existing, authenticated SSE channel.
