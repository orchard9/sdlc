# Updated Implementation Plan

## Approach: Frontend-Only Fix (No Backend Changes)

The data is already streaming. Three changes: rich renderer for active runs, navigation links, dead code cleanup.

### Changes

**1. RunCard.tsx — Swap renderer for active runs**
- Change `liveEvents` type from `AgentEvent[]` to `RawRunEvent[]`
- Cast SSE parsed data as `RawRunEvent` (line 87)
- Replace `<AgentLog>` with `<RunActivityFeed>` for active runs (line 164-165)
- Pass liveEvents directly: `<RunActivityFeed runId={run.id} isRunning={true} events={liveEvents} />`
- Include `<ActivityTimeSeries events={liveEvents} isRunning={true} />` above the feed

**2. RunCard.tsx — Add navigation link to target entity**
- Add a helper `getTargetLink(run: RunRecord): string | null` mapping run_type+target to frontend route
- Render the target slug as a react-router `<Link>` in the header row (between label and chevron)
- Use `ExternalLink` or `ArrowUpRight` icon from lucide-react, small and subtle
- For vision_align/architecture_align, return null (no link — project-level)

**3. RunActivityFeed.tsx — Handle spawning state**
- When `isRunning && events.length === 0`, show "Spawning agent..." spinner instead of "No activity recorded yet"

**4. RunActivityFeed.tsx — Add auto-scroll**
- Add ref + useEffect to scroll to bottom when new events arrive (same pattern as AgentLog)

**5. Delete dead code**
- Remove `AgentLog.tsx` and `AgentEventLine.tsx` (only used by RunCard old active-run path)

### Decided
- Frontend-only change, no backend work
- Use RunActivityFeed for both active and completed runs
- Navigation link from each agent activity tile to the entity detail page
- Delete AgentLog/AgentEventLine as dead code
- Add auto-scroll to RunActivityFeed for isRunning mode
- Show ActivityTimeSeries during active runs (partial data is fine)

### Open
- Should we unify AgentEvent and RawRunEvent types? (probably yes, AgentEvent becomes dead type)