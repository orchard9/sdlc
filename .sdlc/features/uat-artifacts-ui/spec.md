# Feature Specification: uat-artifacts-ui

## Overview

Extend the UAT run display with visual artifacts: a screenshot filmstrip inside `UatHistoryPanel` and a hero thumbnail on dashboard milestone cards. This feature depends on `uat-artifacts-storage` for the backend storage and serving layer that exposes `screenshots` on each `UatRun` record and a binary serving endpoint.

## Problem Statement

UAT runs produce Playwright screenshots and test reports, but these are invisible to the user. The `UatHistoryPanel` shows only verdicts and counts — no visual evidence of what the agent saw during testing. The dashboard milestone cards show no UAT signal at all. Users must manually dig through the filesystem to inspect what happened during a UAT run.

## Solution

Two surface-area changes to the frontend, both reading from the `UatRun` data model extended by `uat-artifacts-storage`:

1. **Screenshot filmstrip in `UatHistoryPanel`** — each run card gains a scrollable horizontal strip of thumbnail screenshots, rendered as `<img>` tags pointing at `/api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}`. Clicking a thumbnail opens a full-size lightbox overlay.

2. **Hero thumbnail on dashboard milestone cards** (`MilestoneDigestRow`) — when the most recent UAT run for a milestone has screenshots, the first screenshot is displayed as a small hero thumbnail in the collapsed milestone card header. This gives at-a-glance visual confidence directly from the dashboard.

## Dependencies

- **`uat-artifacts-storage`** — must be implemented first. It:
  - Adds a `screenshots: Vec<String>` field to `UatRun` (filenames relative to the run's artifact dir)
  - Adds a `GET /api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}` endpoint that serves binary files with correct `Content-Type`
  - Extends `UatRun` in `frontend/src/lib/types.ts` with `screenshots: string[]`

## Data Contract

### Extended `UatRun` type (after `uat-artifacts-storage`)

```ts
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
  screenshots: string[]       // NEW — filenames; empty array if none
}
```

### Artifact serving URL

```
GET /api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}
```

Returns binary content with appropriate `Content-Type` (e.g. `image/png`). This endpoint is implemented by `uat-artifacts-storage`.

### New API client method

```ts
uatArtifactUrl(milestoneSlug: string, runId: string, filename: string): string
// Returns the URL string (not a Promise) — used directly in <img src={...}>
// e.g. `/api/milestones/${slug}/uat-runs/${id}/artifacts/${filename}`
```

## Component Changes

### 1. `UatHistoryPanel` — Screenshot Filmstrip

**File**: `frontend/src/components/milestones/UatHistoryPanel.tsx`

**Change**: Each run card gains a screenshot filmstrip below the existing metadata row.

**Behavior**:
- Renders only when `run.screenshots.length > 0`
- Horizontal scrollable strip of thumbnail `<img>` tags
- Each thumbnail: `object-fit: cover`, fixed height `h-16` (64 px), auto width, rounded corners (`rounded`)
- Clicking a thumbnail opens a full-size lightbox overlay (see below)
- No loading state for images — browser-native lazy loading (`loading="lazy"`)
- `alt` text: `"UAT screenshot {n} of {total}"`

**Filmstrip DOM structure**:
```tsx
<div className="flex gap-2 overflow-x-auto py-1 mt-2">
  {run.screenshots.map((filename, i) => (
    <img
      key={filename}
      src={api.uatArtifactUrl(run.milestone_slug, run.id, filename)}
      alt={`UAT screenshot ${i + 1} of ${run.screenshots.length}`}
      loading="lazy"
      className="h-16 w-auto rounded cursor-pointer shrink-0 border border-border hover:border-primary transition-colors"
      onClick={() => setLightbox({ runId: run.id, index: i })}
    />
  ))}
</div>
```

**Lightbox**:
- Simple overlay: fixed-position dark backdrop with a centered full-size image
- Previous / Next buttons when the run has multiple screenshots
- Keyboard: `Escape` closes, `ArrowLeft` / `ArrowRight` navigates
- Click outside the image closes
- Component: `ScreenshotLightbox` — local to `UatHistoryPanel.tsx`, not shared

### 2. `MilestoneDigestRow` — Hero Thumbnail

**File**: `frontend/src/components/dashboard/MilestoneDigestRow.tsx`

**Change**: Add a hero thumbnail to the collapsed milestone card header.

**Data requirement**: `MilestoneDigestRow` needs access to the most recent UAT run for the milestone. This is fetched via `api.getLatestMilestoneUatRun(slug)` — one call per milestone card, cached in local `useState`, fired in `useEffect` on mount.

**Behavior**:
- Renders only when the latest run has `screenshots.length > 0`
- Single `<img>` tag — first screenshot of the latest run
- Dimensions: `h-8 w-14` (32 × 56 px), `object-fit: cover`, `rounded`
- Positioned to the right of the status badge, before the progress bar
- No lightbox — clicking navigates to `/milestones/{slug}` (same as the title link)
- `alt`: `"Latest UAT screenshot"`
- No loading spinner — image loads naturally; if absent, the space is simply absent (conditional render)

**DOM placement** (inside the existing header flex row):
```tsx
{latestRun?.screenshots?.[0] && (
  <Link to={`/milestones/${milestone.slug}`} className="shrink-0">
    <img
      src={api.uatArtifactUrl(milestone.slug, latestRun.id, latestRun.screenshots[0])}
      alt="Latest UAT screenshot"
      loading="lazy"
      className="h-8 w-14 rounded object-cover border border-border"
    />
  </Link>
)}
```

## API Client Addition

**File**: `frontend/src/api/client.ts`

Add a helper (not a `request()` call — just a URL builder):

```ts
uatArtifactUrl: (milestoneSlug: string, runId: string, filename: string): string =>
  `/api/milestones/${encodeURIComponent(milestoneSlug)}/uat-runs/${encodeURIComponent(runId)}/artifacts/${encodeURIComponent(filename)}`,
```

## Out of Scope

- Backend implementation (handled by `uat-artifacts-storage`)
- Playwright report viewer (separate feature)
- Pagination of screenshots
- Video artifacts (future)
- Screenshot upload on failed steps — that is a UAT agent prompt concern, not UI
- SSE-driven screenshot refresh mid-run (runs complete before the panel refreshes)

## Acceptance Criteria

1. When a `UatRun` has `screenshots: ["step-01.png", "step-02.png"]`, the `UatHistoryPanel` shows a horizontal filmstrip of two thumbnails inside that run's card.
2. Clicking a thumbnail opens a centered lightbox overlay with the full-size image; `Escape` closes it.
3. When a milestone has a completed UAT run with at least one screenshot, the `MilestoneDigestRow` in the dashboard shows a small hero thumbnail in the card header.
4. When `screenshots` is empty or missing, no filmstrip or thumbnail is rendered — no broken image icons.
5. The `uatArtifactUrl` helper is exported from `api/client.ts` and used in both components.
6. No new network calls are added to the hot path (dashboard SSE update cycle) — the `getLatestMilestoneUatRun` call in `MilestoneDigestRow` is fired once on mount only.
