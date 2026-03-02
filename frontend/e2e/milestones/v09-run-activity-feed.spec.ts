import { test, expect } from '@playwright/test'

/**
 * UAT: v09-run-activity-feed
 *
 * Tests the run activity feed UI against run 20260302-020808-lrr
 * which has a synthetic .events.json sidecar with known content.
 *
 * Event format (as produced by message_to_event in runs.rs):
 *   init: { type, model, tools_count, mcp_servers[] }
 *   assistant: { type, text, tools[], thinking[] }
 *   user: { type, tool_results[] }
 *   result: { type, cost_usd, turns }
 */

const TEST_RUN_ID = '20260302-020808-lrr'
const APP_URL = 'http://localhost:7777'

test.describe('Run Activity Feed — v09-run-activity-feed UAT', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(APP_URL)
  })

  test('completed run shows activity feed with init card', async ({ page }) => {
    // Expand the UAT run in the activity panel
    await page.getByRole('button', { name: /UAT: v01-directive-core 07:08/ }).click()

    // Init card should show "Run started" with model info
    await expect(page.getByText('Run started')).toBeVisible()
    await expect(page.getByText('claude-sonnet-4-6')).toBeVisible()
    await expect(page.getByText('12 tools')).toBeVisible()
    await expect(page.getByText(/MCP: sdlc/)).toBeVisible()
  })

  test('activity feed shows tool call cards with tool names', async ({ page }) => {
    await page.getByRole('button', { name: /UAT: v01-directive-core 07:08/ }).click()

    // Tool call cards should show tool names
    await expect(page.getByText('mcp__playwright__browser_navigate')).toBeVisible()
    await expect(page.getByText('Bash')).toBeVisible()
    await expect(page.getByText('mcp__sdlc__sdlc_feature_show')).toBeVisible()
  })

  test('tool call card expands to show input JSON', async ({ page }) => {
    await page.getByRole('button', { name: /UAT: v01-directive-core 07:08/ }).click()

    // Click the first "show input" button
    const showInputBtn = page.getByRole('button', { name: 'show input' }).first()
    await expect(showInputBtn).toBeVisible()
    await showInputBtn.click()

    // Input JSON should expand
    await expect(page.getByText(/"url"/).first()).toBeVisible()
    await expect(page.getByRole('button', { name: 'hide input' }).first()).toBeVisible()

    // Collapse it again
    await page.getByRole('button', { name: 'hide input' }).first().click()
    await expect(page.getByRole('button', { name: 'show input' }).first()).toBeVisible()
  })

  test('activity feed shows assistant text blocks between tool calls', async ({ page }) => {
    await page.getByRole('button', { name: /UAT: v01-directive-core 07:08/ }).click()

    await expect(page.getByText("I'll run the UAT for v01-directive-core milestone by navigating to the app.")).toBeVisible()
    await expect(page.getByText('Navigated to the app. Let me check the milestone status now.')).toBeVisible()
  })

  test('run result card shows cost and turn count', async ({ page }) => {
    await page.getByRole('button', { name: /UAT: v01-directive-core 07:08/ }).click()

    // Result card shows cost and turns
    await expect(page.getByText('Run completed')).toBeVisible()
    await expect(page.getByText('0.0342')).toBeVisible()
    await expect(page.getByText('4 turns')).toBeVisible()
  })

  test('telemetry API returns correct structure for run', async ({ page }) => {
    const response = await page.evaluate(async (runId) => {
      const res = await fetch(`/api/runs/${runId}/telemetry`)
      return await res.json()
    }, TEST_RUN_ID)

    expect(response.run_id).toBe(TEST_RUN_ID)
    expect(Array.isArray(response.events)).toBe(true)
    expect(response.events.length).toBeGreaterThanOrEqual(1)

    const initEvent = response.events.find((e: { type: string }) => e.type === 'init')
    expect(initEvent).toBeDefined()
    expect(initEvent.model).toBe('claude-sonnet-4-6')
  })
})
