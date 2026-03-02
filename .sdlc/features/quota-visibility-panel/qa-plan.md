# QA Plan: Quota Visibility Panel

## Scope

Validate that the `QuotaPanel` component correctly derives and displays quota usage from the existing `RunRecord` list, handles all display states, and integrates correctly into `AgentPanel` without breaking existing behavior.

## Test Scenarios

### QA-1: Zero state (no completed runs today)

**Setup:** No runs have been completed today (or clear all runs from the list).  
**Expected:**
- Panel renders with header "Quota"
- Shows `$0.00 today`
- Progress bar is empty (0 width or all-unfilled)
- Percentage shows `0%`
- No "remaining runs" estimate shown
- No warning icon or red/amber coloring

---

### QA-2: Single completed run

**Setup:** Exactly one run completed today with `cost_usd = 0.15`.  
**Expected:**
- Shows `$0.15 today`
- Progress bar shows 1.5% fill (0.15 / 10.00 * 100)
- No "remaining runs" estimate (need ≥ 2 runs)
- No warning icon

---

### QA-3: Multiple completed runs — normal range

**Setup:** 3 completed runs today with costs $0.20, $0.30, $0.50 (total = $1.00).  
**Expected:**
- Shows `$1.00 today`
- Bar at 10% fill
- Avg: $1.00 / 3 = $0.33
- Remaining: `floor((10.00 - 1.00) / 0.33)` = `floor(27.27)` = 27
- Shows "≈ 27 runs remaining"

---

### QA-4: Approaching limit (≥ 80%)

**Setup:** Total cost today = $8.50 (default $10 budget).  
**Expected:**
- Bar is amber (`bg-amber-500`)
- `AlertTriangle` icon visible next to percentage
- Shows `85%`
- Remaining estimate shown if ≥ 2 runs

---

### QA-5: Budget exceeded (≥ 100%)

**Setup:** Total cost today = $10.40.  
**Expected:**
- Bar is full (100% visual width) and red (`bg-red-500`)
- Percentage label shows `104%` in red text
- Message "Daily budget exceeded" shown instead of remaining estimate
- `AlertTriangle` icon visible

---

### QA-6: Running runs do not count

**Setup:** One run with `status = 'running'` (no `cost_usd` yet), one completed run with `cost_usd = $0.50`.  
**Expected:**
- Total shows `$0.50 today` (running run excluded)
- Bar reflects only the $0.50 completed cost

---

### QA-7: Runs from previous days excluded

**Setup:** One run with `started_at` = yesterday, `cost_usd = $5.00`; one run today with `cost_usd = $0.25`.  
**Expected:**
- Total shows `$0.25 today`
- Yesterday's run is not counted

---

### QA-8: Panel visible in fullscreen modal

**Setup:** Open Agent Activity Panel, then click the fullscreen expand button.  
**Expected:**
- `QuotaPanel` is visible at the bottom of the fullscreen modal
- Values match those in the non-fullscreen panel

---

### QA-9: Panel updates after run completes

**Setup:** Have the Agent Activity panel open while a run is active.  
**Expected:**
- When the run completes (SSE `run_finished` event triggers), the panel recalculates
- Cost and bar update without page refresh
- No stale data shown

---

### QA-10: Custom daily budget from config

**Setup:** Set `observability.daily_budget_usd: 25.0` in `.sdlc/config.yaml`. Total cost today = $5.00.  
**Expected:**
- Bar shows 20% (5.00 / 25.00 * 100)
- No warning icon (below 80% of 25.00 = 20.00)

---

### QA-11: Missing config — default budget applied

**Setup:** Remove `observability` key from config (or have it absent).  
**Expected:**
- Panel uses $10.00 as the default budget
- No errors, no blank state

---

## TypeScript / Build Checks

- `npm run build` (or `npm run type-check`) passes with no TypeScript errors in `QuotaPanel.tsx` or modified files
- No `any` types introduced in new code

## Regression Checks

- Existing `AgentPanel` behavior (RunList, expand/collapse, fullscreen, stop button) is unchanged
- No visual regressions in the panel header or run cards
- `AgentRunContext` is not modified; `QuotaPanel` is a read-only consumer
