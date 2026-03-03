# Tasks: uat-artifacts-ui

## Prerequisite

**`uat-artifacts-storage` must be merged before implementation begins.** It provides:
- `UatRun.screenshots: string[]` on the Rust struct and JSON API
- `UatRun.screenshots: string[]` in `frontend/src/lib/types.ts`
- `GET /api/milestones/{slug}/uat-runs/{id}/artifacts/{filename}` serving endpoint

All tasks below assume these contracts are in place.

---

## T1 — Add `uatArtifactUrl` helper to API client

**File**: `frontend/src/api/client.ts`

Add a URL-builder function (not a `request()` call):

```ts
uatArtifactUrl: (milestoneSlug: string, runId: string, filename: string): string =>
  `/api/milestones/${encodeURIComponent(milestoneSlug)}/uat-runs/${encodeURIComponent(runId)}/artifacts/${encodeURIComponent(filename)}`,
```

Export it as part of the `api` object.

**Verify**: TypeScript compiles with no errors; the function is accessible as `api.uatArtifactUrl(...)`.

---

## T2 — Add `ScreenshotLightbox` component to `UatHistoryPanel.tsx`

**File**: `frontend/src/components/milestones/UatHistoryPanel.tsx`

Add `ScreenshotLightbox` as a module-local component (not exported). It:

1. Accepts props: `screenshots: string[]`, `milestoneSlug: string`, `runId: string`, `initialIndex: number`, `onClose: () => void`
2. Uses `useState(initialIndex)` for current `index`
3. Registers a `keydown` listener via `useEffect` for `Escape` / `ArrowLeft` / `ArrowRight`
4. Renders via `createPortal(…, document.body)` to avoid stacking-context clipping
5. Layout: fixed-position dark backdrop (`bg-black/80`), centered image, prev/next buttons, close button, page counter
6. Clicking the backdrop calls `onClose`; clicking the image calls `e.stopPropagation()`
7. Prev button is disabled when `index === 0`; next button is disabled when `index === screenshots.length - 1`

**Verify**: Opening and closing the lightbox works; keyboard navigation cycles through screenshots; click-outside closes.

---

## T3 — Add screenshot filmstrip to each run card in `UatHistoryPanel`

**File**: `frontend/src/components/milestones/UatHistoryPanel.tsx`

1. Add `const [lightbox, setLightbox] = useState<{ runId: string; index: number } | null>(null)` at the top of `UatHistoryPanel`
2. Inside the run card render loop, after the existing metadata row, add:

```tsx
{run.screenshots?.length > 0 && (
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
)}
```

3. After the run list, render the lightbox conditionally:

```tsx
{lightbox && (() => {
  const run = runs.find(r => r.id === lightbox.runId)
  if (!run || !run.screenshots?.length) return null
  return (
    <ScreenshotLightbox
      screenshots={run.screenshots}
      milestoneSlug={run.milestone_slug}
      runId={run.id}
      initialIndex={lightbox.index}
      onClose={() => setLightbox(null)}
    />
  )
})()}
```

**Verify**: With a mock `UatRun` that has `screenshots: ["a.png", "b.png"]`, two thumbnails appear in the run card. The `data-testid="uat-history-panel"` attribute remains intact.

---

## T4 — Add hero thumbnail to `MilestoneDigestRow`

**File**: `frontend/src/components/dashboard/MilestoneDigestRow.tsx`

1. Add imports: `useState`, `useEffect` from React; `UatRun` from `@/lib/types`; `api` from `@/api/client`
2. Add local state: `const [latestRun, setLatestRun] = useState<UatRun | null>(null)`
3. Add effect on mount:

```ts
useEffect(() => {
  api.getLatestMilestoneUatRun(milestone.slug)
    .then(run => setLatestRun(run))
    .catch(() => {})
}, [milestone.slug])
```

4. In the collapsed header flex row, insert after the `<StatusBadge>` and before `<ProgressBar>`:

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

**Verify**: When `getLatestMilestoneUatRun` returns a run with `screenshots: ["screen.png"]`, a 32×56 thumbnail appears in the card header. When `screenshots` is empty or the call returns `null`, no `<img>` is rendered and no broken image icon appears.

---

## T5 — TypeScript compilation and lint check

Run from the `frontend/` directory:

```bash
npx tsc --noEmit
npm run lint
```

Fix any type errors or lint violations introduced by the changes above.

**Verify**: Zero TypeScript errors, zero ESLint errors/warnings introduced by this feature.

---

## Task Order

T1 → T2 → T3 → T4 → T5

T1 must precede T2, T3, T4 (all use `api.uatArtifactUrl`). T2 must precede T3 (T3 renders `ScreenshotLightbox`). T4 and T3 can be done in parallel after T1 and T2 are complete.
