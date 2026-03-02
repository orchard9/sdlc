# Spec: Quota Visibility Panel

## Problem

Users running agents in SDLC can see the raw dollar cost of each run (e.g. `$0.42`) but have no intuition about what that means in terms of API quota consumption. The Anthropic API enforces a daily token limit — users approaching that limit will hit rate errors without warning. There is no in-product signal for:

- How close the user is to their daily API quota
- How many more runs they can reasonably afford at current burn rate
- Whether they should slow down or parallelize differently

This gap was surfaced directly by an early user:
> "what is happening to my quota? I see $ but I don't know what that means as far as quota usage"

## Solution

Add a Quota Visibility Panel to the Agent Activity panel (right-side panel in the UI). The panel translates the cumulative dollar cost of all runs in the current session into:

1. **% of estimated daily API limit consumed** — based on a configurable daily budget ceiling (default: a reasonable Anthropic API Pro tier daily limit, e.g. $10/day)
2. **Estimated remaining runs** — at the current average cost per run, how many more runs could be done before hitting the ceiling
3. **Cumulative cost today** — aggregated from all RunRecords in the current day

The panel is lightweight: it reads existing `RunRecord.cost_usd` values already present in the system — no new backend calls or data structures needed for the MVP. The daily budget ceiling is either read from `.sdlc/config.yaml` (if present) or defaults to a hardcoded constant.

## User-Facing Behavior

### Quota panel location
The panel appears at the **bottom of the Agent Activity panel** (the right sidebar). It is always visible when the agent panel is open, regardless of whether any runs are active.

### Display states

**No runs today (cost = $0.00):**
```
Quota
──────────────────────
$0.00 today   [░░░░░░░░░░] 0%
```

**Some quota consumed:**
```
Quota
──────────────────────
$2.14 today   [██░░░░░░░░] 21%
≈ 8 runs remaining (at $0.27/run avg)
```

**Approaching limit (> 80%):**
```
Quota
──────────────────────
$8.30 today   [████████░░] 83%  ⚠
≈ 1 run remaining (at $0.27/run avg)
```

**Limit reached (≥ 100%):**
```
Quota
──────────────────────
$10.40 today  [██████████] 104%
Daily budget exceeded
```

### Behavior details

- **Scope:** Only runs started within the current calendar day (local browser time) are counted.
- **Cost input:** Reads `RunRecord.cost_usd` — only populated on completed/failed/stopped runs. Running runs contribute `$0.00` until they complete (cost settles at result time).
- **Refresh:** Recalculates every time the runs list updates (which happens via SSE `run_finished` events). No polling — purely reactive.
- **Average cost:** Computed as `total_cost / number_of_completed_runs_today`. Excluded from display if fewer than 2 completed runs exist today.
- **Daily ceiling:** `$10.00` default. Overridable via `observability.daily_budget_usd` key in `.sdlc/config.yaml`.

### Configuration

No new configuration is strictly required. Optional override in `.sdlc/config.yaml`:

```yaml
observability:
  daily_budget_usd: 25.0  # Default: 10.0
```

## Out of Scope

- Real-time quota data from the Anthropic API (not available without a separate API call; deferred)
- Per-model quota differentiation (all models treated equally for now)
- Notification or alert on approaching limit (pure display only)
- Historical data across days (today only)
- Concurrency recommendations based on quota (that's the `concurrency-heatmap` feature)

## Acceptance Criteria

1. The quota panel is visible in the Agent Activity panel whenever the panel is open.
2. After any run completes, the panel recalculates and updates without a page refresh.
3. The progress bar reflects `(total_cost_today / daily_budget) * 100`, capped at 100% for the bar (but the % label shows the true value even above 100%).
4. "Estimated remaining runs" appears only when ≥ 2 completed runs exist today.
5. The warning indicator (⚠) appears at ≥ 80% of the daily budget.
6. The panel renders correctly when no runs have completed today (zero state).
7. The daily budget ceiling defaults to $10.00 if not configured.
8. The component does not make any new API calls — it derives all values from the existing `runs` list available in `AgentRunContext`.
