# Design: Subagent Input/Output Cards in Activity Feed

## Data Flow

```
Backend (runs.rs)                    pairEvents.ts                      SubagentCard.tsx
─────────────────                    ─────────────                      ────────────────
subagent_started   ──┐
subagent_progress  ──┼──→  Group by task_id  ──→  PairedSubagentExchange  ──→  <SubagentCard />
subagent_completed ──┘     into paired event
```

## Type Additions (types.ts)

```typescript
export interface PairedSubagentExchange {
  kind: 'subagent_exchange'
  taskId: string
  description?: string
  status?: string        // from subagent_completed
  summary?: string       // from subagent_completed
  lastToolName?: string  // from subagent_progress
  totalTokens?: number   // from subagent_progress or subagent_completed
  durationMs?: number    // from subagent_progress or subagent_completed
  isComplete: boolean    // true once subagent_completed received
}
```

Add to `PairedEvent` union:
```typescript
export type PairedEvent =
  | PairedInitEvent
  | PairedToolExchange
  | PairedAssistantText
  | PairedRunResult
  | PairedSubagentExchange   // NEW
```

## pairEvents.ts Changes

Add a `Map<string, PairedSubagentExchange>` to track open subagent exchanges by `task_id`.

- `subagent_started`: Flush pending assistant text. Create a new `PairedSubagentExchange` with `isComplete: false`, store in the map by `task_id`.
- `subagent_progress`: Update the map entry with `lastToolName`, `totalTokens`, `durationMs`.
- `subagent_completed`: Update the map entry with `status`, `summary`, token/duration data, set `isComplete: true`. Push to result array and remove from map.
- At end of function: flush any still-open subagents (in-progress) to the result array.

## SubagentCard Component

Location: `frontend/src/components/runs/SubagentCard.tsx`

### Visual Layout

```
┌─ steel-blue border ──────────────────────────────────────────┐
│ 🤖 "Researching file structure"    ✓ completed   2.1s       │
│ ┌──────────────────────────────────────────────────────────┐ │
│ │ Last tool: Read  ·  1,247 tokens                        │ │  ← collapsible
│ └──────────────────────────────────────────────────────────┘ │
│ Found 3 relevant files matching the pattern...               │  ← summary
└──────────────────────────────────────────────────────────────┘
```

- Left border: `border-indigo-500` (close to the steel blue used in time series)
- Icon: `Bot` from lucide-react (distinguishes from tool's `Wrench`)
- Status badge: green "completed" / amber spinner "running" / red "failed"
- Collapsible details: last tool name, total tokens, duration
- Summary text: line-clamped to 3 lines, matching ToolCallCard style

### States

| State | Rendering |
|-------|-----------|
| Started (no progress/complete) | Description + amber "running" spinner |
| In progress | Description + last tool + token count + amber spinner |
| Completed | Description + status badge + summary + details toggle |
| Failed | Description + red status badge + summary |

## RunActivityFeed Changes

Add import for `SubagentCard`. Add case in `PairedEventRow`:

```typescript
case 'subagent_exchange':
  return <SubagentCard event={event} />
```

## File Changes Summary

| File | Change |
|------|--------|
| `frontend/src/lib/types.ts` | Add `PairedSubagentExchange`, extend `PairedEvent` union |
| `frontend/src/components/runs/pairEvents.ts` | Handle 3 subagent event types, group by task_id |
| `frontend/src/components/runs/SubagentCard.tsx` | New component |
| `frontend/src/components/runs/RunActivityFeed.tsx` | Import + case for subagent_exchange |
