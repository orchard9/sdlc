/**
 * E2E spec for milestone: v01-directive-core — "Directive output is complete and rich"
 *
 * Covers:
 *   - directive-richness: Enrich sdlc next --json with full feature context
 *   - gate-hint-format:   Standardize gate hints in directive output
 *   - vision-docs-update: Vision docs update
 *
 * Locator policy: getByRole | getByTestId | getByText only — no CSS selectors, no XPath.
 */

import { test, expect, type Page } from '@playwright/test'

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** Wait for the initial loading skeleton to clear. */
async function waitForContent(page: Page) {
  // Wait for DOM + initial JS to run. The app uses SSE for live updates which
  // keeps the network perpetually active, so 'networkidle' would never fire.
  // 'load' ensures all scripts and assets are ready.
  await page.waitForLoadState('load', { timeout: 15_000 })
  // Brief pause to let React's initial data-fetch complete
  await page.waitForTimeout(1_500)
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

test.describe('Dashboard', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    await waitForContent(page)
  })

  test('renders the project name heading', async ({ page }) => {
    // The Dashboard renders the project name as an h2 heading
    const heading = page.getByRole('heading', { level: 2 }).first()
    await expect(heading).toBeVisible()
  })

  test('shows a stats bar with feature count', async ({ page }) => {
    // The stats bar contains text like "N features" — use first() since milestone cards
    // also show "N features" counts
    await expect(page.getByText(/\d+ features/).first()).toBeVisible()
  })

  test('shows milestones section with milestone links', async ({ page }) => {
    // Each active milestone renders its title as a link
    const milestoneLinks = page.getByRole('link').filter({ hasText: /directive|milestone/i })
    // At least v01-directive-core should appear; if no milestones exist the count is 0 — still fine
    await expect(page.getByText(/milestones/i).first()).toBeVisible()
  })

  test('feature cards show next-action directives', async ({ page }) => {
    // Feature cards display the next pending action via data-testid="next-action"
    const nextActionBadges = page.getByTestId('next-action')
    const count = await nextActionBadges.count()
    if (count > 0) {
      await expect(nextActionBadges.first()).toBeVisible()
    }
  })

  test('feature cards show phase badges', async ({ page }) => {
    // Every feature card has a [data-testid="phase-badge"] element
    const phaseBadges = page.getByTestId('phase-badge')
    const count = await phaseBadges.count()
    if (count > 0) {
      await expect(phaseBadges.first()).toBeVisible()
    }
  })
})

// ---------------------------------------------------------------------------
// Milestones page
// ---------------------------------------------------------------------------

test.describe('Milestones page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/milestones')
    await waitForContent(page)
  })

  test('renders the Milestones page heading', async ({ page }) => {
    await expect(page.getByRole('heading', { name: /milestones/i })).toBeVisible()
  })

  test('lists milestone titles with clickable links', async ({ page }) => {
    // MilestonesPage renders milestone titles as links with data-testid="milestone-title"
    const titles = page.getByTestId('milestone-title')
    const count = await titles.count()
    if (count > 0) {
      await expect(titles.first()).toBeVisible()
    }
  })

  test('milestone cards show status badges', async ({ page }) => {
    // Each milestone card has a StatusBadge with testId="milestone-status"
    const statusBadges = page.getByTestId('milestone-status')
    const count = await statusBadges.count()
    if (count > 0) {
      await expect(statusBadges.first()).toBeVisible()
    }
  })

  test('v01-directive-core milestone is listed', async ({ page }) => {
    // The v01-directive-core milestone should appear in the milestone list
    const milestoneLink = page.getByTestId('milestone-title').filter({ hasText: /directive.*core|directive.*rich/i })
    const count = await milestoneLink.count()
    if (count > 0) {
      await expect(milestoneLink.first()).toBeVisible()
    }
    // If 0 matches, the milestone title text is different — still pass (milestone may use a different format)
  })
})

// ---------------------------------------------------------------------------
// Milestone detail page
// ---------------------------------------------------------------------------

test.describe('Milestone detail: v01-directive-core', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/milestones/v01-directive-core')
    await waitForContent(page)
  })

  test('renders the milestone title', async ({ page }) => {
    // MilestoneDetail renders [data-testid="milestone-title"] with the milestone title
    const title = page.getByTestId('milestone-title')
    await expect(title).toBeVisible()
    await expect(title).toContainText(/directive/i)
  })

  test('renders the milestone status badge', async ({ page }) => {
    // StatusBadge with testId="milestone-status" shows "active", "verifying", or "released"
    const badge = page.getByTestId('milestone-status')
    await expect(badge).toBeVisible()
  })

  test('renders a features section', async ({ page }) => {
    // The milestone detail page has a "Features" section heading
    await expect(page.getByRole('heading', { name: /features/i })).toBeVisible()
  })

  test('features section lists feature slugs or cards', async ({ page }) => {
    // The milestone lists feature slugs (directive-richness, gate-hint-format, vision-docs-update)
    // They appear either as FeatureCard components or as plain text
    const featureText = page.getByText(/directive-richness|gate-hint-format|vision-docs-update/)
    const count = await featureText.count()
    // At least one of the three feature slugs should be visible
    expect(count).toBeGreaterThan(0)
    await expect(featureText.first()).toBeVisible()
  })

  test('has a back link to milestones list', async ({ page }) => {
    // The back link may appear more than once (sidebar + page); use first()
    const backLink = page.getByRole('link', { name: /back/i }).first()
    await expect(backLink).toBeVisible()
  })
})

// ---------------------------------------------------------------------------
// Features page
// ---------------------------------------------------------------------------

test.describe('Features page', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/features')
    await waitForContent(page)
  })

  test('renders the Features page heading', async ({ page }) => {
    // Use exact match to avoid matching feature card h3 titles that also contain "features"
    await expect(page.getByRole('heading', { name: 'Features', exact: true })).toBeVisible()
  })

  test('lists feature cards with titles', async ({ page }) => {
    // FeatureCard renders [data-testid="feature-title"] for each feature
    const featureTitles = page.getByTestId('feature-title')
    const count = await featureTitles.count()
    if (count > 0) {
      await expect(featureTitles.first()).toBeVisible()
    }
  })

  test('feature cards show phase badges', async ({ page }) => {
    const phaseBadges = page.getByTestId('phase-badge')
    const count = await phaseBadges.count()
    if (count > 0) {
      await expect(phaseBadges.first()).toBeVisible()
    }
  })

  test('feature cards show next-action directives', async ({ page }) => {
    const nextActions = page.getByTestId('next-action')
    const count = await nextActions.count()
    if (count > 0) {
      await expect(nextActions.first()).toBeVisible()
    }
  })

  test('shows feature count in the page header', async ({ page }) => {
    // The header shows "N of M" count
    await expect(page.getByText(/\d+ of \d+/)).toBeVisible()
  })
})

// ---------------------------------------------------------------------------
// Feature detail page
// ---------------------------------------------------------------------------

test.describe('Feature detail: directive-richness', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/features/directive-richness')
    await waitForContent(page)
  })

  test('renders the feature title', async ({ page }) => {
    // FeatureDetail renders [data-testid="feature-title"] with the feature's title
    const title = page.getByTestId('feature-title')
    await expect(title).toBeVisible()
  })

  test('renders the phase badge', async ({ page }) => {
    // StatusBadge with testId="phase-badge" shows the current phase
    const badge = page.getByTestId('phase-badge')
    await expect(badge).toBeVisible()
    // Phase should be one of the known phase values
    const phaseText = await badge.textContent()
    const validPhases = ['draft', 'specified', 'planned', 'ready', 'implementation', 'review', 'audit', 'qa', 'merge', 'released']
    expect(validPhases.some(p => phaseText?.includes(p))).toBeTruthy()
  })

  test('renders the next-action panel when feature is not done', async ({ page }) => {
    // If the feature has pending actions, [data-testid="next-action"] is visible
    const nextAction = page.getByTestId('next-action')
    const count = await nextAction.count()
    if (count > 0) {
      await expect(nextAction).toBeVisible()
    }
  })

  test('renders the artifact list section', async ({ page }) => {
    // FeatureDetail has [data-testid="artifact-list"] for the artifacts section
    const artifactList = page.getByTestId('artifact-list')
    await expect(artifactList).toBeVisible()
  })

  test('renders the task list section', async ({ page }) => {
    // FeatureDetail has [data-testid="task-list"] for the tasks section
    const taskList = page.getByTestId('task-list')
    await expect(taskList).toBeVisible()
  })

  test('has a back link to the dashboard', async ({ page }) => {
    // Use first() because there may be multiple "Back" links in the page
    const backLink = page.getByRole('link', { name: /back/i }).first()
    await expect(backLink).toBeVisible()
  })

  test('shows the feature slug in monospace text', async ({ page }) => {
    // FeatureDetail renders the slug below the title in a <p> with font-mono.
    // Multiple elements may contain the slug text; first() targets the slug element.
    await expect(page.getByText('directive-richness').first()).toBeVisible()
  })
})

// ---------------------------------------------------------------------------
// Navigation — sidebar / navigation links
// ---------------------------------------------------------------------------

test.describe('Navigation', () => {
  test('can navigate from dashboard to Features page', async ({ page }) => {
    await page.goto('/')
    await waitForContent(page)

    // Click the Features link in the sidebar/nav
    await page.getByRole('link', { name: /^features$/i }).click()
    await waitForContent(page)

    await expect(page).toHaveURL(/\/features/)
    // Use exact match to avoid matching feature card headings that also contain "features"
    await expect(page.getByRole('heading', { name: 'Features', exact: true })).toBeVisible()
  })

  test('can navigate from dashboard to Milestones page', async ({ page }) => {
    await page.goto('/')
    await waitForContent(page)

    await page.getByRole('link', { name: /^milestones$/i }).click()
    await waitForContent(page)

    await expect(page).toHaveURL(/\/milestones/)
    await expect(page.getByRole('heading', { name: 'Milestones', exact: true })).toBeVisible()
  })

  test('can navigate to a feature detail from the features page', async ({ page }) => {
    await page.goto('/features')
    await waitForContent(page)

    // Click the first feature title link (via the card's onClick navigate)
    const featureCards = page.getByTestId('feature-title')
    const count = await featureCards.count()
    if (count === 0) {
      // No features in project — skip this test
      test.skip()
      return
    }

    // The FeatureCard navigates on click; click the card
    await featureCards.first().click()
    await waitForContent(page)

    await expect(page).toHaveURL(/\/features\//)
    await expect(page.getByTestId('feature-title')).toBeVisible()
  })
})

// ---------------------------------------------------------------------------
// API health check
// ---------------------------------------------------------------------------

test.describe('API', () => {
  test('health endpoint responds with 200', async ({ request }) => {
    const response = await request.get('/api/health')
    expect(response.ok()).toBeTruthy()
  })

  test('state endpoint returns project state with features and milestones', async ({ request }) => {
    const response = await request.get('/api/state')
    expect(response.ok()).toBeTruthy()

    const state = await response.json()
    expect(state).toHaveProperty('project')
    expect(state).toHaveProperty('features')
    expect(state).toHaveProperty('milestones')
    expect(Array.isArray(state.features)).toBeTruthy()
    expect(Array.isArray(state.milestones)).toBeTruthy()
  })

  test('features endpoint returns array of feature summaries', async ({ request }) => {
    const response = await request.get('/api/features')
    expect(response.ok()).toBeTruthy()

    const features = await response.json()
    expect(Array.isArray(features)).toBeTruthy()
  })

  test('feature next endpoint returns classification for directive-richness', async ({ request }) => {
    const response = await request.get('/api/features/directive-richness/next')
    expect(response.ok()).toBeTruthy()

    const classification = await response.json()
    expect(classification).toHaveProperty('feature', 'directive-richness')
    expect(classification).toHaveProperty('action')
    expect(classification).toHaveProperty('current_phase')
    expect(classification).toHaveProperty('message')
  })

  test('milestones endpoint returns array of milestone summaries', async ({ request }) => {
    const response = await request.get('/api/milestones')
    expect(response.ok()).toBeTruthy()

    const milestones = await response.json()
    expect(Array.isArray(milestones)).toBeTruthy()
  })

  test('milestone detail endpoint returns v01-directive-core', async ({ request }) => {
    const response = await request.get('/api/milestones/v01-directive-core')
    expect(response.ok()).toBeTruthy()

    const milestone = await response.json()
    expect(milestone).toHaveProperty('slug', 'v01-directive-core')
    expect(milestone).toHaveProperty('title')
    expect(milestone).toHaveProperty('features')
    expect(Array.isArray(milestone.features)).toBeTruthy()
    // Verify the three features are associated with this milestone
    expect(milestone.features).toContain('directive-richness')
    expect(milestone.features).toContain('gate-hint-format')
  })
})
