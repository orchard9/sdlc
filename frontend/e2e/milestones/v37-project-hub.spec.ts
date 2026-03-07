import { test, expect } from '@playwright/test';

const HUB_PORT = 9998;
const HUB_URL = `http://localhost:${HUB_PORT}`;
const HUB_HEARTBEAT_URL = `${HUB_URL}/api/hub/heartbeat`;

async function sendHeartbeat(request: any, payload: object) {
  const res = await request.post(HUB_HEARTBEAT_URL, { data: payload });
  expect(res.ok()).toBeTruthy();
}

const PAYMENTS_HEARTBEAT = {
  name: 'payments-api',
  url: 'http://localhost:3001',
  active_milestone: 'v12-checkout-flow',
  feature_count: 3,
  agent_running: false,
};

const AUTH_HEARTBEAT = {
  name: 'auth-service',
  url: 'http://localhost:3004',
  active_milestone: 'v8-oauth-flow',
  feature_count: 5,
  agent_running: true,
};

test.describe('v37-project-hub — Acceptance Tests', () => {
  test.beforeAll(async ({ request }) => {
    // Verify hub is reachable
    const res = await request.get(`${HUB_URL}/api/hub/projects`);
    if (!res.ok()) {
      throw new Error(
        `Hub not reachable at ${HUB_URL} — start it with: ./target/debug/sdlc ui start --hub --port ${HUB_PORT} --no-open --no-tunnel`,
      );
    }
  });

  test('Scenario 7: Empty state renders without errors', async ({ page }) => {
    const consoleErrors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') consoleErrors.push(msg.text());
    });

    await page.goto(HUB_URL);
    await page.waitForLoadState('domcontentloaded');
    // Wait for loading spinner to disappear
    await page.waitForSelector('.animate-spin', { state: 'detached', timeout: 5000 }).catch(() => {});

    const body = page.locator('body');
    await expect(body).toBeVisible();
    await expect(page.getByRole('button', { name: 'Add Project' })).toBeVisible();

    const nonFaviconErrors = consoleErrors.filter(e => !e.includes('favicon'));
    expect(nonFaviconErrors).toHaveLength(0);
  });

  test('Scenario 8: Hub API returns valid response when no projects registered', async ({ request }) => {
    const res = await request.get(`${HUB_URL}/api/hub/projects`);
    expect(res.ok()).toBeTruthy();
    const projects = await res.json();
    expect(Array.isArray(projects)).toBeTruthy();
  });

  test('Scenario 1: Projects appear after heartbeat — fleet rows show project data', async ({
    page,
    request,
  }) => {
    await sendHeartbeat(request, PAYMENTS_HEARTBEAT);
    await sendHeartbeat(request, AUTH_HEARTBEAT);

    await page.goto(HUB_URL);
    await page.waitForLoadState('domcontentloaded');

    await expect(page.getByText('Fleet control, not just project inventory.')).toBeVisible();
    await expect(page.getByText('payments-api')).toBeVisible({ timeout: 8000 });
    await expect(page.getByText('auth-service')).toBeVisible({ timeout: 8000 });

    const authRow = page.locator('tr').filter({ hasText: 'auth-service' }).first();
    await authRow.click();
    await expect(page.getByText('http://localhost:3004')).toBeVisible();
  });

  test('Scenario 2: Filter narrows fleet rows in real time', async ({ page, request }) => {
    await sendHeartbeat(request, PAYMENTS_HEARTBEAT);
    await sendHeartbeat(request, AUTH_HEARTBEAT);

    await page.goto(HUB_URL);
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByText('payments-api')).toBeVisible({ timeout: 8000 });

    const filterInput = page.locator('input[placeholder*="Search fleet"]').first();
    await expect(filterInput).toBeVisible();

    // Type to filter
    await filterInput.fill('pay');

    await expect(page.getByText('payments-api')).toBeVisible();
    await expect(page.getByText('auth-service')).not.toBeVisible();

    // Clear filter restores all
    await filterInput.fill('');
    await expect(page.getByText('auth-service')).toBeVisible();
  });

  test('Scenario 3: Row metadata — milestone and agent column are visible', async ({ page, request }) => {
    await sendHeartbeat(request, AUTH_HEARTBEAT); // agent_running: true
    await sendHeartbeat(request, PAYMENTS_HEARTBEAT); // agent_running: false

    await page.goto(HUB_URL);
    await page.waitForLoadState('domcontentloaded');
    await expect(page.getByText('auth-service')).toBeVisible({ timeout: 8000 });

    const authRow = page.locator('tr').filter({ hasText: 'auth-service' }).first();
    const paymentsRow = page.locator('tr').filter({ hasText: 'payments-api' }).first();

    await expect(authRow.getByText('v8-oauth-flow')).toBeVisible();
    await expect(authRow.getByText('Running')).toBeVisible();

    await expect(paymentsRow.getByText('Idle')).toBeVisible();
  });

  test('Scenario 6: Page title is "sdlc hub"', async ({ page, request }) => {
    await sendHeartbeat(request, PAYMENTS_HEARTBEAT);

    await page.goto(HUB_URL);
    await page.waitForLoadState('domcontentloaded');

    const title = await page.title();
    // Known bug: currently shows "sdlc — sdlc" instead of "sdlc hub"
    // A task has been created for page-title-fix feature.
    expect(title).toBe('sdlc hub');
  });
});
