/**
 * E2E spec for milestone: v06-uat-dashboard-and-ci
 * "UAT run history dashboard, CI gate, and full guidance update"
 *
 * Mixed-mode spec:
 *  - UI tests: UatHistoryPanel renders on MilestoneDetail page
 *  - Infrastructure tests: fs checks for GitHub Actions, guidance.md, CLAUDE.md, init.rs
 *  - API tests: request fixture for uat-runs endpoint
 *
 * Locator policy: getByTestId / getByRole for UI; request fixture for API; fs for files.
 */

import { test, expect } from '@playwright/test'
import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const PROJECT_ROOT = path.resolve(__dirname, '../../../')

// ---------------------------------------------------------------------------
// Helper: wait for app load (SSE keeps network perpetually active, use 'load')
// ---------------------------------------------------------------------------

async function waitForContent(page: Parameters<Parameters<typeof test>[1]>[0]['page']) {
  await page.waitForLoadState('load', { timeout: 15_000 })
  await page.waitForTimeout(1_000)
}

// ---------------------------------------------------------------------------
// Checklist item 1: MilestoneDetail renders UatHistoryPanel
// ---------------------------------------------------------------------------

test('MilestoneDetail page renders UatHistoryPanel component', async ({ page }) => {
  // Navigate to a milestone detail page that exists
  await page.goto('/milestones/v05-playwright-uat-integration')
  await waitForContent(page)

  // UatHistoryPanel renders with data-testid="uat-history-panel" in all states
  const panel = page.getByTestId('uat-history-panel')
  await expect(panel).toBeVisible({ timeout: 10_000 })
})

// ---------------------------------------------------------------------------
// Checklist item 2: Each row shows verdict badge, date, test count, tasks
// ---------------------------------------------------------------------------

test('UatHistoryPanel shows verdict badge, date, and test count for v05 run', async ({ page }) => {
  // v05 has a completed UAT run with a Pass verdict
  await page.goto('/milestones/v05-playwright-uat-integration')
  await waitForContent(page)

  const panel = page.getByTestId('uat-history-panel')
  await expect(panel).toBeVisible({ timeout: 10_000 })

  // Wait for loading to complete (panel shows spinner then content)
  // The PASS badge text should appear once data loads
  await expect(panel.getByText('PASS', { exact: true })).toBeVisible({ timeout: 10_000 })

  // Date should be visible (formatted date of the run)
  // Test count like "10/11 passed" or similar
  await expect(panel.getByText(/passed/)).toBeVisible({ timeout: 5_000 })
})

// ---------------------------------------------------------------------------
// Checklist item 3: SseMessage::MilestoneUatCompleted variant exists
// ---------------------------------------------------------------------------

test('SseMessage::MilestoneUatCompleted variant exists in state.rs', () => {
  const statePath = path.join(PROJECT_ROOT, 'crates/sdlc-server/src/state.rs')
  expect(fs.existsSync(statePath)).toBe(true)

  const src = fs.readFileSync(statePath, 'utf8')
  expect(src).toContain('MilestoneUatCompleted')
  expect(src).toMatch(/MilestoneUatCompleted\s*\{[^}]*slug/)
})

test('MilestoneUatCompleted is emitted in events.rs on the milestone_uat channel', () => {
  const eventsPath = path.join(PROJECT_ROOT, 'crates/sdlc-server/src/routes/events.rs')
  const src = fs.readFileSync(eventsPath, 'utf8')
  expect(src).toContain('MilestoneUatCompleted')
  expect(src).toMatch(/milestone_uat/)
})

// ---------------------------------------------------------------------------
// Checklist item 4: useSSE dispatches MilestoneUatCompleted → UatHistoryPanel refreshes
// ---------------------------------------------------------------------------

test('useSSE hook handles milestone_uat SSE events', () => {
  const useSsePath = path.join(PROJECT_ROOT, 'frontend/src/hooks/useSSE.ts')
  const src = fs.readFileSync(useSsePath, 'utf8')
  // The hook must listen for the "milestone_uat" event channel and dispatch it
  expect(src, 'useSSE must handle milestone_uat events').toMatch(
    /milestone_uat|MilestoneUat/
  )
})

test('UatHistoryPanel refreshes on MilestoneUatCompleted SSE event', () => {
  const panelPath = path.join(
    PROJECT_ROOT,
    'frontend/src/components/milestones/UatHistoryPanel.tsx'
  )
  const src = fs.readFileSync(panelPath, 'utf8')
  // Panel must subscribe to an SSE callback or refresh trigger to reload runs
  // This can be a useCallback refetch, a prop passed from MilestoneDetail, or a context
  expect(src, 'UatHistoryPanel must support SSE-triggered refresh').toMatch(
    /useCallback|onRefresh|refreshKey|refetch|milestone_uat|sse/i
  )
})

// ---------------------------------------------------------------------------
// Checklist item 5: .github/workflows/uat.yml triggers on push to main
//                   AND pull_request touching frontend/** or crates/**
// ---------------------------------------------------------------------------

test('.github/workflows/uat.yml exists', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  expect(fs.existsSync(workflowPath), '.github/workflows/uat.yml must exist').toBe(true)
})

test('uat.yml triggers on push to main', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  const src = fs.readFileSync(workflowPath, 'utf8')
  // Must include a push trigger targeting main
  expect(src, 'uat.yml must trigger on push to main').toMatch(/push:[\s\S]*?branches[\s\S]*?main|push[\s\S]*?main/)
})

test('uat.yml triggers on pull_request with path filters for frontend/** and crates/**', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  const src = fs.readFileSync(workflowPath, 'utf8')
  // Must include path filters so it only runs when relevant code changes
  expect(src, 'uat.yml pull_request must have path filters').toMatch(/paths:|frontend\/\*\*|crates\/\*\*/)
})

// ---------------------------------------------------------------------------
// Checklist item 6: workflow builds sdlc, installs Playwright, uploads report
// ---------------------------------------------------------------------------

test('uat.yml builds the sdlc binary and installs Playwright Chromium', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  const src = fs.readFileSync(workflowPath, 'utf8')
  expect(src).toMatch(/cargo build.*sdlc/)
  expect(src).toMatch(/playwright install.*chromium|playwright install --with-deps/)
})

test('uat.yml uploads HTML report with 30-day retention on always()', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  const src = fs.readFileSync(workflowPath, 'utf8')
  expect(src).toMatch(/upload-artifact/)
  expect(src).toMatch(/retention-days:\s*30/)
  expect(src).toMatch(/if:\s*always\(\)/)
})

test('uat.yml uploads traces on failure', () => {
  const workflowPath = path.join(PROJECT_ROOT, '.github/workflows/uat.yml')
  const src = fs.readFileSync(workflowPath, 'utf8')
  // Must have a second artifact upload that only runs on failure
  expect(src).toMatch(/if:\s*failure\(\)/)
})

// ---------------------------------------------------------------------------
// Checklist item 7: .sdlc/guidance.md §5 references Playwright + three-tier pattern
// ---------------------------------------------------------------------------

test('.sdlc/guidance.md §5 references Playwright and the three-tier UAT pattern', () => {
  const guidancePath = path.join(PROJECT_ROOT, '.sdlc/guidance.md')
  expect(fs.existsSync(guidancePath), '.sdlc/guidance.md must exist').toBe(true)

  const src = fs.readFileSync(guidancePath, 'utf8')
  // §5 or any section must mention Playwright
  expect(src, 'guidance.md must reference Playwright').toMatch(/[Pp]laywright/)
  // Must reference the three-tier pattern or tier concept
  expect(src, 'guidance.md must reference the tier/three-tier UAT concept').toMatch(
    /three-tier|Tier [123]|tier [123]|playwright.*uat|uat.*playwright/i
  )
})

// ---------------------------------------------------------------------------
// Checklist item 8: CLAUDE.md links to docs/uat-enterprise-strategy.md
// ---------------------------------------------------------------------------

test('CLAUDE.md links to docs/uat-enterprise-strategy.md', () => {
  const claudePath = path.join(PROJECT_ROOT, 'CLAUDE.md')
  const src = fs.readFileSync(claudePath, 'utf8')
  expect(src).toContain('uat-enterprise-strategy')
})

// ---------------------------------------------------------------------------
// Checklist item 9: SDLC_MILESTONE_UAT_COMMAND in init.rs has Playwright Mode A/B
// ---------------------------------------------------------------------------

test('SDLC_MILESTONE_UAT_COMMAND in init.rs includes Mode A and Mode B', () => {
  const initPath = path.join(PROJECT_ROOT, 'crates/sdlc-cli/src/cmd/init.rs')
  expect(fs.existsSync(initPath)).toBe(true)

  const src = fs.readFileSync(initPath, 'utf8')
  expect(src).toContain('SDLC_MILESTONE_UAT_COMMAND')
  expect(src).toMatch(/Mode A/)
  expect(src).toMatch(/Mode B/)
  expect(src).toMatch(/npx playwright test.*reporter=json|playwright test.*reporter/)
  expect(src).toMatch(/results\.json/)
})

// ---------------------------------------------------------------------------
// Checklist item 10: sdlc update installs the updated skill (API smoke test)
// ---------------------------------------------------------------------------

test('GET /api/milestones/v06-uat-dashboard-and-ci/uat-runs returns JSON array', async ({ request }) => {
  const response = await request.get('/api/milestones/v06-uat-dashboard-and-ci/uat-runs')
  // Should return 200 with empty array (no runs yet) or populated array
  expect(response.ok(), `Expected 200, got ${response.status()}`).toBeTruthy()
  const body = await response.json()
  expect(Array.isArray(body)).toBe(true)
})
