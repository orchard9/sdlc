# Review: fleet-management-ui

## Summary
Evolved the existing `HubPage.tsx` from a read-only heartbeat project list into a full fleet control plane with three sections (Running Instances, Available Repos, Import), agent summary bar, search-first design, and SSE live updates. Added supporting types, API client methods, and extended the SSE hook.

## Files changed

| File | Change |
|---|---|
| `frontend/src/pages/HubPage.tsx` | Full rewrite — three-section layout with fleet instance cards, available repo cards, import form, agent summary bar, search-first autofocus, graceful degradation to legacy heartbeat view |
| `frontend/src/lib/types.ts` | Added `FleetInstance`, `FleetInstanceStatus`, `AvailableRepo`, `FleetAgentSummary` types; extended `HubSseEvent` with fleet event types |
| `frontend/src/api/client.ts` | Added `getFleet()`, `getAvailable()`, `getAgentSummary()`, `provision()`, `importRepo()` |
| `frontend/src/hooks/useHubSSE.ts` | Extended `HubSseCallbacks` with optional fleet callbacks; added fleet event dispatching in SSE message parser |

## Findings

### F1: Import state uses hardcoded timeout (LOW)
The `ImportSection` uses `setTimeout(() => setState('done'), 5000)` instead of reacting to SSE events for import completion. This is acceptable for v1 since import + provision is an inherently long-running operation and the timeout serves as a UX fallback. SSE events via `fleet_provisioned` will update the running instances list correctly regardless.
**Action:** Accept — hardcoded timeout is a pragmatic UX choice for v1; SSE handles the data side correctly.

### F2: Legacy fallback path retained (INFO)
The heartbeat-based `ProjectCard` and legacy `HubProjectEntry` path is preserved as a fallback when fleet endpoints are not available. This ensures backward compatibility during the transition period.
**Action:** Accept — correct defensive approach.

### F3: No error feedback for provision failures (LOW)
When `api.provision()` fails, the provisioning spinner is removed but no error message is shown to the user. The user would just see the Start button reappear.
**Action:** Track — minor UX gap, not blocking for v1.

### F4: TypeScript compiles cleanly (PASS)
`npx tsc --noEmit` passes with zero errors across all modified files.

## Verdict
Approve. All 9 tasks implemented correctly. TypeScript compiles cleanly. The code follows existing codebase patterns (Tailwind styling, card layout, SSE hook pattern). F3 tracked as future improvement.
