# Design: Iterate button on ReleasedPanel

## Component changes

### `frontend/src/components/milestones/ReleasedPanel.tsx`

**New imports:**
- `RefreshCw` from `lucide-react`
- `NewIdeaModal` from `@/components/ponder/NewIdeaModal`
- `nextIterationSlug` from `@/lib/iterate`
- `useNavigate` from `react-router-dom`
- `api` (already imported)

**New state:**
- `iterateModalOpen: boolean` — controls NewIdeaModal visibility
- `iterateSlug: string` — computed next version slug
- `iterateBrief: string` — pre-populated brief text

**Flow:**
1. User clicks "Iterate" button
2. `handleIterate` fetches `api.getRoadmap(true)` to get all existing ponder slugs
3. Calls `nextIterationSlug(milestoneSlug, existingSlugs)` to compute next version
4. Builds brief from milestone title + vision
5. Opens `NewIdeaModal` with `initialTitle`, `initialSlug`, `initialBrief`
6. On `onCreated`, navigates to `/ponder/{slug}`

**Button placement:** In the actions `<div>` (line 104), after the "Re-run UAT" / "Running" button and before the "Submit manually" link. Styled as a secondary action button matching the existing button pattern.

## Visual layout

```
+--------------------------------------------------+
| [check] Milestone Released                       |
|          {milestone title}                       |
|                                                  |
| 5 features  2 UAT runs  Latest: PASS  Mar 7     |
|                                                  |
| [Re-run UAT]  [Iterate]  Submit manually         |
|                                                  |
| --- separator ---                                |
| Next milestone: {title} ->                       |
+--------------------------------------------------+
```

The "Iterate" button uses `RefreshCw` icon, border-border bg-muted styling (neutral, secondary to the green Re-run UAT button).

## Data flow

```
ReleasedPanel
  -> user clicks Iterate
  -> api.getRoadmap(true) -> PonderSummary[]
  -> nextIterationSlug(milestoneSlug, slugs) -> "milestone-slug-v2"
  -> build brief template with milestone.title + milestone.vision
  -> open NewIdeaModal(initialTitle, initialSlug, initialBrief)
  -> user confirms in modal -> api.createPonderEntry()
  -> onCreated(slug) -> navigate("/ponder/{slug}")
```

## No new files

This feature only modifies `ReleasedPanel.tsx`. It depends on `nextIterationSlug` from `frontend/src/lib/iterate.ts` (created by the `iterate-slug-utility` feature).

[Mockup](mockup.html)
