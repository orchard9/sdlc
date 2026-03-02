# Design: Quota Visibility Panel

## Overview

The Quota Visibility Panel is a pure frontend component added to the bottom of the existing Agent Activity panel. It reads from the `runs` list already available in `AgentRunContext` — no new server routes, no new API calls, no new backend state. The only addition is an optional `observability.daily_budget_usd` key in `.sdlc/config.yaml` which the frontend can read via the existing `GET /api/config` endpoint.

## Architecture

```
AgentPanel (frontend/src/components/layout/AgentPanel.tsx)
└── QuotaPanel (frontend/src/components/layout/QuotaPanel.tsx)  ← NEW
    └── useAgentRuns() → reads runs[]
        → filters to today's completed runs
        → computes: totalCostToday, pct, avgCostPerRun, remainingRuns
        → reads dailyBudget from config or uses 10.0 default
```

No server-side changes needed. The daily budget default is hardcoded as `DEFAULT_DAILY_BUDGET_USD = 10.0` in the component; projects that want a custom ceiling set it in config and it is returned by the existing `GET /api/config` response (the `ProjectConfig` type already includes an extensible structure).

If `observability.daily_budget_usd` is not present in config, the component falls back to the default silently.

## Component: QuotaPanel

**File:** `frontend/src/components/layout/QuotaPanel.tsx`

### Props

```ts
interface QuotaPanelProps {
  dailyBudgetUsd?: number  // From config, falls back to DEFAULT (10.0)
}
```

### Internal derivations

```ts
const DEFAULT_DAILY_BUDGET_USD = 10.0

// Filter to today (local browser time)
const todayRuns = runs.filter(r => {
  if (!r.cost_usd || r.status === 'running') return false
  const d = new Date(r.started_at)
  const now = new Date()
  return d.getFullYear() === now.getFullYear()
      && d.getMonth() === now.getMonth()
      && d.getDate() === now.getDate()
})

const totalCostToday = todayRuns.reduce((sum, r) => sum + (r.cost_usd ?? 0), 0)
const budget = dailyBudgetUsd ?? DEFAULT_DAILY_BUDGET_USD
const pct = (totalCostToday / budget) * 100          // can exceed 100
const barPct = Math.min(pct, 100)                     // capped for bar width

const completedToday = todayRuns.filter(r => r.status === 'completed')
const avgCostPerRun = completedToday.length >= 2
  ? totalCostToday / completedToday.length
  : null

const remainingRuns = avgCostPerRun !== null && avgCostPerRun > 0
  ? Math.floor((budget - totalCostToday) / avgCostPerRun)
  : null
```

### Visual design (ASCII wireframe)

```
┌─────────────────────────────┐
│ Quota                       │
│ $2.14 today                 │
│ ████████░░░░░░░░░░░░  21%   │
│ ≈ 8 runs remaining          │
└─────────────────────────────┘
```

Progress bar uses Tailwind `bg-primary` for filled portion, `bg-muted` for unfilled. At ≥ 80% the filled bar turns `bg-amber-500`; at ≥ 100% it turns `bg-red-500`.

Warning icon (`AlertTriangle` from lucide-react) appears inline next to the % when `pct >= 80`.

### Rendering states

| Condition | What renders |
|---|---|
| No runs today | `$0.00 today` + empty bar + no remaining estimate |
| 1 completed run | Cost + bar + % but no remaining estimate |
| ≥ 2 completed runs | Cost + bar + % + remaining estimate |
| ≥ 80% consumed | Amber bar + ⚠ icon |
| ≥ 100% consumed | Red bar + red % text + "Daily budget exceeded" |

## Config integration

`ProjectConfig` (types.ts) already has an extensible top-level structure. The `observability` sub-key will be added as an optional field:

```ts
// In types.ts — extend ProjectConfig
export interface ProjectConfig {
  // ... existing fields ...
  observability?: {
    daily_budget_usd?: number
  }
}
```

The Rust server already serializes the entire `config.yaml` into the config response, so no server-side change is needed as long as the YAML key is added to the `ProjectConfig` Rust struct in `sdlc-core`. Since this feature avoids server changes, the frontend will simply treat the field as optional and use the default if absent. If the Rust struct doesn't include the field, it will be absent from the JSON and the frontend gracefully defaults.

**Decision:** For this feature, the frontend handles the default entirely in the component. Adding `observability.daily_budget_usd` to the Rust `ProjectConfig` struct is logged as a future enhancement task (not blocking MVP).

## Integration point: AgentPanel

The `QuotaPanel` is added at the **bottom** of `AgentPanel`, below the `RunList`, separated by a divider. It always renders when the panel is open.

```tsx
// AgentPanel.tsx — bottom section
<div className="border-t border-border/50 px-2 py-2">
  <QuotaPanel dailyBudgetUsd={config?.observability?.daily_budget_usd} />
</div>
```

The `config` object comes from a `useEffect` fetch of `GET /api/config`, same pattern as `Dashboard.tsx`. The fetch result is stored in local state within `AgentPanel`. Since config rarely changes, no SSE subscription is needed — a single fetch on mount is sufficient.

## Files Changed

| File | Change |
|---|---|
| `frontend/src/components/layout/QuotaPanel.tsx` | NEW — quota panel component |
| `frontend/src/components/layout/AgentPanel.tsx` | ADD QuotaPanel at bottom, add config fetch |
| `frontend/src/lib/types.ts` | ADD `observability?: { daily_budget_usd?: number }` to `ProjectConfig` |

No Rust, no new routes, no new tests beyond the component itself.

## Edge Cases

- **Running run with no cost yet:** Excluded from `totalCostToday` (cost_usd only present after completion)
- **Negative remaining runs:** Clamped to 0, shows "Daily budget exceeded" message
- **Zero avg cost per run:** Remaining estimate hidden (division guard)
- **Config fetch fails:** `dailyBudgetUsd` prop is `undefined`; component uses default silently
- **Timezone boundary:** Uses local browser date; a run started at 11:59 PM and completing at 12:01 AM goes into the previous day's bucket — acceptable for MVP
