# Spec: UatHistoryPanel React Component

## Overview

Add a `UatHistoryPanel` React component to the `MilestoneDetail` page that displays the full history of UAT runs for a milestone. The component performs a one-time fetch on mount from `GET /api/milestones/{slug}/uat-runs` and renders a list of runs sorted most-recent-first.

## Data Contract

### API

- **Endpoint**: `GET /api/milestones/{slug}/uat-runs`
- **Client method**: `api.listMilestoneUatRuns(slug: string): Promise<UatRun[]>`
- **Response**: Array of `UatRun` objects (may be empty)

### Types (already in `frontend/src/lib/types.ts`)

```ts
export type UatVerdict = 'pass' | 'pass_with_tasks' | 'failed'

export interface UatRun {
  id: string
  milestone_slug: string
  started_at: string
  completed_at: string | null
  verdict: UatVerdict
  tests_total: number
  tests_passed: number
  tests_failed: number
  playwright_report_path: string | null
  tasks_created: string[]
  summary_path: string
}
```

## Component Specification

### `UatHistoryPanel`

**File**: `frontend/src/components/milestones/UatHistoryPanel.tsx`

**Props**:
```ts
interface UatHistoryPanelProps {
  milestoneSlug: string
}
```

**Behavior**:
- Fetches UAT run history on mount via `api.listMilestoneUatRuns(milestoneSlug)`
- Displays runs sorted most-recent-first (by `completed_at` or `started_at`)
- Shows an empty state message "No UAT runs yet." when no runs exist
- Shows a loading spinner while fetching
- Handles fetch errors gracefully (shows nothing or a brief error note)

**Root element**: Must have `data-testid="uat-history-panel"`.

### Visual Design

Each run row displays:
1. **Verdict badge** — inline-colored badge using Tailwind:
   - `pass` → green (`bg-emerald-600/80 text-emerald-100`)
   - `pass_with_tasks` → yellow (`bg-amber-600/80 text-amber-100`)
   - `failed` → red (`bg-red-600/80 text-red-100`)
2. **Date** — `completed_at` formatted as a short locale date string; fall back to `started_at` if null
3. **Test count** — `{tests_passed}/{tests_total} passed`
4. **Tasks created** — `{tasks_created.length} task(s) created`; omit if zero

### Placement in MilestoneDetail

The panel is placed below the existing Features section in `frontend/src/pages/MilestoneDetail.tsx`, inside a new `<section>` with a heading "UAT History".

## Out of Scope

- No SSE subscription — a simple one-time fetch on mount is sufficient
- No pagination
- No inline display of UAT run details
