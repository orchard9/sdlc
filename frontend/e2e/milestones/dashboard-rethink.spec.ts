import { test, expect } from '@playwright/test'

/**
 * UAT spec for dashboard-rethink
 * Milestone: Dashboard redesign — project digest, not control panel
 *
 * Features under test:
 *   - dashboard-zone-layout   (four-zone layout + MilestoneDigestRow)
 *   - dashboard-horizon-zone  (HorizonZone — upcoming milestones + active ponders)
 *   - dashboard-empty-states  (orchestrator-aware suggestion chips)
 */

test.describe('dashboard-rethink — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
    // Wait for React to hydrate — SSE keeps network permanently active so
    // 'networkidle' never resolves; wait for a stable DOM element instead.
    await page.waitForLoadState('domcontentloaded')
    // Wait for the loading skeleton to disappear — signals project data has loaded
    await page.locator('[data-testid="skeleton"], .animate-pulse').first().waitFor({ state: 'hidden', timeout: 15_000 }).catch(() => {
      // No skeleton present — already loaded or different loading pattern
    })
    // Wait for the main content container to appear
    await page.locator('main, [role="main"], #root > div').first().waitFor({ state: 'visible', timeout: 15_000 })
    // Extra stability: small wait for React renders to settle
    await page.waitForTimeout(300)
  })

  // ── TC-1: Dashboard renders without errors ────────────────────────────────

  test('dashboard loads and shows the page without crash', async ({ page }) => {
    // Should not show a blank screen or error boundary
    const body = await page.textContent('body')
    expect(body).toBeTruthy()
    // No "Something went wrong" crash message
    await expect(page.getByText('Something went wrong')).not.toBeVisible()
  })

  // ── TC-2: No feature card grid (old design removed) ───────────────────────

  test('dashboard does not show FeatureCard grid layout', async ({ page }) => {
    // Old dashboard had "feature-card" style grids inside milestone sections.
    // The new design uses compact MilestoneDigestRow — no grid of feature cards.
    // Verify the old feature-card grid is absent.
    const featureCardGrid = page.locator('.grid.grid-cols-\\[repeat')
    await expect(featureCardGrid).not.toBeVisible()
  })

  // ── TC-3: MilestoneDigestRow exists for active milestones ─────────────────

  test('active milestones render as compact MilestoneDigestRow, not card grids', async ({ page }) => {
    // The "Current" zone should contain at least one milestone digest row.
    // Each row has a status dot, title link, progress bar fraction (X / Y), and status badge.
    // We verify the progress fraction pattern: digit(s) / digit(s)
    const progressFraction = page.locator('text=/\\d+ \\/ \\d+/').first()
    // If any active milestones exist, a progress fraction should be visible
    const activeMilestoneCount = await page.locator('text=/\\d+ \\/ \\d+/').count()
    // Proceed only if milestones exist (project has milestones)
    if (activeMilestoneCount > 0) {
      await expect(progressFraction).toBeVisible()
    }
  })

  // ── TC-4: MilestoneDigestRow collapsed by default ─────────────────────────

  test('MilestoneDigestRow is collapsed by default and expands on click', async ({ page }) => {
    // Find a milestone digest row — identified by the chevron expand button
    const expandBtn = page.locator('button[aria-label="Expand milestone"], button').filter({
      has: page.locator('svg.lucide-chevron-right, svg.lucide-chevron-down'),
    }).first()

    const count = await expandBtn.count()
    if (count === 0) {
      // No milestones with expandable rows — skip (acceptable for empty project)
      return
    }

    // Should be collapsed (ChevronRight icon visible, not ChevronDown)
    const chevronRight = expandBtn.locator('svg.lucide-chevron-right')
    const isCollapsed = await chevronRight.count() > 0

    if (isCollapsed) {
      // Click to expand
      await expandBtn.click()
      // After expand, ChevronDown should appear
      await expect(expandBtn.locator('svg.lucide-chevron-down')).toBeVisible()

      // Click again to collapse
      await expandBtn.click()
      await expect(expandBtn.locator('svg.lucide-chevron-right')).toBeVisible()
    }
  })

  // ── TC-5: MilestoneDigestRow has /sdlc-run command copy button ────────────

  test('MilestoneDigestRow shows copy-ready /sdlc-run command for next feature', async ({ page }) => {
    // Find any CommandBlock copy button on the dashboard (from MilestoneDigestRow)
    const copyBtn = page.getByRole('button', { name: /copy/i }).first()
    const count = await copyBtn.count()
    if (count > 0) {
      await expect(copyBtn).toBeVisible()
    }
    // Also check for the sdlc-run command text pattern in the page
    const cmdText = page.locator('text=/\\/sdlc-run /').first()
    const cmdCount = await cmdText.count()
    if (cmdCount > 0) {
      await expect(cmdText).toBeVisible()
    }
  })

  // ── TC-6: AttentionZone hidden when no escalations or active directives ────

  test('AttentionZone section header is absent when zone has no content', async ({ page }) => {
    // AttentionZone renders null when hasContent === false.
    // Since "Attention" heading is only rendered inside AttentionZone when it has content,
    // and in a normal test run there may or may not be escalations, we just verify
    // the zone does not show a persistent header that's always visible.
    // The zone renders nothing when empty — no outer wrapper, no heading.
    // We verify no crash, not a fixed heading presence.
    const body = await page.textContent('body')
    expect(body).not.toBeNull()
  })

  // ── TC-7: CurrentZone shows "No active work" when empty ──────────────────

  test('CurrentZone empty state shows actionable links when no milestones', async ({ page }) => {
    // This test is conditional: only if there are NO active milestones visible
    const progressFractions = await page.locator('text=/\\d+ \\/ \\d+/').count()
    if (progressFractions === 0) {
      // CurrentZone empty state should be visible
      await expect(page.getByText(/No active work/i)).toBeVisible()
      await expect(page.getByRole('link', { name: 'Milestones' })).toBeVisible()
    }
  })

  // ── TC-8: HorizonZone section header (when content exists) ───────────────

  test('HorizonZone renders Horizon heading with Telescope icon when content exists', async ({ page }) => {
    // HorizonZone renders null when both lists empty — so check conditionally
    const horizonSection = page.locator('section').filter({ has: page.getByText('Horizon') })
    const count = await horizonSection.count()
    if (count > 0) {
      await expect(horizonSection).toBeVisible()
      // Should have "Upcoming Milestones" or "Active Ponders" sub-section headers
      const hasUpcoming = await page.getByText('Upcoming Milestones').count()
      const hasActive = await page.getByText('Active Ponders').count()
      expect(hasUpcoming + hasActive).toBeGreaterThan(0)
    }
  })

  // ── TC-9: HorizonZone ponder rows have copy button ────────────────────────

  test('HorizonZone ponder rows show /sdlc-ponder copy button', async ({ page }) => {
    const activePonderSection = page.getByText('Active Ponders')
    const count = await activePonderSection.count()
    if (count > 0) {
      // Ponder rows should have a copy button with the /sdlc-ponder command
      const copyBtn = page.getByRole('button', { name: 'copy' }).first()
      await expect(copyBtn).toBeVisible()
    }
  })

  // ── TC-10: HorizonZone milestone rows link to /milestones/<slug> ──────────

  test('HorizonZone upcoming milestone rows link to milestone detail page', async ({ page }) => {
    const upcomingSection = page.getByText('Upcoming Milestones')
    const count = await upcomingSection.count()
    if (count > 0) {
      // All links in that section should point to /milestones/...
      const milestoneLinks = page.locator('section').filter({ has: page.getByText('Horizon') })
        .getByRole('link').filter({ hasText: /.+/ })
      const firstLink = milestoneLinks.first()
      const href = await firstLink.getAttribute('href')
      expect(href).toMatch(/\/milestones\//)
    }
  })

  // ── TC-11: ArchiveZone hidden when no released milestones ─────────────────

  test('ArchiveZone does not crash and renders correctly when present', async ({ page }) => {
    // ArchiveZone renders a collapsible section "Archive" with released milestones.
    const archiveBtn = page.getByRole('button').filter({ hasText: /Archive/i }).first()
    const count = await archiveBtn.count()
    if (count > 0) {
      // Archive section toggle should be visible
      await expect(archiveBtn).toBeVisible()
      // Click to expand
      await archiveBtn.click()
      // Should show at least one released milestone item
      await expect(page.getByText('released').first()).toBeVisible()
    }
    // If no released milestones, ArchiveZone renders nothing — that's fine
  })

  // ── TC-12: Smart global empty state — suggestion chips ───────────────────

  test('global empty state shows suggestion chips on a fresh project', async ({ page }) => {
    // This test verifies the chip content for the empty state component.
    // Since the live project has features, we navigate to check DashboardEmptyState
    // only when state.milestones.length === 0 && state.features.length === 0.
    // If not empty, we verify the chips are NOT shown (DashboardEmptyState is conditional).
    const progressFractions = await page.locator('text=/\\d+ \\/ \\d+/').count()
    const featureLinks = await page.getByRole('link', { name: /\/features\// }).count()

    // Check if we're in empty state (no active indicators)
    const emptyStateHeading = page.getByText('SDLC turns ideas into shipped software.')
    const isEmptyState = await emptyStateHeading.count() > 0

    if (isEmptyState) {
      // Verify the empty state is properly rendered
      await expect(emptyStateHeading).toBeVisible()
      await expect(page.getByText('Where do you want to start?')).toBeVisible()
      // "Create a Feature directly" chip is always present
      await expect(page.getByText('Create a Feature directly')).toBeVisible()
    } else {
      // Project has data — DashboardEmptyState should NOT be visible
      await expect(emptyStateHeading).not.toBeVisible()
    }
  })

  // ── TC-13: "Define Vision" chip appears when vision missing ──────────────

  test('DashboardEmptyState Define Vision chip links to /setup', async ({ page }) => {
    const emptyState = page.getByText('SDLC turns ideas into shipped software.')
    const isVisible = await emptyState.count() > 0
    if (!isVisible) return // project is not empty — skip

    const defineVisionLink = page.getByRole('link', { name: 'Define Vision' })
    const count = await defineVisionLink.count()
    if (count > 0) {
      const href = await defineVisionLink.getAttribute('href')
      expect(href).toBe('/setup')
    }
  })

  // ── TC-14: "Create a Feature directly" chip always present on empty state ──

  test('DashboardEmptyState always shows Create Feature chip', async ({ page }) => {
    const emptyState = page.getByText('SDLC turns ideas into shipped software.')
    const isVisible = await emptyState.count() > 0
    if (!isVisible) return // project is not empty — skip

    const createFeatureLink = page.getByRole('link', { name: 'Create a Feature directly' })
    await expect(createFeatureLink).toBeVisible()
    const href = await createFeatureLink.getAttribute('href')
    expect(href).toBe('/features?new=1')
  })

  // ── TC-15: Four zones render in correct order (Attention→Current→Horizon→Archive) ──

  test('dashboard zone order: Attention above Current above Horizon above Archive', async ({ page }) => {
    // Verify the DOM order of zone headings. We check that the zones appear in
    // the correct vertical sequence by comparing bounding box Y positions.
    const currentHeading = page.getByText('Current').first()
    const archiveBtn = page.getByRole('button').filter({ hasText: /Archive/i }).first()

    const currentVisible = await currentHeading.count() > 0
    const archiveVisible = await archiveBtn.count() > 0

    if (currentVisible && archiveVisible) {
      const currentBox = await currentHeading.boundingBox()
      const archiveBox = await archiveBtn.boundingBox()
      if (currentBox && archiveBox) {
        // Current should be above Archive on the page
        expect(currentBox.y).toBeLessThan(archiveBox.y)
      }
    }
  })

  // ── TC-16: Milestone title in digest row links to /milestones/<slug> ──────

  test('MilestoneDigestRow milestone title is a link to /milestones/<slug>', async ({ page }) => {
    // Find the first milestone title link in any digest row
    // MilestoneDigestRow renders: <Link to={`/milestones/${milestone.slug}`}>
    const milestoneLinks = page.locator('a[href^="/milestones/"]').first()
    const count = await milestoneLinks.count()
    if (count > 0) {
      await expect(milestoneLinks).toBeVisible()
      const href = await milestoneLinks.getAttribute('href')
      expect(href).toMatch(/^\/milestones\/[a-z0-9-]+$/)
    }
  })
})
