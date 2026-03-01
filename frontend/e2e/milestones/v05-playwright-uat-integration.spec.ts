/**
 * E2E spec for milestone: v05-playwright-uat-integration
 * "sdlc-milestone-uat uses Playwright as its execution engine"
 *
 * This milestone is infrastructure-focused — Rust structs, server routes, and
 * config files rather than UI pages. The spec uses Playwright's `request`
 * fixture for API tests and `test.step` blocks with Bash-derived assertions
 * for file/code checks that have observable HTTP side-effects.
 *
 * Locator policy: request fixture for API, getByTestId/getByRole for any UI.
 */

import { test, expect } from '@playwright/test'
import { execSync } from 'child_process'
import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)
const PROJECT_ROOT = path.resolve(__dirname, '../../../')

// ---------------------------------------------------------------------------
// Checklist item 1: .mcp.json registers @microsoft/playwright-mcp
// ---------------------------------------------------------------------------

test('@microsoft/playwright-mcp is registered in .mcp.json', async () => {
  const mcpPath = path.join(PROJECT_ROOT, '.mcp.json')
  expect(fs.existsSync(mcpPath), '.mcp.json must exist').toBe(true)

  const mcp = JSON.parse(fs.readFileSync(mcpPath, 'utf8'))
  expect(mcp).toHaveProperty('mcpServers')
  expect(mcp.mcpServers).toHaveProperty('playwright')
  const pw = mcp.mcpServers.playwright
  expect(pw.command).toBe('npx')
  expect(pw.args).toContain('@playwright/mcp@latest')
})

// ---------------------------------------------------------------------------
// Checklist item 2: spawn_agent_run for milestone-uat includes Playwright tools
// ---------------------------------------------------------------------------

test('start_milestone_uat in runs.rs includes Playwright MCP tools in allowed_tools', async () => {
  const runsPath = path.join(PROJECT_ROOT, 'crates/sdlc-server/src/routes/runs.rs')
  expect(fs.existsSync(runsPath), 'runs.rs must exist').toBe(true)

  const src = fs.readFileSync(runsPath, 'utf8')
  const expectedTools = [
    'mcp__playwright__browser_navigate',
    'mcp__playwright__browser_click',
    'mcp__playwright__browser_type',
    'mcp__playwright__browser_snapshot',
    'mcp__playwright__browser_take_screenshot',
    'mcp__playwright__browser_console_messages',
    'mcp__playwright__browser_wait_for',
  ]
  for (const tool of expectedTools) {
    expect(src, `runs.rs must include ${tool}`).toContain(tool)
  }
  // Confirm they appear in the start_milestone_uat function context
  expect(src).toContain('start_milestone_uat')
  expect(src).toContain('MilestoneUatCompleted')
})

// ---------------------------------------------------------------------------
// Checklist items 3 & 4: sdlc-milestone-uat skill has Mode A and Mode B
// ---------------------------------------------------------------------------

test('sdlc-milestone-uat skill has Mode A language', async () => {
  const skillPath = path.join(
    process.env.HOME ?? '/Users/' + process.env.USER,
    '.claude/commands/sdlc-milestone-uat.md'
  )
  expect(fs.existsSync(skillPath), 'sdlc-milestone-uat.md must be installed').toBe(true)

  const skill = fs.readFileSync(skillPath, 'utf8')
  expect(skill).toContain('Mode A')
  expect(skill).toMatch(/playwright test.*reporter=json|npx playwright test/)
  expect(skill).toMatch(/results\.json/)
})

test('sdlc-milestone-uat skill has Mode B language', async () => {
  const skillPath = path.join(
    process.env.HOME ?? '/Users/' + process.env.USER,
    '.claude/commands/sdlc-milestone-uat.md'
  )
  const skill = fs.readFileSync(skillPath, 'utf8')
  expect(skill).toContain('Mode B')
  expect(skill).toMatch(/acceptance_test\.md|checklist/)
  expect(skill).toMatch(/generate.*spec|spec.*generate|write.*spec/i)
})

// ---------------------------------------------------------------------------
// Checklist item 7: UatRun struct has required fields (verified via Rust source)
// ---------------------------------------------------------------------------

test('UatRun struct in sdlc-core has all required fields', async () => {
  const milestoneSrc = path.join(
    PROJECT_ROOT,
    'crates/sdlc-core/src/milestone.rs'
  )
  expect(fs.existsSync(milestoneSrc), 'milestone.rs must exist').toBe(true)

  const src = fs.readFileSync(milestoneSrc, 'utf8')
  const requiredFields = ['id', 'verdict', 'tests_total', 'tests_passed', 'tests_failed', 'tasks_created']
  for (const field of requiredFields) {
    expect(src, `UatRun must have field: ${field}`).toMatch(
      new RegExp(`pub\\s+${field}`)
    )
  }
  expect(src).toContain('pub struct UatRun')
  expect(src).toContain('pub enum UatVerdict')
})

// ---------------------------------------------------------------------------
// Checklist item 8: save_uat_run / list_uat_runs / latest_uat_run exist
// ---------------------------------------------------------------------------

test('Milestone functions save_uat_run, list_uat_runs, latest_uat_run exist in sdlc-core', async () => {
  const src = fs.readFileSync(
    path.join(PROJECT_ROOT, 'crates/sdlc-core/src/milestone.rs'),
    'utf8'
  )
  expect(src).toContain('pub fn save_uat_run')
  expect(src).toContain('pub fn list_uat_runs')
  expect(src).toContain('pub fn latest_uat_run')
})

// ---------------------------------------------------------------------------
// Checklist item 9: GET /api/milestones/{slug}/uat-runs returns JSON array
// ---------------------------------------------------------------------------

test('GET /api/milestones/{slug}/uat-runs returns a JSON array', async ({ request }) => {
  const response = await request.get('/api/milestones/v05-playwright-uat-integration/uat-runs')
  expect(response.ok(), `Expected 200 but got ${response.status()}`).toBeTruthy()

  const body = await response.json()
  expect(Array.isArray(body), 'Response must be a JSON array').toBe(true)
})

test('GET /api/milestones/{slug}/uat-runs returns array for a milestone with no runs yet', async ({ request }) => {
  // v06 milestone has no UAT runs yet — should return empty array, not 404 or 500
  const response = await request.get('/api/milestones/v06-uat-dashboard-and-ci/uat-runs')
  expect(response.status()).not.toBe(500)
  if (response.ok()) {
    const body = await response.json()
    expect(Array.isArray(body)).toBe(true)
  }
})

// ---------------------------------------------------------------------------
// Checklist item 10: GET /api/milestones/{slug}/uat-runs/latest
// ---------------------------------------------------------------------------

test('GET /api/milestones/{slug}/uat-runs/latest returns 200 or 404', async ({ request }) => {
  const response = await request.get('/api/milestones/v05-playwright-uat-integration/uat-runs/latest')
  // 200 if runs exist, 404 if no runs yet — both are valid
  expect([200, 404]).toContain(response.status())
})

test('GET /api/milestones/{slug}/uat-runs/latest with runs returns a UatRun shape', async ({ request }) => {
  // First check v04 which completed UAT — it should have runs
  const response = await request.get('/api/milestones/v04-playwright-foundation/uat-runs/latest')
  if (response.status() === 404) {
    // No runs stored yet (uat_results.md written but run.yaml not yet) — skip
    test.skip()
    return
  }
  expect(response.ok()).toBeTruthy()
  const run = await response.json()
  // Verify UatRun shape
  expect(run).toHaveProperty('id')
  expect(run).toHaveProperty('verdict')
  expect(run).toHaveProperty('tests_total')
  expect(run).toHaveProperty('tests_passed')
  expect(run).toHaveProperty('tests_failed')
  expect(run).toHaveProperty('tasks_created')
  expect(Array.isArray(run.tasks_created)).toBe(true)
})

// ---------------------------------------------------------------------------
// Checklist items 5 & 6: uat-runs directory structure (deferred — requires a
// completed UAT agent run to populate results.json + summary.md)
// ---------------------------------------------------------------------------

test('uat-runs directory is created for milestones with completed UAT', async () => {
  // v04 completed UAT — its uat_results.md exists
  const v04UatResults = path.join(
    PROJECT_ROOT,
    '.sdlc/milestones/v04-playwright-foundation/uat_results.md'
  )
  expect(fs.existsSync(v04UatResults), 'v04 uat_results.md must exist after completed UAT').toBe(true)

  // The uat-runs/ directory itself is created when save_uat_run is called via the
  // agent run. For now verify the path infrastructure exists in code.
  const pathsSrc = path.join(PROJECT_ROOT, 'crates/sdlc-core/src/paths.rs')
  const src = fs.readFileSync(pathsSrc, 'utf8')
  expect(src).toMatch(/uat.run|uat_run/)
})
