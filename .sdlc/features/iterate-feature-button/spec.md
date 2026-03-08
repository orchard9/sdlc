# Spec: Iterate Button on FeatureDetail

## Summary

Add an "Iterate" button to the released feature panel in `FeatureDetail.tsx`. When a feature is in the `released` phase (i.e., `classification.action === 'done'`), the released banner should include an "Iterate" button that creates a new versioned ponder entry pre-seeded with context from the shipped feature.

## Motivation

When a feature ships, teams often want to improve or extend it. Currently there is no way to launch a follow-up ideation session directly from a released feature. This feature closes the build-ship-reflect loop by providing a one-click path from a released feature to a new ponder workspace.

## Behavior

### Button Placement

The "Iterate" button appears inside the existing green "Released" banner (`classification.action === 'done'` block) in `FeatureDetail.tsx`, aligned to the right of the banner header row.

### On Click

1. **Generate a versioned slug** using the `nextIterationSlug` utility from the `iterate-slug-utility` feature. The base slug is the feature's slug (e.g., `git-status-chip` becomes `git-status-chip-v2`).
2. **Create a new ponder entry** via `api.createPonderEntry({ slug, title, brief })` where:
   - `slug` = the generated versioned slug
   - `title` = `"Iterate: {feature.title}"`
   - `brief` = A pre-seeded brief summarizing the feature's spec and design artifacts, e.g., "Follow-up iteration on {feature.title}. Original spec and design are available in the feature artifacts."
3. **Navigate** to the new ponder page at `/ponder/{slug}`.

### Edge Cases

- If ponder creation fails (e.g., slug collision), show a toast/error notification.
- The button is only visible when `classification.action === 'done'` (released features).
- The button should be disabled while the creation request is in flight (loading state).

## Dependencies

- `iterate-slug-utility` feature must provide the `nextIterationSlug` function.
- Existing `api.createPonderEntry` endpoint.

## Out of Scope

- Pre-populating the ponder with the full text of artifacts (just a brief reference).
- Modifying the ponder page itself.
- Backend changes — this is a pure frontend feature using existing APIs.
