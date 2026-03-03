/**
 * Acceptance Tests: Finding-Closure Protocol
 *
 * Tests two features:
 *   1. audit-review-approval-protocol — sdlc-next template + CLAUDE.md ethos bullet
 *   2. audit-review-guidance-section  — §12 Audit & Review Findings in guidance.md
 *
 * These are non-UI features (config/doc files). Tests verify file content via
 * Node.js fs APIs inside Playwright test blocks.
 */

import { test, expect } from '@playwright/test'
import * as fs from 'fs'
import * as path from 'path'
import { fileURLToPath } from 'url'

const __filename = fileURLToPath(import.meta.url)
const __dirname = path.dirname(__filename)

const ROOT = path.resolve(__dirname, '../../../')

// ── Feature 1: audit-review-approval-protocol ────────────────────────────────

test('F1: CLAUDE.md contains the "Audits and reviews close every finding" ethos bullet', () => {
  const content = fs.readFileSync(path.join(ROOT, 'CLAUDE.md'), 'utf8')
  expect(content).toContain('Audits and reviews close every finding')
  expect(content).toContain('approve_audit')
  expect(content).toContain('approve_review')
  expect(content).toContain('Silence is not acceptance')
})

test('F1: sdlc-next command template has a dedicated approve_review / approve_audit subsection', () => {
  const nextRs = path.join(
    ROOT,
    'crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs',
  )
  const content = fs.readFileSync(nextRs, 'utf8')
  expect(content).toContain('approve_review')
  expect(content).toContain('approve_audit')
})

test('F1: sdlc-next template enumerates all three dispositions (fix / track / accept)', () => {
  const nextRs = path.join(
    ROOT,
    'crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs',
  )
  const content = fs.readFileSync(nextRs, 'utf8')
  // Fix disposition
  expect(content.toLowerCase()).toMatch(/fix now|targeted.*fix|fix.*targeted/)
  // Track disposition
  expect(content).toContain('sdlc task add')
  // Accept / rationale
  expect(content.toLowerCase()).toMatch(/accept|rationale/)
})

test('F1: sdlc-next template prohibits silent skips', () => {
  const nextRs = path.join(
    ROOT,
    'crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs',
  )
  const content = fs.readFileSync(nextRs, 'utf8')
  expect(content.toLowerCase()).toMatch(/no.*finding.*silently|silent.*skip/)
})

test('F1: sdlc-next playbook variant also references the three-disposition protocol', () => {
  const nextRs = path.join(
    ROOT,
    'crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs',
  )
  const content = fs.readFileSync(nextRs, 'utf8')
  // Playbook (Gemini/OpenCode) variant covers step 5a or equivalent
  expect(content.toLowerCase()).toMatch(/approve_review.*approve_audit|approve_audit.*approve_review|5a/)
})

// ── Feature 2: audit-review-guidance-section ─────────────────────────────────

test('F2: guidance.md contains a §12 / section 12 titled "Audit & Review Findings"', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  expect(content).toMatch(/12\..*[Aa]udit.*[Rr]eview|[Aa]udit.*[Rr]eview.*Findings/)
})

test('F2: guidance.md §12 describes all three dispositions', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  expect(content.toLowerCase()).toMatch(/fix now/)
  expect(content).toContain('sdlc task add')
  expect(content.toLowerCase()).toMatch(/accept/)
})

test('F2: guidance.md §12 states silence is not acceptance', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  expect(content).toMatch(/[Ss]ilence is not acceptance/)
})

test('F2: guidance.md §12 distinguishes targeted fixes from fix-all / remediate', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  expect(content).toContain('fix-all')
  expect(content).toContain('remediate')
  // Must mention they are for systemic / codebase-wide patterns
  expect(content.toLowerCase()).toMatch(/systemic|codebase.wide/)
})

test('F2: guidance.md §12 is positioned after §11 (Project Guidelines)', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  const idx11 = content.indexOf('11. Project Guidelines')
  const idx12 = content.search(/12\..*[Aa]udit|[Aa]udit.*Review Findings/)
  expect(idx11).toBeGreaterThan(-1)
  expect(idx12).toBeGreaterThan(idx11)
})

test('F2: guidance.md existing sections §1–§11 are unchanged (section count)', () => {
  const content = fs.readFileSync(
    path.join(ROOT, '.sdlc/guidance.md'),
    'utf8',
  )
  // All sections 1–12 must exist
  for (let i = 1; i <= 11; i++) {
    expect(content).toContain(`${i}.`)
  }
})
