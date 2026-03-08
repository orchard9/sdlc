# Spec: Iterate button on ReleasedPanel

## Problem

When a milestone is released, the ReleasedPanel shows UAT results and a "Re-run UAT" button but offers no forward path for iterating on the shipped work. Users who want to explore improvements or follow-up ideas must manually navigate to the Ponder page, create a new entry, and re-type context from the milestone they just shipped.

## Solution

Add an "Iterate" button to the ReleasedPanel component. Clicking it opens the existing `NewIdeaModal` pre-populated with:

- **Title**: the milestone's original title (no version suffix)
- **Slug**: auto-incremented versioned slug (e.g. `git-status-indicator-v2`, `git-status-indicator-v3`) computed by the `nextIterationSlug` utility from the `iterate-slug-utility` feature
- **Brief**: a template containing the milestone title, slug, original vision text, and a prompt for reflection

After creation, the user is navigated to `/ponder/{newSlug}`.

## Scope

### In scope

- Add "Iterate" button to `ReleasedPanel.tsx` actions row, next to "Re-run UAT"
- Button uses `RefreshCw` icon from lucide-react with label "Iterate"
- On click: fetch existing ponder slugs via `api.getRoadmap(true)`, compute next version slug, open `NewIdeaModal` with pre-populated fields
- On modal create: navigate to `/ponder/{newSlug}`
- Brief template:
  ```
  Iteration of milestone: {title} ({slug})

  Original vision:
  {vision || 'No vision recorded.'}

  What worked well, what to improve, and what to explore next:
  ```

### Out of scope

- Iterate button on FeatureDetail (separate feature: `iterate-feature-button`)
- The `nextIterationSlug` utility itself (separate feature: `iterate-slug-utility`)
- Any backend/API changes (all needed APIs already exist)

## Dependencies

- `iterate-slug-utility` must be implemented first (provides `nextIterationSlug` from `frontend/src/lib/iterate.ts`)
- Existing `NewIdeaModal` component with `initialTitle`, `initialSlug`, `initialBrief` props
- Existing `api.getRoadmap(true)` to fetch all ponder slugs for collision avoidance
- Existing `api.createPonderEntry()` for ponder creation

## Acceptance criteria

1. ReleasedPanel displays an "Iterate" button in the actions row when the milestone is released
2. Clicking "Iterate" opens NewIdeaModal pre-populated with the milestone title, a versioned slug, and the brief template
3. The versioned slug avoids collisions with existing ponder entries
4. After successful creation, the user is navigated to the new ponder entry page
5. The button is styled consistently with the existing action buttons in ReleasedPanel
