# QA Plan: activity-time-series

## Unit tests — `buildTimeSeries`

These are pure function tests; no rendering required. Test file: `frontend/src/components/runs/buildTimeSeries.test.ts` (or `.spec.ts`).

### QA-1: Returns null when no events have `ts`
- Input: events with no `ts` field
- Expected: `buildTimeSeries(events)` returns `null`

### QA-2: Returns null with fewer than 2 timestamped events
- Input: single event with `ts`
- Expected: returns `null`

### QA-3: Correct bucket count
- Input: events spanning 10 seconds with 2 timestamps; `bucketCount = 5`
- Expected: result has exactly 5 buckets

### QA-4: LLM interval classification
- Input: `assistant` event at t=0s, `user` event at t=4s, result at t=4.1s; bucketCount=1
- Expected: single bucket with `llm ≈ 4000ms`, `tool` and `subagent` near 0

### QA-5: Tool interval classification
- Input: `user` event at t=0s, `assistant` event at t=3s; bucketCount=1
- Expected: single bucket with `tool ≈ 3000ms`

### QA-6: Subagent interval classification
- Input: `subagent_started` at t=1s (task_id="x"), `subagent_completed` at t=6s (task_id="x"); bucketCount=1
- Expected: single bucket with `subagent ≈ 5000ms`

### QA-7: Idle fills uncovered time
- Input: only `init` at t=0s, `result` at t=10s; bucketCount=1
- Expected: single bucket with `idle ≈ 10000ms`, others near 0

### QA-8: Bucket overlap computation
- Input: `llm` interval from t=0s–t=6s; `bucketCount=3` (each bucket = 2s)
- Expected: buckets[0].llm ≈ 2000, buckets[1].llm ≈ 2000, buckets[2].llm ≈ 2000

### QA-9: Partial overlap across bucket boundary
- Input: `llm` interval from t=1s–t=3s; `bucketCount=4` (each bucket = 1s for 4s total run)
- Expected: buckets[1].llm ≈ 1000ms, buckets[2].llm ≈ 1000ms, others ≈ 0

### QA-10: Unmatched `subagent_started` (no completed event)
- Input: `subagent_started` at t=2s (task_id="y"), no matching `subagent_completed`; last ts at t=5s
- Expected: subagent interval extends to t=5s (last ts); no crash

### QA-11: `runDurationMs` is correct
- Input: first ts at "2026-01-01T00:00:00Z", last ts at "2026-01-01T00:00:10Z"
- Expected: `result.runDurationMs === 10000`

---

## Component render tests — `ActivityTimeSeries`

Test file: `frontend/src/components/runs/ActivityTimeSeries.test.tsx`.

### QA-12: Renders fallback when `buildTimeSeries` returns null
- Pass empty events array
- Expected: DOM contains "Time breakdown not available" text; no SVG element

### QA-13: Renders SVG when valid events provided
- Pass synthetic events with `ts` fields covering llm + tool intervals
- Expected: DOM contains an `<svg>` element

### QA-14: Legend items present
- Pass valid events
- Expected: "LLM", "Tool", "Subagent", "Idle" labels present in rendered output

### QA-15: Correct number of bars rendered
- Pass valid events; default 20 buckets
- Expected: 20 bar group elements in SVG

### QA-16: Tooltip appears on hover
- Render with valid events; simulate `mouseenter` on first bar group
- Expected: tooltip element becomes visible with time range text

---

## Integration / manual smoke tests

### QA-17: Chart visible in expanded RunCard (completed run)
- Start the dev server (`sdlc ui`)
- Expand any completed run in the sidebar
- Verify stacked bar chart appears above the event feed
- Verify color-coded bars (violet/amber/gray visible for LLM/Tool/Idle runs)
- Verify x-axis labels present

### QA-18: Fallback for legacy runs
- Select a run that was recorded before `telemetry-wallclock-timestamps` shipped (no `ts` fields in events)
- Verify the fallback message "Time breakdown not available" is shown
- Verify the `RunActivityFeed` event list still renders below

### QA-19: Live update during active run
- Start a new agent run (via any feature action)
- Expand the run card while it is still running
- Observe the chart updating as new events arrive (every 2s poll)
- Verify bars grow/change as the run progresses

### QA-20: Tooltip content accuracy
- Hover over a specific bar
- Verify tooltip shows the correct time range (matches bar position)
- Verify per-type ms values sum to approximately the bucket width

### QA-21: Responsive chart width
- Resize the browser window narrower
- Verify chart reflows to fill container without overflow

### QA-22: Single-type run (all LLM)
- Use a run event fixture where all time is LLM (only assistant/user events, no tool calls or subagents)
- Verify bars are entirely violet with no other color segments

---

## Regression checks

### QA-23: `RunActivityFeed` still works unchanged
- Expand any run; verify event feed renders correctly (no regression from the `useRunTelemetry` lift refactor)
- Check both running and completed runs

### QA-24: `RunCard` stop button still functional
- Expand a running run; click the stop button; verify run stops and status updates

### QA-25: TypeScript compilation clean
- Run `cd frontend && npx tsc --noEmit`
- Expected: zero type errors
