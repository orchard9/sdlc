import { test, expect } from '@playwright/test'

test.describe('Ask Ponder — Acceptance Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/')
  })

  test('Ask Ponder button is visible in sidebar bottom strip, below Search', async ({ page }) => {
    const buttons = page.locator('aside').getByRole('button')
    const labels = await buttons.allTextContents()
    const normalised = labels.map(l => l.replace(/\s+/g, ' ').trim())
    const searchIdx = normalised.findIndex(l => l.includes('Search'))
    const askIdx = normalised.findIndex(l => l.includes('Ask Ponder'))
    expect(askIdx).toBeGreaterThan(-1)
    expect(askIdx).toBeGreaterThan(searchIdx)
  })

  test('Clicking Ask Ponder opens modal in input state with autofocused textarea', async ({ page }) => {
    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Ask Ponder' })
    await expect(dialog).toBeVisible()
    const textarea = dialog.getByRole('textbox')
    await expect(textarea).toBeFocused()
    await expect(dialog.getByRole('button', { name: 'Ask' })).toBeDisabled()
  })

  test('Ask button enables when question is typed', async ({ page }) => {
    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Ask Ponder' })
    await dialog.getByRole('textbox').fill('How does Fix Right Away diagnose issues?')
    await expect(dialog.getByRole('button', { name: 'Ask' })).toBeEnabled()
  })

  test('Escape closes the modal', async ({ page }) => {
    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    await expect(page.getByRole('dialog', { name: 'Ask Ponder' })).toBeVisible()
    await page.keyboard.press('Escape')
    await expect(page.getByRole('dialog', { name: 'Ask Ponder' })).not.toBeVisible()
  })

  test('⌘/ keyboard shortcut opens the modal', async ({ page }) => {
    await page.keyboard.press('Meta+/')
    await expect(page.getByRole('dialog', { name: 'Ask Ponder' })).toBeVisible()
    // textarea autofocused and empty on shortcut open
    const textarea = page.getByRole('dialog', { name: 'Ask Ponder' }).getByRole('textbox')
    await expect(textarea).toBeFocused()
  })

  test('⌘/ toggles modal closed when already open', async ({ page }) => {
    await page.keyboard.press('Meta+/')
    await expect(page.getByRole('dialog', { name: 'Ask Ponder' })).toBeVisible()
    await page.keyboard.press('Meta+/')
    await expect(page.getByRole('dialog', { name: 'Ask Ponder' })).not.toBeVisible()
  })

  // Steps 6-11 require a live sdlc-server backend (answering/streaming/answered states,
  // "Ask another", "Open as Thread"). These are verified in production via the full stack.
  test('Ask button transitions to answering state when backend is available', async ({ page }) => {
    // This test exercises the full flow and requires sdlc-server running.
    // Skipped in frontend-only test environments; verified manually against production.
    test.skip(!process.env.SDLC_SERVER_URL, 'requires live sdlc-server backend')

    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Ask Ponder' })
    await dialog.getByRole('textbox').fill('How does Fix Right Away diagnose issues?')
    await dialog.getByRole('button', { name: 'Ask' }).click()

    // Should transition to answering state — look for streaming indicator
    await expect(dialog.getByText('Reading codebase')).toBeVisible({ timeout: 10000 })

    // Wait for answered state — "Ask another" button appears
    await expect(dialog.getByRole('button', { name: 'Ask another' })).toBeVisible({ timeout: 60000 })
    await expect(dialog.getByRole('button', { name: 'Open as Thread' })).toBeVisible()
  })

  test('"Ask another" resets modal to input state when backend is available', async ({ page }) => {
    test.skip(!process.env.SDLC_SERVER_URL, 'requires live sdlc-server backend')

    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Ask Ponder' })
    await dialog.getByRole('textbox').fill('How does Fix Right Away diagnose issues?')
    await dialog.getByRole('button', { name: 'Ask' }).click()
    await expect(dialog.getByRole('button', { name: 'Ask another' })).toBeVisible({ timeout: 60000 })
    await dialog.getByRole('button', { name: 'Ask another' }).click()
    // Back to input — textarea visible and Ask button disabled (empty)
    await expect(dialog.getByRole('textbox')).toBeVisible()
    await expect(dialog.getByRole('button', { name: 'Ask' })).toBeDisabled()
  })

  test('"Open as Thread" navigates to /threads/:id when backend is available', async ({ page }) => {
    test.skip(!process.env.SDLC_SERVER_URL, 'requires live sdlc-server backend')

    await page.getByRole('button', { name: /Ask Ponder/ }).click()
    const dialog = page.getByRole('dialog', { name: 'Ask Ponder' })
    await dialog.getByRole('textbox').fill('How does Fix Right Away diagnose issues?')
    await dialog.getByRole('button', { name: 'Ask' }).click()
    await expect(dialog.getByRole('button', { name: 'Open as Thread' })).toBeVisible({ timeout: 60000 })
    await dialog.getByRole('button', { name: 'Open as Thread' }).click()
    await expect(page).toHaveURL(/\/threads\//)
  })
})
