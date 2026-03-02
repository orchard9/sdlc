# QA Plan: SSE State Consistency — UatHistoryPanel and SettingsPage gaps

## Scope

Verify that:
1. `UatHistoryPanel` auto-refreshes when a UAT run completes via SSE.
2. `SettingsPage` clears stale errors on successful SSE-triggered refreshes.
3. No existing SSE subscriptions are broken.
4. TypeScript compiles cleanly.

---

## Static checks

### QA-S1 — TypeScript compilation

```bash
cd frontend && npx tsc --noEmit
```

Expected: zero errors.

### QA-S2 — ESLint

```bash
cd frontend && npx eslint src/lib/types.ts src/contexts/SseContext.tsx \
  src/hooks/useSSE.ts \
  src/components/milestones/UatHistoryPanel.tsx \
  src/pages/SettingsPage.tsx
```

Expected: zero new warnings or errors (existing baseline unchanged).

---

## Unit / integration checks

### QA-U1 — `MilestoneUatSseEvent` type shape

Verify `types.ts` exports `MilestoneUatSseEvent` with `type` and `slug` fields.
Import the type in a scratch `.ts` file and assert both fields are present (or
inspect the compiled output).

### QA-U2 — `useSSE` signature backward compatibility

Confirm all existing `useSSE(...)` call sites compile without change. Grep for
`useSSE(` in `frontend/src` and verify none require updating.

```bash
grep -rn "useSSE(" frontend/src --include="*.tsx" --include="*.ts"
```

Expected: all callers pass ≤ 6 arguments (new 7th arg is optional).

### QA-U3 — `SseContext` dispatch completeness

Read `SseContext.tsx` and confirm the `milestone_uat` branch is present in the
dispatch function alongside the other named channels.

---

## Functional checks (manual, requires running server)

### QA-F1 — `UatHistoryPanel` auto-refresh on UAT completion

1. Navigate to a milestone detail page that has at least one feature.
2. Open DevTools → Network → filter by `events` to confirm the SSE stream is
   active.
3. From a terminal, start a UAT run: `sdlc milestone uat start <slug>` (or
   trigger via the UI start button).
4. Wait for the UAT run to complete (agent finishes).
5. Observe `UatHistoryPanel` — the new run should appear **without** a manual
   page reload.

Expected: new `UatRun` row appears automatically within ~1 second of the
`milestone_uat_completed` event arriving.

### QA-F2 — `UatHistoryPanel` does not refresh for other milestones

1. Open milestone detail page for milestone A.
2. Trigger a UAT run for milestone B from a second terminal.
3. Verify `UatHistoryPanel` on milestone A's page does NOT re-fetch (no extra
   network request to `/api/milestones/A/uat-runs`).

This can be verified via DevTools Network tab — confirm the request URL matches
only the active milestone slug.

### QA-F3 — `SettingsPage` error cleared on successful refresh

1. Temporarily cause `api.getConfig()` to fail (e.g., stop the server briefly).
2. Navigate to the Settings page — verify an error message is shown.
3. Restart the server.
4. Wait for the SSE `update` event to fire (triggered by any file change, or
   modify `.sdlc/config.yaml` and save).
5. Verify the error message disappears and the config is displayed.

Expected: error clears as soon as the refresh call succeeds.

### QA-F4 — Existing SSE subscriptions unaffected

1. Navigate to the PonderPage — verify ponder run events still work.
2. Navigate to the InvestigationPage — verify investigation events work.
3. Navigate to the AgentsPage — verify the run panel updates correctly.

These smoke tests confirm the refactored `SseContext` dispatch and `useSSE`
hook did not break existing wiring.

---

## Regression baseline

Run the existing Playwright E2E suite (if any) against a local `sdlc ui`:

```bash
export SDLC_BASE_URL=$(sdlc ui url)
cd frontend && npx playwright test
```

Expected: all previously passing tests continue to pass.

---

## Pass criteria

| Check | Required |
|---|---|
| QA-S1 TypeScript compiles | Yes |
| QA-S2 ESLint clean | Yes |
| QA-U1 Type shape correct | Yes |
| QA-U2 Backward compat | Yes |
| QA-U3 Dispatch branch present | Yes |
| QA-F1 Auto-refresh works | Yes |
| QA-F2 No cross-milestone refresh | Yes |
| QA-F3 Error cleared on success | Yes |
| QA-F4 Existing SSE unaffected | Yes |
| Playwright regression | Yes (or waived if suite is empty) |
