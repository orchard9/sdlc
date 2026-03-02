# Tasks: Quota Visibility Panel

## T1 — Extend `ProjectConfig` type in types.ts

**File:** `frontend/src/lib/types.ts`

Add optional `observability` sub-key to the `ProjectConfig` interface:

```ts
export interface ProjectConfig {
  // ... existing fields ...
  observability?: {
    daily_budget_usd?: number
  }
}
```

This is a purely additive, non-breaking change. The field will be absent from the JSON if not set in config.yaml — the frontend must handle undefined gracefully.

---

## T2 — Create `QuotaPanel` component

**File:** `frontend/src/components/layout/QuotaPanel.tsx` (new file)

Implement the full quota panel component:

- Import `useAgentRuns` from `AgentRunContext`
- Derive `todayRuns`, `totalCostToday`, `pct`, `barPct`, `avgCostPerRun`, `remainingRuns` as defined in design.md
- Render:
  - Section header "Quota"
  - Cost display: `$X.XX today`
  - Progress bar (Tailwind-styled, color changes at 80% and 100%)
  - Percentage label with optional `AlertTriangle` icon at ≥ 80%
  - Remaining runs estimate (only shown when ≥ 2 completed runs today)
  - "Daily budget exceeded" message when pct ≥ 100%
- Accept `dailyBudgetUsd?: number` prop; default to `DEFAULT_DAILY_BUDGET_USD = 10.0`
- The component must re-derive values from context on every render (no internal state needed — `runs` from context is the reactive source)

---

## T3 — Integrate `QuotaPanel` into `AgentPanel`

**File:** `frontend/src/components/layout/AgentPanel.tsx`

1. Add a `useEffect` that calls `api.getConfig()` on mount and stores the result in local state (`config`).
2. Add a horizontal divider below the `RunList` scroll area.
3. Render `<QuotaPanel dailyBudgetUsd={config?.observability?.daily_budget_usd} />` at the bottom of the panel (outside the scroll container so it is always visible).
4. The `QuotaPanel` must also appear inside the `FullscreenModal` variant of the panel.

---

## T4 — Visual QA pass

Manual verification steps (no automated test):

- Open the Agent Panel with no runs today → confirm zero state renders without errors
- Trigger a feature run, let it complete → confirm cost updates in the panel
- With ≥ 2 completed runs, confirm "remaining runs" estimate appears
- Manually set `totalCostToday` above 80% of budget (via DevTools mock or running enough) → confirm amber bar and ⚠ icon
- Confirm panel renders correctly in fullscreen modal
- Confirm no console errors or TypeScript errors in strict mode
