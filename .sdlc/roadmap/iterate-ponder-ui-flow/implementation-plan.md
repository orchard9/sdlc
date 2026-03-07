# Implementation Plan

## Phase 1: Committed Ponder — Run Button + Milestone Links

### PonderPage.tsx `EntryDetailPane` changes (line ~468-525)

1. **Add milestone links section** after the header when `entry.status === 'committed' && entry.committed_to.length > 0`:
```tsx
{entry.status === 'committed' && entry.committed_to.length > 0 && (
  <div className="shrink-0 px-4 py-2.5 border-b border-border/50 bg-emerald-500/5">
    <div className="flex items-center gap-2 text-xs text-muted-foreground mb-1.5">
      <GitMerge className="w-3 h-3" />
      <span>Committed to {entry.committed_to.length} milestone{entry.committed_to.length > 1 ? 's' : ''}</span>
    </div>
    <div className="flex flex-wrap gap-1.5">
      {entry.committed_to.map(ms => (
        <Link key={ms} to={`/milestones/${ms}`}
          className="inline-flex items-center gap-1 px-2 py-1 text-xs font-medium bg-card border border-border rounded-md hover:bg-accent/60 transition-colors">
          {ms}
          <ArrowUpRight className="w-3 h-3" />
        </Link>
      ))}
    </div>
  </div>
)}
```

2. **Add Prepare button** next to status badge for committed ponders:
```tsx
{entry.status === 'committed' && entry.committed_to.length > 0 && (
  <button onClick={handlePrepare} disabled={prepareRunning}
    className="shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg bg-primary text-primary-foreground hover:bg-primary/90 disabled:opacity-60">
    {prepareRunning ? <Loader2 className="w-3 h-3 animate-spin" /> : <Play className="w-3 h-3" />}
    <span className="hidden sm:inline">{prepareRunning ? 'Preparing...' : 'Prepare'}</span>
  </button>
)}
```

3. **handlePrepare function** using AgentRunContext:
```tsx
const handlePrepare = useCallback(async () => {
  const ms = entry?.committed_to[0]
  if (!ms) return
  await startRun({
    key: `milestone-prepare:${ms}`,
    runType: 'milestone_prepare',
    target: ms,
    label: `prepare: ${ms}`,
    startUrl: `/api/milestone/${ms}/prepare`,
    stopUrl: `/api/milestone/${ms}/prepare/stop`,
  })
}, [entry, startRun])
```

### Phase 2: Parked State — Resume Button
- Add "Resume exploring" button when `status === 'parked'`
- Calls `api.updatePonderEntry(slug, { status: 'exploring' })`

### Phase 3: Status Progress Indicator
- Replace single StatusBadge with a horizontal step indicator:
  exploring → converging → committed
- Grey for future steps, colored for current/past

## Files to Modify
1. `frontend/src/pages/PonderPage.tsx` — EntryDetailPane header area
2. `frontend/src/components/ponder/DialoguePanel.tsx` — emptyState for committed entries (show milestone links instead of hiding all actions)

## Dependencies
- Need `Link` from react-router-dom (already imported as `useNavigate`, need to add `Link`)
- Need `Play` from lucide-react (new import)
- `startRun` from `useAgentRuns()` (already available)
