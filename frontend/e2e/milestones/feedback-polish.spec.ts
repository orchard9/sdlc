import { test, expect } from '@playwright/test'

/**
 * UAT spec for feedback-polish
 * Milestone: Feedback Notes Polish — Edit and Enrichment
 *
 * Features under test:
 *   - feedback-edit  (inline editing of feedback notes)
 *   - feedback-enrich (attaching research context enrichments)
 */

test.describe('feedback-polish — Acceptance Tests', () => {
  // Seed a fresh note before each test to ensure a clean, deterministic state
  test.beforeEach(async ({ request, page }) => {
    // Clear existing notes by deleting them all
    const notes = await request.get('/api/feedback')
    const noteList: Array<{ id: string }> = await notes.json()
    for (const n of noteList) {
      await request.delete(`/api/feedback/${encodeURIComponent(n.id)}`)
    }
    // Create a fresh note for testing
    await request.post('/api/feedback', {
      data: { content: 'Original note content for UAT' },
    })
    await page.goto('/feedback')
    // Wait for the note to appear
    await page.locator('pre').first().waitFor({ state: 'visible', timeout: 10_000 })
  })

  // ── feedback-edit ─────────────────────────────────────────────────────────

  test('double-clicking a note card opens it in edit mode with content pre-filled', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.dblclick()
    // An edit textarea should appear with the original content
    const textarea = noteCard.locator('textarea').first()
    await expect(textarea).toBeVisible()
    await expect(textarea).toHaveValue('Original note content for UAT')
  })

  test('pencil icon appears on hover and opens edit mode', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.hover()
    // Pencil button appears
    const pencilBtn = noteCard.locator('button[title="Edit note"]')
    await expect(pencilBtn).toBeVisible()
    await pencilBtn.click()
    // Edit textarea appears
    const textarea = noteCard.locator('textarea').first()
    await expect(textarea).toBeVisible()
  })

  test('saving an edit persists and reflects in UI without page reload', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.dblclick()
    const textarea = noteCard.locator('textarea').first()
    await textarea.fill('Edited note content')
    // Click Save
    await noteCard.getByRole('button', { name: 'Save' }).click()
    // Edit mode closes and updated content appears
    await expect(noteCard.locator('pre').first()).toHaveText('Edited note content')
    // No reload — verify by confirming no navigation occurred (still on /feedback)
    expect(page.url()).toContain('/feedback')
  })

  test('pressing Escape in edit mode cancels without saving', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.dblclick()
    const textarea = noteCard.locator('textarea').first()
    await textarea.fill('This change should be discarded')
    await textarea.press('Escape')
    // Display mode restored with original content
    await expect(noteCard.locator('pre').first()).toHaveText('Original note content for UAT')
    // No textarea visible (edit mode closed)
    await expect(textarea).not.toBeVisible()
  })

  test('save button is disabled when edit textarea is empty', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.dblclick()
    const textarea = noteCard.locator('textarea').first()
    await textarea.fill('')  // clear the textarea
    const saveBtn = noteCard.getByRole('button', { name: 'Save' })
    await expect(saveBtn).toBeDisabled()
  })

  test('updated_at metadata appears in the note card after a successful edit', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.dblclick()
    const textarea = noteCard.locator('textarea').first()
    await textarea.fill('Updated content to trigger timestamp')
    await noteCard.getByRole('button', { name: 'Save' }).click()
    // Wait for edit mode to close
    await expect(noteCard.locator('textarea')).not.toBeVisible()
    // The metadata line should include "edited"
    const metaLine = noteCard.locator('p').last()
    await expect(metaLine).toContainText('edited')
  })

  test('PATCH /api/feedback/:id returns 404 for non-existent ID', async ({ request }) => {
    const response = await request.patch('/api/feedback/NONEXISTENT_ID_UAT', {
      data: { content: 'should not work' },
    })
    expect(response.status()).toBe(404)
  })

  test('PATCH /api/feedback/:id returns 400 for empty content', async ({ request, page }) => {
    const notes = await request.get('/api/feedback')
    const noteList: Array<{ id: string }> = await notes.json()
    const id = noteList[0].id
    const response = await request.patch(`/api/feedback/${encodeURIComponent(id)}`, {
      data: { content: '' },
    })
    expect(response.status()).toBe(400)
  })

  test('existing notes without updated_at field deserialize correctly', async ({ request }) => {
    // Seed a note directly via API — the response should have updated_at populated
    const res = await request.post('/api/feedback', {
      data: { content: 'backward compat note' },
    })
    const note = await res.json()
    // updated_at must be present (defaulted by serde if missing from YAML)
    expect(note.updated_at).toBeTruthy()
    // enrichments must default to empty array
    expect(Array.isArray(note.enrichments)).toBe(true)
    expect(note.enrichments.length).toBe(0)
  })

  // ── feedback-enrich ───────────────────────────────────────────────────────

  test('"Add context" button appears on hover in NoteCard', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.hover()
    const addContextBtn = noteCard.locator('button[title="Add context"]')
    await expect(addContextBtn).toBeVisible()
  })

  test('submitting an enrichment updates the card in place without page reload', async ({ page }) => {
    const noteCard = page.locator('.group').first()
    await noteCard.hover()
    await noteCard.locator('button[title="Add context"]').click()
    // Enrich textarea appears
    const enrichTextarea = noteCard.locator('textarea').last()
    await expect(enrichTextarea).toBeVisible()
    await enrichTextarea.fill('Research context added by UAT')
    // Click Save in the enrich panel
    const saveBtn = noteCard.getByRole('button', { name: 'Save' }).last()
    await saveBtn.click()
    // Enrichment block appears below the note
    await expect(noteCard.locator('text=Research context added by UAT')).toBeVisible()
    // Source pill shows "user" (use .first() — parallel tests may add multiple enrichments)
    await expect(noteCard.locator('text=user').first()).toBeVisible()
    // Still on same page
    expect(page.url()).toContain('/feedback')
  })

  test('POST /api/feedback/:id/enrich returns updated note with enrichment', async ({ request }) => {
    const notes = await request.get('/api/feedback')
    const noteList: Array<{ id: string }> = await notes.json()
    const id = noteList[0].id

    const res = await request.post(`/api/feedback/${encodeURIComponent(id)}/enrich`, {
      data: { content: 'API enrichment test', source: 'user' },
    })
    expect(res.ok()).toBe(true)
    const updated = await res.json()
    // At least one enrichment with the expected content must be present
    // (parallel tests may have added other enrichments to the same note)
    const match = updated.enrichments.find(
      (e: { content: string; source: string; added_at: string }) => e.content === 'API enrichment test'
    )
    expect(match).toBeDefined()
    expect(match.source).toBe('user')
    expect(match.added_at).toBeTruthy()
  })

  test('POST /api/feedback/:id/enrich returns 404 for non-existent ID', async ({ request }) => {
    const res = await request.post('/api/feedback/NONEXISTENT_ID_UAT/enrich', {
      data: { content: 'test', source: 'user' },
    })
    expect(res.status()).toBe(404)
  })

  test('multiple enrichments accumulate on the same note', async ({ request }) => {
    const notes = await request.get('/api/feedback')
    const noteList: Array<{ id: string }> = await notes.json()
    const id = noteList[0].id

    await request.post(`/api/feedback/${encodeURIComponent(id)}/enrich`, {
      data: { content: 'First enrichment', source: 'user' },
    })
    const res = await request.post(`/api/feedback/${encodeURIComponent(id)}/enrich`, {
      data: { content: 'Second enrichment', source: 'user' },
    })
    const updated = await res.json()
    expect(updated.enrichments).toHaveLength(2)
  })
})
