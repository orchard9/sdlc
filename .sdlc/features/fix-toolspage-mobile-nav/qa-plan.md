# QA Plan: ToolsPage mobile back navigation fix

## TC-1 — Mobile: back button is visible and functional

**Viewport:** 375×812 (mobile)

1. Navigate to `/tools`.
2. Confirm the tool list is visible.
3. Tap any tool card.
4. Confirm: the detail pane is shown and the list pane is hidden.
5. Confirm: an `←` (ArrowLeft) back button is visible in the top-left of the detail header.
6. Tap the back button.
7. Confirm: the list pane is restored; the detail pane is hidden.

**Pass criteria:** steps 5 and 7 both succeed.

## TC-2 — Desktop: back button is not rendered

**Viewport:** 1280×800 (desktop)

1. Navigate to `/tools`.
2. Select any tool.
3. Inspect the detail header.
4. Confirm: no back button is visible (element is `display:none` via `md:hidden`).

**Pass criteria:** step 4 succeeds — no button visible.

## TC-3 — No TypeScript build error

Run `cd frontend && npm run build`.

**Pass criteria:** exits 0 with no TS errors.

## TC-4 — Desktop two-pane layout unchanged

**Viewport:** 1280×800

1. Navigate to `/tools`.
2. Select a tool.
3. Confirm: tool list is still visible in left pane alongside the detail pane.

**Pass criteria:** both panes visible simultaneously on desktop.
</content>
