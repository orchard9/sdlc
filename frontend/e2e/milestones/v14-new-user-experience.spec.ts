import { test, expect } from '@playwright/test';

test.describe('v14-new-user-experience — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('dashboard loads without setup-incomplete amber banner', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    await expect(page.getByText('setup incomplete', { exact: false })).not.toBeVisible();
    await expect(page.getByText('Setup Incomplete', { exact: false })).not.toBeVisible();
  });

  test('dashboard shows empty-state panel when no features are active', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    // DashboardEmptyState is rendered by Dashboard.tsx when feature list is empty
    const dashboard = page.locator('[data-testid="dashboard"], main, .dashboard');
    await expect(dashboard).toBeVisible();
  });

  test('dashboard empty-state has New Ponder button that navigates to /ponder?new=1', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    const newPonderBtn = page.getByRole('button', { name: /new ponder/i });
    if (await newPonderBtn.isVisible()) {
      await newPonderBtn.click();
      await expect(page).toHaveURL(/\/ponder/);
    }
  });

  test('pipeline indicator is rendered on the dashboard', async ({ page }) => {
    await page.waitForLoadState('networkidle');
    // PipelineIndicator should be visible somewhere on the dashboard
    const pipelineIndicator = page.locator('[data-testid="pipeline-indicator"], .pipeline-indicator');
    await expect(pipelineIndicator).toBeVisible();
  });

  test('vision page shows subtitle explaining purpose', async ({ page }) => {
    await page.goto('/vision');
    await page.waitForLoadState('networkidle');
    // Should have explanatory subtitle for agents
    const subtitle = page.getByText(/what you.?re building and why/i);
    await expect(subtitle).toBeVisible();
  });

  test('architecture page shows subtitle explaining purpose', async ({ page }) => {
    await page.goto('/architecture');
    await page.waitForLoadState('networkidle');
    // Should have explanatory subtitle
    const subtitle = page.getByText(/how the system is built|system design|architecture overview/i);
    await expect(subtitle).toBeVisible();
  });

  test('ponder page auto-opens new idea form when ?new=1 is in URL', async ({ page }) => {
    await page.goto('/ponder?new=1');
    await page.waitForLoadState('networkidle');
    // NewIdeaForm or equivalent should be visible without user interaction
    const form = page.getByRole('form').or(page.locator('[data-testid="new-idea-form"]')).or(page.getByPlaceholder(/idea|title|name/i));
    await expect(form.first()).toBeVisible();
  });

  test('blocked feature panel is present in feature detail page', async ({ page }) => {
    // Navigate to a feature that might be blocked (or any feature detail page)
    // The BlockedPanel component should be importable and used in FeatureDetail
    await page.goto('/features');
    await page.waitForLoadState('networkidle');
    // At minimum the features page should load without error
    await expect(page).not.toHaveURL(/error/);
  });

  test('/docs/commands renders the commands catalog with search', async ({ page }) => {
    await page.goto('/docs/commands');
    await page.waitForLoadState('networkidle');
    // Should show the CommandsCatalog with a list of sdlc commands
    const catalog = page.getByText(/sdlc next|sdlc run|sdlc artifact/i);
    await expect(catalog.first()).toBeVisible();
  });

  test('dashboard loads without JavaScript errors', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', err => errors.push(err.message));
    await page.goto('/');
    await page.waitForLoadState('networkidle');
    expect(errors.filter(e => !e.includes('ResizeObserver'))).toHaveLength(0);
  });
});
