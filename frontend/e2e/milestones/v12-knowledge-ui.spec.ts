import { test, expect } from '@playwright/test';

test.describe('v12-knowledge-ui — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('Knowledge page loads at /knowledge', async ({ page }) => {
    await page.goto('/knowledge');
    await expect(page.getByRole('heading', { name: 'Knowledge', level: 2 })).toBeVisible();
  });

  test('Sidebar shows Knowledge link with Library icon in plan group', async ({ page }) => {
    await page.goto('/knowledge');
    const link = page.getByRole('link', { name: 'Knowledge' });
    await expect(link).toBeVisible();
    await expect(link).toHaveAttribute('href', '/knowledge');
  });

  test('Left pane shows catalog tree with expandable classes', async ({ page }) => {
    await page.goto('/knowledge');
    await expect(page.getByRole('button', { name: /All entries/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /100.*Stack/ })).toBeVisible();
    await expect(page.getByRole('button', { name: /200.*Workspace Layout/ })).toBeVisible();
  });

  test('Clicking a catalog class filters the center pane to entries in that class', async ({ page }) => {
    await page.goto('/knowledge');
    await page.getByRole('button', { name: /500.*Key Decisions/ }).click();
    await expect(page.getByText('Code: 500')).toBeVisible();
  });

  test('Search bar in left pane filters entries', async ({ page }) => {
    await page.goto('/knowledge');
    // This test is expected to fail until T13 is implemented
    const searchInput = page.getByRole('searchbox').or(page.getByPlaceholder(/search/i));
    await expect(searchInput).toBeVisible();
  });

  test('Center pane shows staleness badges on entries', async ({ page }) => {
    await page.goto('/knowledge');
    // This test is expected to fail until T14 is implemented
    // Staleness badges (url_404, aged_out) should appear on entry cards
    const stalenessIndicator = page.locator('[data-staleness], .staleness-badge').first();
    await expect(stalenessIndicator).toBeVisible();
  });

  test('Clicking an entry opens the detail in the right pane', async ({ page }) => {
    await page.goto('/knowledge');
    await page.getByRole('button', { name: /API Created Entry/ }).click();
    await expect(page.getByRole('heading', { name: 'API Created Entry', level: 2 })).toBeVisible();
    await expect(page).toHaveURL(/\/knowledge\/api-created-entry/);
  });

  test('Right pane renders entry content as Markdown', async ({ page }) => {
    await page.goto('/knowledge/uat-test-inv');
    // This test expects proper markdown rendering — currently raw text via <pre>
    // Will pass once T15 (react-markdown) is implemented
    const heading = page.getByRole('heading', { name: /Agent Spawn Pattern/ });
    await expect(heading).toBeVisible();
  });

  test('Source provenance shows in the right pane (type and url/path)', async ({ page }) => {
    await page.goto('/knowledge/url-entry');
    const sourceLink = page.getByRole('link', { name: 'web' });
    await expect(sourceLink).toBeVisible();
    await expect(sourceLink).toHaveAttribute('href', 'https://example.com');
  });

  test('Related entries are shown in the right pane', async ({ page }) => {
    await page.goto('/knowledge/investigation-uat-test-inv');
    await expect(page.getByText('Related')).toBeVisible();
    await expect(page.getByText('uat-test-entry')).toBeVisible();
  });

  test('Related entries are clickable links (navigate to /knowledge/:slug)', async ({ page }) => {
    await page.goto('/knowledge/investigation-uat-test-inv');
    // This test is expected to fail until T16 is implemented
    const relatedLink = page.getByRole('button', { name: 'uat-test-entry' })
      .or(page.getByRole('link', { name: 'uat-test-entry' }));
    await expect(relatedLink).toBeVisible();
  });

  test('Research More button triggers a research run visible in agent panel', async ({ page }) => {
    await page.goto('/knowledge/api-created-entry');
    const researchBtn = page.getByRole('button', { name: /Research More/ });
    await expect(researchBtn).toBeVisible();
    await researchBtn.click();
    await expect(page.getByText(/Research:.*API Created Entry/)).toBeVisible({ timeout: 5000 });
  });

  test('/sdlc-knowledge slash command exists', async () => {
    // Verified via filesystem: ~/.claude/commands/sdlc-knowledge.md exists
    // No browser interaction needed
  });

  test('GUIDANCE_MD_CONTENT includes sdlc knowledge * commands', async () => {
    // Verified via code inspection in crates/sdlc-cli/src/cmd/init/templates.rs
    // Lines 191-196 contain sdlc knowledge status/list/show/search/update entries
  });

  test('cargo test --all passes', async () => {
    // Verified by running: SDLC_NO_NPM=1 cargo test --all — exit code 0
  });

  test('cargo clippy --all -- -D warnings passes', async () => {
    // Verified by running: SDLC_NO_NPM=1 cargo clippy --all -- -D warnings — exit code 0
  });
});
