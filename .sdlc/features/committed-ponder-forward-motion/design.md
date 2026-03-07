# Design: Committed Ponder Forward Motion

## Overview

Two additions to `EntryDetailPane` in `PonderPage.tsx` and the `DialoguePanel` empty state:

1. **Milestone links section** — rendered below the header when committed
2. **Prepare button** — rendered in the header next to the status badge

## Component Changes

### PonderPage.tsx — EntryDetailPane

#### Milestone Links Banner

Insert a banner between the header `</div>` (line 525) and the desktop layout div. Conditionally rendered when `entry.status === 'committed' && entry.committed_to.length > 0`.

```tsx
{entry.status === 'committed' && entry.committed_to.length > 0 && (
  <div className="shrink-0 px-4 py-2.5 border-b border-border/50 bg-emerald-500/5">
    <p className="text-xs font-medium text-emerald-400/80 mb-1.5">Committed milestones</p>
    <div className="flex flex-wrap gap-1.5">
      {entry.committed_to.map(ms => (
        <Link
          key={ms}
          to={`/milestones/${ms}`}
          className="text-xs font-mono px-2 py-0.5 rounded bg-emerald-500/10 text-emerald-300 hover:bg-emerald-500/20 transition-colors"
        >
          {ms}
        </Link>
      ))}
    </div>
  </div>
)}
```

#### Prepare Button

Add after the `StatusBadge` in the header, replacing the spot where the commit button currently hides for committed status. When committed, show a "Prepare" button instead.

Uses `useAgentRuns()` — already available via context. Key pattern: `milestone-prepare:{slug}` matching the existing WavePlan pattern.

```tsx
{entry.status === 'committed' && entry.committed_to.length > 0 && (
  <button
    onClick={handlePrepare}
    disabled={prepareRunning}
    className="shrink-0 flex items-center gap-1.5 px-2.5 py-1 text-xs font-medium rounded-lg border border-emerald-500/30 text-emerald-400 hover:bg-emerald-500/10 transition-colors disabled:opacity-60 disabled:cursor-not-allowed"
    title={`Prepare milestone ${entry.committed_to[0]}`}
  >
    {prepareRunning
      ? <Loader2 className="w-3 h-3 animate-spin" />
      : <Play className="w-3 h-3" />}
    <span className="hidden sm:inline">{prepareRunning ? 'Preparing…' : 'Prepare'}</span>
  </button>
)}
```

`handlePrepare` calls `startRun()`:
```tsx
const prepareSlug = entry.committed_to[0]
const prepareKey = `milestone-prepare:${prepareSlug}`
const prepareRunning = isRunning(prepareKey)

const handlePrepare = () => {
  startRun({
    key: prepareKey,
    runType: 'milestone_prepare',
    target: prepareSlug,
    label: `prepare: ${prepareSlug}`,
    startUrl: `/api/milestone/${prepareSlug}/prepare`,
    stopUrl: `/api/milestone/${prepareSlug}/prepare/stop`,
  })
}
```

### DialoguePanel.tsx

Update the empty state for committed ponders to show milestone links instead of hiding all actions:

```tsx
{entry.status === 'committed' && entry.committed_to.length > 0 && (
  <div className="flex flex-col items-center gap-1.5">
    <p className="text-xs text-emerald-400/60">Committed to:</p>
    {entry.committed_to.map(ms => (
      <Link key={ms} to={`/milestones/${ms}`}
        className="text-xs font-mono text-emerald-300 hover:underline">
        {ms}
      </Link>
    ))}
  </div>
)}
```

## Imports Added

- `PonderPage.tsx`: `Play` from lucide-react, `Link` from react-router-dom (if not already imported)
- `DialoguePanel.tsx`: `Link` from react-router-dom

## No Backend Changes

All backend endpoints already exist. The `committed_to` field is already returned by `/api/roadmap/{slug}`.
