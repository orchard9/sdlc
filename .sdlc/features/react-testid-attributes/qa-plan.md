# QA Plan: data-testid Attributes

## Objective

Confirm that every `data-testid` attribute specified in the spec is present in the rendered DOM and that no regressions were introduced.

## Test Strategy

Because this project has no Playwright test suite yet (the feature exists to enable one), QA focuses on:

1. **TypeScript compilation** — proves no type errors from the new `testId` prop or attribute additions.
2. **DOM attribute inspection** — manual spot-check using browser DevTools or a headless render, confirming attributes appear on the correct elements.
3. **Regression check** — visual/functional review that UI renders identically to before.

## Test Cases

### TC-1: TypeScript compiles without errors

**Command:**
```bash
cd /Users/jordanwashburn/Workspace/orchard9/sdlc/frontend && npx tsc --noEmit
```
**Pass:** Exit 0, zero diagnostic messages.

### TC-2: StatusBadge renders data-testid when testId prop is supplied

**Verify (DevTools / Playwright snippet):**
```js
document.querySelector('[data-testid="phase-badge"]') !== null
```
**Expected:** Element found on any FeatureCard or FeatureDetail page.

### TC-3: feature-title present on FeatureCard

**Verify:**
```js
document.querySelector('[data-testid="feature-title"]') !== null
```
**Expected:** Element found on Dashboard or FeaturesPage where FeatureCards are rendered.

### TC-4: artifact-list, artifact-status, next-action, task-list in FeatureDetail

**Navigate to** `/features/<any-slug>` with at least one artifact and task.

**Verify:**
```js
['artifact-list', 'artifact-status', 'next-action', 'task-list'].every(
  id => document.querySelector(`[data-testid="${id}"]`) !== null
)
```
**Expected:** All four attributes found.

### TC-5: milestone-title and milestone-status in MilestoneDetail

**Navigate to** `/milestones/<any-slug>`.

**Verify:**
```js
['milestone-title', 'milestone-status'].every(
  id => document.querySelector(`[data-testid="${id}"]`) !== null
)
```
**Expected:** Both found.

### TC-6: milestone-title and milestone-status in MilestonesPage

**Navigate to** `/milestones`.

**Verify same selectors as TC-5 but on the list page.**

### TC-7: No visual regression

Open the app after the changes and confirm:
- Feature cards render identically to before (same layout, colours, text).
- Feature detail page renders identically.
- Milestone pages render identically.

## Pass Criteria

All TC-1 through TC-7 pass. TypeScript is the automated gate; the remainder are confirmable via brief manual inspection or a Playwright `page.locator` call.
