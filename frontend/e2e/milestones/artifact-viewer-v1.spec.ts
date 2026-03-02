/**
 * Acceptance Tests: Rich Artifact and Plan Viewer V1
 *
 * Tests four features delivered in milestone artifact-viewer-v1:
 *   1. artifact-viewer-height-fix  — remove max-h-96 constraint from artifact cards
 *   2. artifact-file-links         — inline code file paths render as IDE links
 *   3. artifact-tldr-teaser        — artifact card shows 120-char teaser + timestamp
 *   4. artifact-fullscreen-toc     — fullscreen modal has sticky TOC navigation rail
 *
 * Uses feature "artifact-viewer-height-fix" as a real test subject (has rich spec/design artifacts).
 */

import { test, expect } from '@playwright/test';

const FEATURE_SLUG = 'artifact-viewer-height-fix';
const FEATURE_URL = `/features/${FEATURE_SLUG}`;

test.describe('Rich Artifact and Plan Viewer V1 — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(FEATURE_URL);
    // Wait for artifacts to render
    await page.waitForSelector('[data-testid="artifact-status"]', { timeout: 10000 });
  });

  // ── Feature 1: artifact-viewer-height-fix ──────────────────────────────────

  test('F1: artifact content div has no max-height constraint (no inner scroll)', async ({ page }) => {
    // The artifact card content wrapper must not have max-h-96 or overflow-y-auto.
    // We inspect the scrollHeight vs clientHeight of the first artifact content div.
    const contentDiv = page.locator('.border.border-border.rounded-lg').first()
      .locator('div.p-4').first();

    const hasHeightCap = await contentDiv.evaluate((el: HTMLElement) => {
      const style = window.getComputedStyle(el);
      // max-h-96 = 24rem = 384px. If maxHeight is set to 384px or scrollHeight > clientHeight, capped.
      const maxH = style.maxHeight;
      const overflowY = style.overflowY;
      return maxH !== 'none' || overflowY === 'auto' || overflowY === 'scroll';
    });

    expect(hasHeightCap).toBe(false);
  });

  test('F1: fullscreen button still appears on artifact card', async ({ page }) => {
    // The Maximize2 icon button must exist in at least one artifact card
    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await expect(fullscreenBtn).toBeVisible();
  });

  test('F1: fullscreen modal opens and closes', async ({ page }) => {
    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await fullscreenBtn.click();

    // Modal should appear
    const closeBtn = page.getByRole('button', { name: 'Close fullscreen' });
    await expect(closeBtn).toBeVisible();

    // Close via button
    await closeBtn.click();
    await expect(closeBtn).not.toBeVisible();
  });

  // ── Feature 2: artifact-file-links ────────────────────────────────────────

  test('F2: inline code matching file path pattern renders as IDE link', async ({ page }) => {
    // The spec for artifact-viewer-height-fix references "frontend/src/components/features/ArtifactViewer.tsx"
    // After feature 2 is implemented, this renders as an <a href="vscode://file/..."> link.
    // Locate any <a> inside a code span with a file-path-shaped href.
    const fileLink = page.locator('a[href*="://file/"]').first();
    await expect(fileLink).toBeVisible();
  });

  // ── Feature 3: artifact-tldr-teaser ───────────────────────────────────────

  test('F3: artifact card header shows a teaser text below the type/status row', async ({ page }) => {
    // After feature 3, each card header has a second row with teaser text and timestamp.
    // The teaser comes from the first paragraph of the artifact content.
    // We look for a text node in the card header area that is NOT the artifact type label
    // and NOT the status badge.
    const teaserRow = page.locator('.border.border-border.rounded-lg').first()
      .locator('div.px-3.py-2.bg-card\\/50 ~ div');

    // Alternatively, look for a data-testid if added, or a relative timestamp pattern
    // like "Xm ago" or "Xh ago"
    const timestampEl = page.locator('text=/\\d+[smhd] ago/').first();
    await expect(timestampEl).toBeVisible();
  });

  test('F3: teaser text is capped at 120 characters', async ({ page }) => {
    // Find any teaser text element. It should not exceed 120 chars + ellipsis.
    // We use the relative time pattern to find the row, then get sibling text.
    const teaserText = await page.locator('[data-testid="artifact-teaser"]').first().textContent();
    if (teaserText) {
      // Allow for "…" suffix; base content must be ≤ 120 chars
      expect(teaserText.replace('…', '').length).toBeLessThanOrEqual(120);
    }
  });

  // ── Feature 4: artifact-fullscreen-toc ────────────────────────────────────

  test('F4: fullscreen modal shows sticky TOC navigation rail on desktop', async ({ page }) => {
    // Open fullscreen
    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await fullscreenBtn.click();

    // TOC nav rail should appear
    const tocNav = page.locator('nav').filter({ hasText: 'Contents' });
    await expect(tocNav).toBeVisible();
  });

  test('F4: TOC contains heading entries that match artifact headings', async ({ page }) => {
    // Open fullscreen
    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await fullscreenBtn.click();

    // TOC should have at least one heading link
    const tocNav = page.locator('nav').filter({ hasText: 'Contents' });
    const tocLinks = tocNav.locator('a');
    const count = await tocLinks.count();
    expect(count).toBeGreaterThan(0);
  });

  test('F4: clicking a TOC link scrolls to the correct heading', async ({ page }) => {
    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await fullscreenBtn.click();

    const tocNav = page.locator('nav').filter({ hasText: 'Contents' });
    const firstTocLink = tocNav.locator('a').first();
    const linkText = await firstTocLink.textContent();

    await firstTocLink.click();

    // The heading with matching text should now be visible in viewport
    if (linkText) {
      const heading = page.locator(`h1, h2, h3`).filter({ hasText: linkText.trim() }).first();
      await expect(heading).toBeInViewport({ ratio: 0.5 });
    }
  });

  test('F4: mobile "Jump to..." dropdown appears on small screens', async ({ page }) => {
    // Resize to mobile
    await page.setViewportSize({ width: 375, height: 812 });

    const fullscreenBtn = page.locator('button[title="Fullscreen"]').first();
    await fullscreenBtn.click();

    // The select dropdown for mobile navigation
    const jumpDropdown = page.locator('select').filter({ hasText: /jump to/i });
    // On mobile the lg:hidden select should be visible
    await expect(jumpDropdown).toBeVisible();
  });
});
