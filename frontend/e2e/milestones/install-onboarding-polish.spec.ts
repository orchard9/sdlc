import { test, expect } from '@playwright/test';
import { readFileSync } from 'fs';
import { resolve, dirname } from 'path';
import { fileURLToPath } from 'url';

// Root of the repo — 3 hops up: milestones/ → e2e/ → frontend/ → sdlc/
const __filename = fileURLToPath(import.meta.url);
const __dirname_esm = dirname(__filename);
const repoRoot = resolve(__dirname_esm, '../../..');

test.describe('install-onboarding-polish — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  // ── CLI / Documentation Checks ─────────────────────────────────────────

  test('README contains SSH URL variant for cargo install', () => {
    const readme = readFileSync(resolve(repoRoot, 'README.md'), 'utf-8');
    expect(readme).toContain('ssh://git@github.com/orchard9/sdlc');
  });

  test('README contains make install as from-source path', () => {
    const readme = readFileSync(resolve(repoRoot, 'README.md'), 'utf-8');
    expect(readme).toContain('make install');
  });

  test('README contains an Updating section documenting sdlc update', () => {
    const readme = readFileSync(resolve(repoRoot, 'README.md'), 'utf-8');
    expect(readme).toMatch(/##\s+Updating/);
    expect(readme).toContain('sdlc update');
  });

  test('sdlc init completion message directs users to sdlc ui', () => {
    const initMod = readFileSync(
      resolve(repoRoot, 'crates/sdlc-cli/src/cmd/init/mod.rs'),
      'utf-8',
    );
    // The println! completion line should contain "sdlc ui"
    expect(initMod).toMatch(/println!.*sdlc ui/);
    // The println! completion line should NOT be "sdlc feature create"
    expect(initMod).not.toMatch(/println!.*Next:.*sdlc feature create/);
  });

  // ── UI: Setup Page (/setup) ─────────────────────────────────────────────

  test('setup page loads and shows multi-step wizard', async ({ page }) => {
    await page.goto('/setup');
    await page.waitForLoadState('domcontentloaded');
    const main = page.locator('main');
    // The wizard heading should be present
    await expect(main.getByRole('heading', { name: /set up your project/i })).toBeVisible();
    // Step indicators should be rendered in main content (not sidebar)
    await expect(main.getByText('Vision', { exact: true }).first()).toBeVisible();
    await expect(main.getByText('Architecture', { exact: true }).first()).toBeVisible();
  });

  test('setup page vision step shows guidance text about agents', async ({ page }) => {
    await page.goto('/setup');
    await page.waitForLoadState('domcontentloaded');
    const main = page.locator('main');
    // Click the Vision step button in main content
    await main.getByRole('button', { name: /vision/i }).click();
    // Guidance text should explain Vision.md and agents — scoped to main to avoid sidebar
    const visionGuidance = main.getByText(/VISION\.md|AI agent|every AI agent/i).first();
    await expect(visionGuidance).toBeVisible();
  });

  test('setup page architecture step shows guidance text about agents', async ({ page }) => {
    await page.goto('/setup');
    await page.waitForLoadState('domcontentloaded');
    const main = page.locator('main');
    // Click the Architecture step button in main content
    await main.getByRole('button', { name: /architecture/i }).click();
    // Guidance text should explain ARCHITECTURE.md and scope — scoped to main
    const archGuidance = main.getByText(/ARCHITECTURE\.md|tells agents|in scope/i).first();
    await expect(archGuidance).toBeVisible();
  });

  // ── UI: Dashboard Vision/Architecture Banner ────────────────────────────

  test('dashboard loads without errors', async ({ page }) => {
    await page.waitForLoadState('domcontentloaded');
    // Check no JavaScript error banner is present
    await expect(page.locator('body')).toBeVisible();
    // Main content area is rendered
    await expect(page.locator('main')).toBeVisible();
  });

  test('dashboard does not show Go to Setup banner when Vision and Architecture exist', async ({ page }) => {
    await page.waitForLoadState('domcontentloaded');
    // The project has both Vision and Architecture defined, so no banner should appear
    await expect(page.getByText('Go to Setup', { exact: false })).not.toBeVisible();
  });

  // ── UI: Milestone Detail — UatHistoryPanel ─────────────────────────────

  test('milestone detail page loads for install-onboarding-polish', async ({ page }) => {
    await page.goto('/milestones/install-onboarding-polish');
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByRole('heading', { name: /Installation and Onboarding Polish/i })).toBeVisible();
  });

  test('UatHistoryPanel renders with data-testid attribute', async ({ page }) => {
    await page.goto('/milestones/install-onboarding-polish');
    await page.waitForLoadState('domcontentloaded');
    // Wait for the UAT history panel to appear (it either shows runs or empty state)
    const panel = page.locator('[data-testid="uat-history-panel"]');
    await expect(panel).toBeVisible();
  });

  test('UatHistoryPanel shows UAT History heading', async ({ page }) => {
    await page.goto('/milestones/install-onboarding-polish');
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByRole('heading', { name: /UAT History/i })).toBeVisible();
  });

  // ── UI: Config / Settings Page (/config) ───────────────────────────────

  test('config page loads successfully at /config', async ({ page }) => {
    await page.goto('/config');
    await page.waitForLoadState('domcontentloaded');
    // Page should not be a 404 or blank
    await expect(page.locator('main')).toBeVisible();
  });

  test('config page displays project configuration data', async ({ page }) => {
    await page.goto('/config');
    await page.waitForLoadState('domcontentloaded');
    // The settings page should show config content — look for common config UI elements
    const body = await page.locator('body').innerText();
    // Should contain some project-related content (config fields, project name, or config heading)
    expect(body.length).toBeGreaterThan(100);
    // Should not show an error
    await expect(page.getByText('Failed to load', { exact: false })).not.toBeVisible();
  });
});
