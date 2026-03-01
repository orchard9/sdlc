# Design: UatHistoryPanel

## Component Structure

```
MilestoneDetail
└── <section> "UAT History"
    └── UatHistoryPanel (milestoneSlug)
        ├── Loading spinner (while fetching)
        ├── Empty state (no runs)
        └── Run list (sorted most-recent-first)
            └── UatRunRow (per run)
                ├── Verdict badge
                ├── Date
                ├── Tests passed/total
                └── Tasks created count (omit if 0)
```

## File Layout

```
frontend/src/components/milestones/
└── UatHistoryPanel.tsx   (new)

frontend/src/pages/
└── MilestoneDetail.tsx   (modified — import and render UatHistoryPanel)
```

## Props Interface

```ts
// UatHistoryPanel.tsx
interface UatHistoryPanelProps {
  milestoneSlug: string
}
```

## Data Flow

1. `useEffect` on mount: call `api.listMilestoneUatRuns(milestoneSlug)`
2. Sort result by `completed_at ?? started_at` descending
3. Store in local `runs` state; clear `loading` flag on settle
4. Render list

No SSE subscription. No polling. One-time fetch.

## Verdict Badge Colors

| Verdict        | Tailwind classes                        | Label         |
|----------------|-----------------------------------------|---------------|
| `pass`         | `bg-emerald-600/80 text-emerald-100`    | PASS          |
| `pass_with_tasks` | `bg-amber-600/80 text-amber-100`    | PASS + TASKS  |
| `failed`       | `bg-red-600/80 text-red-100`           | FAILED        |

## ASCII Wireframe

```
┌─ UAT History ──────────────────────────────────────────────────┐
│                                                                  │
│  [ PASS ]   Jan 15, 2026   12/12 passed   0 tasks created        │
│  [ PASS+TASKS ] Jan 10, 2026  10/12 passed  2 tasks created      │
│  [ FAILED ] Jan 5, 2026    6/12 passed    0 tasks created        │
│                                                                  │
│  (empty state: "No UAT runs yet.")                               │
└──────────────────────────────────────────────────────────────────┘
```

## Integration in MilestoneDetail

After the existing `<section>` for Features, add:

```tsx
<section className="mt-8">
  <h3 className="text-sm font-semibold mb-3">UAT History</h3>
  <UatHistoryPanel milestoneSlug={slug} />
</section>
```

## Styling Conventions

- Follow existing Tailwind patterns in the codebase
- Row background: `bg-card border border-border rounded-xl p-4`
- Space between rows: `space-y-2`
- Text sizes: label `text-xs`, value `text-sm`
- Loading spinner: `Loader2` from `lucide-react` with `animate-spin`
- Muted text: `text-muted-foreground`
