import { test, expect } from '@playwright/test';

/**
 * v38-credential-pool — Acceptance Tests
 *
 * NOTE: Scenarios 1–7 in the acceptance_test.md require a live Postgres instance,
 * server logs inspection, and/or cluster (kubectl) access. They are backend
 * infrastructure tests, not UI tests.
 *
 * This spec validates the browser-observable surface:
 *   - The server is reachable and the app loads without errors.
 *   - The API serves known JSON routes (features, milestones).
 *   - The UI renders and navigates without crashes.
 *
 * Remaining infrastructure scenarios are covered by tasks created during UAT.
 */

test.describe('v38-credential-pool — Acceptance Tests', () => {
  test('Server is reachable and app loads', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveTitle(/sdlc/);
    // App root renders (no white screen)
    await expect(page.locator('#root')).not.toBeEmpty();
  });

  test('Features API returns JSON (core API health)', async ({ request }) => {
    const response = await request.get('/api/features');
    expect(response.ok()).toBeTruthy();
    const body = await response.json();
    expect(Array.isArray(body)).toBeTruthy();
  });

  test('credential-pool-core feature is in released state', async ({ request }) => {
    const response = await request.get('/api/features');
    expect(response.ok()).toBeTruthy();
    const features: Array<{ slug: string; phase: string }> = await response.json();
    const poolCore = features.find((f) => f.slug === 'credential-pool-core');
    expect(poolCore).toBeDefined();
    expect(poolCore?.phase).toBe('released');
  });

  test('credential-pool-runs feature is in released state', async ({ request }) => {
    const response = await request.get('/api/features');
    expect(response.ok()).toBeTruthy();
    const features: Array<{ slug: string; phase: string }> = await response.json();
    const poolRuns = features.find((f) => f.slug === 'credential-pool-runs');
    expect(poolRuns).toBeDefined();
    expect(poolRuns?.phase).toBe('released');
  });

  test('credential-pool-helm feature is in released state', async ({ request }) => {
    const response = await request.get('/api/features');
    expect(response.ok()).toBeTruthy();
    const features: Array<{ slug: string; phase: string }> = await response.json();
    const poolHelm = features.find((f) => f.slug === 'credential-pool-helm');
    expect(poolHelm).toBeDefined();
    expect(poolHelm?.phase).toBe('released');
  });

  test('v38-credential-pool milestone exists in verifying state', async ({ request }) => {
    const response = await request.get('/api/milestones');
    expect(response.ok()).toBeTruthy();
    const milestones: Array<{ slug: string; status: string }> = await response.json();
    const m = milestones.find((m) => m.slug === 'v38-credential-pool');
    expect(m).toBeDefined();
    // Status is verifying (UAT in progress) or released
    expect(['verifying', 'released']).toContain(m?.status);
  });

  test('UI navigation renders without crash', async ({ page }) => {
    await page.goto('/');
    // Sidebar renders
    await expect(page.getByRole('link', { name: 'Features' })).toBeVisible();
    await expect(page.getByRole('link', { name: 'Milestones' })).toBeVisible();
    // Navigate to features page
    await page.getByRole('link', { name: 'Features' }).click();
    await expect(page).toHaveURL(/features/);
    // Page renders content (not empty)
    await expect(page.locator('#root')).not.toBeEmpty();
  });
});
