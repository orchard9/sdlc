# Spec: playwright-first-spec

## Overview

Write and verify the first milestone e2e spec at `frontend/e2e/milestones/v01-directive-core.spec.ts`. This spec covers the acceptance test checklist for the v01-directive-core milestone ("Directive output is complete and rich") using Playwright's `getByRole`, `getByTestId`, and `getByText` locators.

## Goals

1. Write a production-quality Playwright spec targeting the v01-directive-core milestone acceptance criteria.
2. Use only approved locators: `getByRole`, `getByTestId`, `getByText` — no CSS selectors, no XPath.
3. Verify the spec compiles (`npx tsc --noEmit` passes).
4. Run the spec against a live sdlc-server and confirm all tests pass (or document if server cannot start).
5. Record results in qa-results.md.

## Scope

### Features under v01-directive-core

- `directive-richness` — Enrich sdlc next --json with full feature context
- `gate-hint-format` — Standardize gate hints in directive output
- `vision-docs-update` — Vision docs update

### What to test

The acceptance test for v01-directive-core validates that the sdlc-server UI:

1. **Dashboard loads** — the project dashboard renders with a project name, feature count, and milestone sections.
2. **Milestones page** — the `/milestones` route lists milestones with titles and status badges.
3. **Milestone detail** — navigating to `/milestones/v01-directive-core` shows the milestone title and features.
4. **Features page** — the `/features` route lists features with feature cards.
5. **Feature detail** — navigating to a feature shows the feature title, phase badge, next-action panel, artifact list, and task list.
6. **Phase badges render** — `[data-testid="phase-badge"]` elements show valid phase text.
7. **Next action renders** — `[data-testid="next-action"]` shows the pending action on feature cards and detail pages.

## Locators

All selectors must use only:
- `page.getByRole('heading', { name: '...' })` — headings
- `page.getByRole('link', { name: '...' })` — navigation links
- `page.getByRole('button', { name: '...' })` — buttons
- `page.getByTestId('...')` — elements with `data-testid` attributes (see below)
- `page.getByText('...')` — text content

### Known data-testid attributes

| testId | Component | Description |
|---|---|---|
| `feature-title` | FeatureCard, FeatureDetail | Feature title text |
| `phase-badge` | StatusBadge (via testId prop) | Phase or status badge |
| `next-action` | FeatureCard, FeatureDetail | Next pending action display |
| `artifact-list` | FeatureDetail | Artifacts section |
| `task-list` | FeatureDetail | Tasks section |
| `milestone-title` | MilestonesPage, MilestoneDetail | Milestone title text |
| `milestone-status` | StatusBadge (via testId prop) | Milestone status badge |

## Out of Scope

- Authentication flows (server runs locally without auth)
- Agent run interactions (require live Claude API)
- File upload / secret management
- Responsive layout tests

## Success Criteria

- All tests pass against a running sdlc-server (or are documented as blocked by server build issues)
- TypeScript compilation succeeds
- `playwright-report/results.json` is produced
- qa-results.md documents the outcome
