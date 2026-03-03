# Security Audit: Concurrency Heatmap

## Scope

Frontend-only feature. No backend changes, no new API endpoints, no new data transmission. All data is derived from the existing `GET /api/runs` endpoint already used by `AgentRunContext`.

---

## Attack Surface Analysis

### New Components

| Component | Input | Output | Risk Surface |
|-----------|-------|--------|--------------|
| `useHeatmap.ts` | `RunRecord[]` from existing context | Computed `HeatmapData` | Pure computation — no I/O |
| `ConcurrencyStrip.tsx` | `HeatmapData` | SVG-like flex divs | Rendering only |
| `RunsHeatmap.tsx` | `RunRecord[]`, callbacks | React DOM | XSS via run labels |
| `RunsPage.tsx` | `runs` from context | React DOM | Same as heatmap |

### Modified Components

| Component | Change | Risk |
|-----------|--------|------|
| `AgentPanel.tsx` | Added `RunsHeatmap compact` + `Link` | No new inputs; link is hardcoded to `/runs` |
| `App.tsx` | Added `/runs` route | Route to an authenticated page; no open redirect |
| `Sidebar.tsx` | Added nav link to `/runs` | Hardcoded internal route |

---

## Findings

### XSS Potential in Run Labels

**Severity:** Low (mitigated)

`RunsHeatmap.tsx` renders `run.label` in two places:
1. The label column: `{run.label}` rendered as React text node — **not vulnerable** (React escapes text content).
2. The bar tooltip: `title={tooltip}` where `tooltip` is a template literal including `run.label`, `run.run_type`, and `formatDuration(...)`. The native `title` attribute is sanitized by the browser — **not vulnerable**.

The `run.label` field is populated by the backend from `RunRecord` which is written by the sdlc-server. Labels are under server control, not end-user input. No injection risk.

**Action:** None required. React text escaping and native browser title attribute are sufficient.

---

### Open Redirect

**Severity:** None

The "full view →" `Link` in `AgentPanel.tsx` is a hardcoded string `"/runs"` using `react-router-dom`'s `Link` component. It cannot be made to redirect to an external URL. No open redirect risk.

---

### Information Disclosure

**Severity:** None

The `/runs` page displays run history already shown in the Agent Activity panel. No new data is surfaced — it is a visual re-arrangement of existing `RunRecord` data. The same data is accessible in the existing panel with no authentication change.

---

### Client-Side Computation Safety

**Severity:** None

`useHeatmap` uses `Math.max(...array)` spread operator on the bucket array. With the existing 50-record retention limit on runs, and bucket counts bounded to ≤200 per 30-minute bucket size, this is safe from stack overflow. Max spread size is well under V8's ~100k element safe limit.

---

### Denial of Service (Browser)

**Severity:** None

The heatmap computation is O(N * B) where N ≤ 50 runs (existing retention) and B ≤ 200 buckets. Total operations ≤ 10,000 per render. No performance concern.

---

## Compliance Notes

- No PII is introduced or surfaced. `run.label` and `run.run_type` are operational metadata, not user data.
- No new cookies, localStorage keys, or session data are written.
- No new network requests are made.

---

## Verdict

**No security issues found.** This is a read-only visualization of existing data, computed entirely in the browser from already-fetched context state. React's default XSS protections cover all rendering paths. No audit findings require action.
