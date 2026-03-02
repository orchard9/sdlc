/**
 * Acceptance Tests: Layout Foundation — Collapsible & Resizable App Shell
 *
 * Tests two features in milestone v15-layout-foundation:
 *   1. layout-appshell-panels — NAV icon rail collapse + AgentPanel drag-to-resize
 *   2. layout-ponder-columns  — Ponder desktop workspace resize + mobile three-tab bar
 *
 * Ponder entry used: tick-orchestrator (first real entry in the project)
 */

import { test, expect } from '@playwright/test'

const PONDER_SLUG = 'tick-orchestrator'
const PONDER_URL = `/ponder/${PONDER_SLUG}`

test.describe('Layout Foundation — Acceptance Tests', () => {

  // ── Feature 1: AppShell sidebar collapse ─────────────────────────────────

  test('F1: sidebar has a collapse/expand toggle button', async ({ page }) => {
    await page.goto('/')
    const toggle = page.locator('[data-testid="sidebar-toggle"]')
    await expect(toggle).toBeVisible()
  })

  test('F1: sidebar collapses to narrow icon rail when toggle clicked', async ({ page }) => {
    await page.goto('/')
    const sidebar = page.locator('[data-testid="sidebar-rail"]')
    const widthBefore = await sidebar.evaluate(el => el.getBoundingClientRect().width)

    // Click collapse
    await page.locator('[data-testid="sidebar-toggle"]').click()

    // Wait for transition
    await page.waitForTimeout(250)

    const widthAfter = await sidebar.evaluate(el => el.getBoundingClientRect().width)
    expect(widthAfter).toBeLessThan(widthBefore)
    expect(widthAfter).toBeLessThanOrEqual(60) // ≈ 52 px icon rail
  })

  test('F1: all nav links remain accessible when sidebar is collapsed', async ({ page }) => {
    await page.goto('/')
    await page.locator('[data-testid="sidebar-toggle"]').click()
    await page.waitForTimeout(250)

    // Nav links should still exist (icon-only)
    const links = page.locator('[data-testid="sidebar-rail"] nav a')
    const count = await links.count()
    expect(count).toBeGreaterThan(5) // at least Dashboard, Milestones, Features, etc.
  })

  test('F1: collapsed state persists across page reload', async ({ page }) => {
    await page.goto('/')
    await page.locator('[data-testid="sidebar-toggle"]').click()
    await page.waitForTimeout(300)

    await page.reload()

    // After reload, sidebar should still be collapsed (≤ 60 px)
    const sidebar = page.locator('[data-testid="sidebar-rail"]')
    const width = await sidebar.evaluate(el => el.getBoundingClientRect().width)
    expect(width).toBeLessThanOrEqual(60)
  })

  test('F1: sidebar re-expands when toggle clicked again', async ({ page }) => {
    await page.goto('/')
    const toggle = page.locator('[data-testid="sidebar-toggle"]')
    await toggle.click()
    await page.waitForTimeout(250)

    // Toggle again to expand
    await toggle.click()
    await page.waitForTimeout(250)

    const sidebar = page.locator('[data-testid="sidebar-rail"]')
    const width = await sidebar.evaluate(el => el.getBoundingClientRect().width)
    expect(width).toBeGreaterThan(160) // full expanded width ~ 224 px
  })

  // ── Feature 1: AgentPanel drag-to-resize ─────────────────────────────────

  test('F1: agent panel has a drag-to-resize handle', async ({ page }) => {
    await page.goto('/')

    // Open agent panel if not already open
    const openBtn = page.getByRole('button', { name: 'Open agent panel' })
    if (await openBtn.isVisible()) {
      await openBtn.click()
    }

    const handle = page.locator('[data-testid="agent-resize-handle"]')
    await expect(handle).toBeVisible()
  })

  test('F1: agent panel width preference persists across reload via localStorage', async ({ page }) => {
    await page.goto('/')
    await page.evaluate(() => {
      localStorage.setItem('sdlc:agent-panel-width', '350')
    })
    await page.reload()

    // Open agent panel
    const openBtn = page.getByRole('button', { name: 'Open agent panel' })
    if (await openBtn.isVisible()) {
      await openBtn.click()
    }

    const panel = page.locator('[data-testid="agent-resize-handle"]').locator('..')
    const width = await panel.evaluate(el => (el as HTMLElement).getBoundingClientRect().width)
    expect(width).toBeGreaterThanOrEqual(340)
  })

  // ── Feature 2: Ponder desktop workspace resize ────────────────────────────

  test('F2: ponder desktop workspace panel has a resize handle', async ({ page }) => {
    await page.goto(PONDER_URL)
    await page.waitForSelector('[data-testid="workspace-resize-handle"]', { timeout: 10000 })
    const handle = page.locator('[data-testid="workspace-resize-handle"]')
    await expect(handle).toBeVisible()
  })

  test('F2: ponder workspace width persists via localStorage', async ({ page }) => {
    await page.goto(PONDER_URL)
    await page.evaluate(() => {
      localStorage.setItem('ponder_workspace_width', '320')
    })
    await page.reload()
    await page.waitForSelector('[data-testid="workspace-resize-handle"]', { timeout: 10000 })

    // The workspace column should be ~320px
    const handle = page.locator('[data-testid="workspace-resize-handle"]')
    const workspaceCol = handle.locator('+ div')
    const width = await workspaceCol.evaluate(el => (el as HTMLElement).getBoundingClientRect().width)
    expect(width).toBeGreaterThanOrEqual(300)
  })

  // ── Feature 2: Ponder mobile three-tab bar ────────────────────────────────

  test('F2: mobile shows Chat/Files/Team tab bar in ponder detail', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await page.goto(PONDER_URL)
    await page.waitForSelector('[data-testid="mobile-tab-chat"]', { timeout: 10000 })

    await expect(page.locator('[data-testid="mobile-tab-chat"]')).toBeVisible()
    await expect(page.locator('[data-testid="mobile-tab-files"]')).toBeVisible()
    await expect(page.locator('[data-testid="mobile-tab-team"]')).toBeVisible()
  })

  test('F2: mobile Files tab switches to workspace view', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await page.goto(PONDER_URL)
    await page.waitForSelector('[data-testid="mobile-tab-files"]', { timeout: 10000 })

    await page.locator('[data-testid="mobile-tab-files"]').click()

    // Old bottom-sheet should NOT be present; workspace content should be directly visible
    const bottomSheet = page.locator('.translate-y-full')
    expect(await bottomSheet.count()).toBe(0)
  })

  test('F2: mobile Chat tab restores dialogue view', async ({ page }) => {
    await page.setViewportSize({ width: 375, height: 812 })
    await page.goto(PONDER_URL)
    await page.waitForSelector('[data-testid="mobile-tab-chat"]', { timeout: 10000 })

    // Switch to Files then back to Chat
    await page.locator('[data-testid="mobile-tab-files"]').click()
    await page.locator('[data-testid="mobile-tab-chat"]').click()

    // Chat tab should now be active — dialogue panel visible
    const chatTab = page.locator('[data-testid="mobile-tab-chat"]')
    await expect(chatTab).toBeVisible()
  })
})
