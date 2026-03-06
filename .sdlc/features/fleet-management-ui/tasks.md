# Tasks: fleet-management-ui

## T1: Fleet dashboard page — three sections: running instances, available repos, import
Refactor `HubPage.tsx` to have three distinct sections with section headers and counts. Add the `AgentSummaryBar` component at the top. Wire up three parallel `useEffect` fetches for fleet, available, and agent summary data.

## T2: Running instances section — cards with project name, URL, pod status, active milestone, agent badge
Create `FleetInstanceCard` component. Add `FleetInstance` type to `types.ts`. Add `getFleet()` to API client. Card shows status dot (healthy/degraded/failing/unknown color mapping), project name, clickable URL, active milestone badge, feature count, and agent-running pulse badge.

## T3: Available repos section — cards for repos without instances, Start button triggers POST /api/hub/provision
Create `AvailableRepoCard` component. Add `AvailableRepo` type to `types.ts`. Add `getAvailable()` and `provision()` to API client. Start button shows provisioning spinner state. Include first-time user context one-liner above the section.

## T4: Import section — URL input, optional PAT input, Import button calls POST /api/hub/import with progress feedback
Create `ImportSection` component with URL and PAT inputs. Add `importRepo()` to API client. State machine: idle → importing → provisioning → done/error. Inline error display. Clear form on success.

## T5: SSE integration — FleetUpdated event, live status updates without polling
Extend `useHubSSE` hook to handle `fleet_updated`, `fleet_provisioned`, `fleet_agent_status` event types. Update `HubSseEvent` type. Wire SSE callbacks to upsert instances, remove from available on provision, and update agent summary.

## T6: Search and filter — client-side text filter across running and available sections
Move search input to top of page with autofocus. Filter applies to both Running and Available sections simultaneously. Match on name, URL, and description. Show filtered count vs total.

## T7: [user-gap] Search-first design — autofocus search input, instant filter
Ensure search input gets `autoFocus` prop. Search is visually prominent at the top. No submit button — filtering is instant on every keystroke.

## T8: [user-gap] First-time user context — one-liner explaining what an sdlc instance is and what Start does
Add explanatory text above Available section: "Start deploys an sdlc workspace for this repo in the fleet." Subtle muted-foreground styling so it does not distract experienced users.

## T9: [user-gap] Fleet-wide agent activity summary — show count of active agent runs across all instances
Add `FleetAgentSummary` type. Add `getAgentSummary()` to API client. Render `AgentSummaryBar` below search showing "N agents running across M projects" or "No active agents".
