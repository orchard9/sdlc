# Feature Spec: Add data-testid Attributes to Key React Components

## Overview

Add `data-testid` attributes to specific interactive and structural elements across the React frontend so that Playwright end-to-end tests can reliably locate them without depending on fragile CSS class names or text content.

## Problem Statement

The current React components lack stable, semantic selectors. Playwright specs must rely on text matches (e.g. `getByText('approved')`) or class-name chains that break whenever styling changes. `data-testid` attributes are invisible to users, survive refactors, and are the industry-standard mechanism for test-stable selectors.

## Scope

Audit the existing `.tsx` files in `frontend/src/` and add `data-testid` attributes only to elements that currently exist. No new UI elements are introduced.

### Elements to annotate

| testid | Location | Element |
|---|---|---|
| `phase-badge` | `FeatureCard`, `FeatureDetail`, `MilestoneDetail`, `MilestonesPage` | The `<StatusBadge>` that shows a feature's current phase |
| `feature-title` | `FeatureCard`, `FeatureDetail` | The heading / text node showing the feature title |
| `artifact-list` | `FeatureDetail` | The `<section>` wrapping the list of artifact viewers |
| `artifact-status` | `ArtifactViewer` | The `<StatusBadge>` that shows an artifact's status |
| `next-action` | `FeatureDetail` | The panel / div showing the next directive action |
| `task-list` | `FeatureDetail` | The `<section>` wrapping the tasks list |
| `milestone-status` | `MilestoneDetail`, `MilestonesPage` (MilestoneCard) | The `<StatusBadge>` for a milestone's status |
| `milestone-title` | `MilestoneDetail`, `MilestonesPage` (MilestoneCard) | The heading text showing the milestone title |

### Elements that do NOT exist and are therefore excluded

- `approve-button` — no approve/reject buttons exist in the current UI; artifact approval is done via the CLI only.
- `reject-button` — same reason.
- `start-uat-button` — MilestoneDetail has no UAT button in the current source.
- `directive-panel` — the next-action block in FeatureDetail is used; labelled as `next-action` (the concept is identical).

## Out of Scope

- No new UI components or features.
- No styling changes.
- No changes to tests (there are none to update).
- No changes to Rust or backend code.

## Acceptance Criteria

1. `data-testid="phase-badge"` is present on the `StatusBadge` that renders the feature phase in `FeatureCard` and `FeatureDetail`.
2. `data-testid="feature-title"` is present on the title `<h3>` in `FeatureCard` and the `<h2>` in `FeatureDetail`.
3. `data-testid="artifact-list"` is present on the artifacts `<section>` in `FeatureDetail`.
4. `data-testid="artifact-status"` is present on the `StatusBadge` in `ArtifactViewer`.
5. `data-testid="next-action"` is present on the directive panel div in `FeatureDetail`.
6. `data-testid="task-list"` is present on the tasks `<section>` in `FeatureDetail`.
7. `data-testid="milestone-status"` is present on the milestone status `StatusBadge` in both `MilestoneDetail` and `MilestonesPage`.
8. `data-testid="milestone-title"` is present on the milestone title element in both `MilestoneDetail` and `MilestonesPage`.
9. TypeScript compiles without errors (`npx tsc --noEmit` in `frontend/`).
10. No existing behaviour, styling, or functionality is changed.

## Implementation Notes

- `StatusBadge` accepts arbitrary extra props via `className`; a new `data-testid` prop must be threaded through explicitly by adding it to the `StatusBadge` component interface.
- All edits use the `Edit` tool against actual `.tsx` source files — no generated code.
- Verify with `npx tsc --noEmit` after changes.
