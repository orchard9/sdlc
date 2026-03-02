# Audit: Quota Visibility Panel

## Scope

Security, accessibility, and code quality audit of the three changed files:
- `frontend/src/lib/types.ts` — `observability` field addition to `ProjectConfig`
- `frontend/src/components/layout/QuotaPanel.tsx` — new component
- `frontend/src/components/layout/AgentPanel.tsx` — integration changes

## Security

### S1 — No user-controlled data rendered as HTML
`QuotaPanel` renders only computed numbers (`totalCostToday`, `pct`, `remainingRuns`) derived from `RunRecord.cost_usd` values sourced from the server. There is no `dangerouslySetInnerHTML` and no string interpolation of external data into DOM. **Pass.**

### S2 — Config fetch uses existing authenticated client
`api.getConfig()` uses the same `api` client as all other endpoints — authenticated via session cookie/token. No new auth surface introduced. **Pass.**

### S3 — No secrets or tokens exposed
`QuotaPanel` reads only cost totals from run records. No API keys, tokens, or credentials are touched. **Pass.**

## Accessibility

### A1 — Progress bar lacks ARIA role
The progress bar `<div>` does not have `role="progressbar"` or `aria-valuenow`/`aria-valuemax` attributes. Screen readers cannot interpret it as a progress indicator.

**Action:** Add ARIA attributes to the bar container:
```tsx
<div
  role="progressbar"
  aria-valuenow={Math.round(barPct)}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-label="Daily API quota usage"
  className="h-1.5 w-full rounded-full bg-muted overflow-hidden"
>
```

### A2 — Warning icon has no accessible label
`AlertTriangle` renders as a bare SVG with no `aria-label` or `title`. Screen readers will skip it or announce "icon."

**Action:** Add `aria-label` to the icon wrapper or use `aria-describedby` on the percentage span.

### A3 — Color-only status signaling
Warning (amber) and exceeded (red) states are communicated via color alone. Users with color vision deficiencies may not distinguish these states from the normal state.

**Mitigation (already present):** The `AlertTriangle` icon provides a non-color signal at ≥ 80%, and "Daily budget exceeded" text provides a non-color signal at 100%. The color-only gap exists in the 80-99% range where the icon is shown — mitigated by the icon. **Acceptable for MVP.**

## Code Quality

### Q1 — `isToday` helper is inline in the component file
The `isToday` function is a pure utility and could be unit-tested independently. It is correct but not exported or tested.

**Action:** Move to `frontend/src/lib/utils.ts` in a future pass (tracked as F3 in review). Acceptable as-is.

### Q2 — No empty dependency array warning risk
The `useEffect` in `AgentPanel` has an empty dependency array `[]` — correct for a one-time mount fetch of config. `api.getConfig` is a stable reference. **Pass.**

### Q3 — TypeScript strictness
`npx tsc --noEmit` passes with zero errors. All props typed, no implicit `any`. **Pass.**

## Findings and Actions

| ID | Severity | Finding | Action |
|---|---|---|---|
| A1 | Medium | Progress bar missing ARIA role/attributes | Fix now |
| A2 | Low | AlertTriangle icon has no accessible label | Fix now |
| A3 | Low | Color-only state in 80-99% range | Accepted — icon present |
| Q1 | Low | isToday not unit tested | Track as task |
| S1-S3 | — | No security issues | Pass |

## Fixes Applied

### Fix A1 — ARIA on progress bar

In `QuotaPanel.tsx`, update the outer progress bar div:

```tsx
<div
  role="progressbar"
  aria-valuenow={Math.round(barPct)}
  aria-valuemin={0}
  aria-valuemax={100}
  aria-label="Daily API quota usage"
  className="h-1.5 w-full rounded-full bg-muted overflow-hidden"
>
```

### Fix A2 — Accessible label on warning icon

Wrap the `AlertTriangle` with a `<span>` that has `aria-label`, or add a visually hidden label:

```tsx
{(isWarning || isExceeded) && (
  <span aria-label="Warning: approaching daily limit">
    <AlertTriangle className="w-2.5 h-2.5 shrink-0" aria-hidden="true" />
  </span>
)}
```

**Verdict: APPROVE after applying A1 and A2 fixes.**
