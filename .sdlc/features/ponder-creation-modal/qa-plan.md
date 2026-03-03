# QA Plan: ponder-creation-modal

## Approach

Manual browser testing via Playwright MCP against the running dev server. No unit tests required — this is pure UI behavior on top of existing, already-tested API endpoints.

---

## Test Cases

### TC-1: Modal opens from sidebar "+" button

1. Navigate to `/ponder`
2. Click the `+` (New Idea) button in the sidebar header
3. **Expected**: A centered modal dialog opens with Title, Slug, Description, and References fields visible
4. **Expected**: The inline sidebar form area is gone — no form appears in the sidebar itself

### TC-2: Modal opens from empty-state "New idea" button

1. If no ponder entries exist (or filter to empty): click "New idea" in the right-pane empty state
2. **Expected**: Same modal opens

### TC-3: Title input auto-derives slug

1. Open the modal
2. Type "My Cool Idea" in the Title field
3. **Expected**: Slug field shows `my-cool-idea`

### TC-4: Manual slug override

1. Open the modal, type a title
2. Manually edit the Slug field to a custom value
3. Continue typing in Title
4. **Expected**: Slug no longer changes — manual edit broke the auto-derive link

### TC-5: Create with title only (no description, no refs)

1. Open modal, type title "Test Minimal"
2. Leave Description and References empty
3. Click "Create Idea"
4. **Expected**: Entry created, navigated to `/ponder/test-minimal`
5. **Expected**: No `references.md` artifact in the ponder workspace panel

### TC-6: Create with description

1. Open modal, type title and a description
2. Submit
3. **Expected**: `brief.md` appears in the ponder workspace panel with the description content

### TC-7: Create with URL references

1. Open modal, type title
2. Add a URL in the References field: `https://example.com`
3. Click "+ Add reference", add a second URL: `https://github.com`
4. Submit
5. **Expected**: Navigated to ponder entry
6. **Expected**: `references.md` appears in the workspace panel
7. **Expected**: Contents of `references.md` is:
```
# References

- https://example.com
- https://github.com
```

### TC-8: Empty reference rows are ignored

1. Open modal, type title
2. The default empty reference row is left empty
3. Submit
4. **Expected**: No `references.md` artifact created

### TC-9: Remove reference row

1. Open modal
2. Click "+ Add reference" to get two rows
3. Type a URL in the first row
4. Click the remove (X) button on the second (empty) row
5. **Expected**: Row is removed; first row remains with the typed URL

### TC-10: Escape closes modal

1. Open modal
2. Press Escape
3. **Expected**: Modal closes, no entry created

### TC-11: Backdrop click closes modal

1. Open modal
2. Click outside the card (on the backdrop)
3. **Expected**: Modal closes

### TC-12: Submit with empty title is blocked

1. Open modal
2. Leave title empty
3. Click "Create Idea"
4. **Expected**: Button is disabled or no submission occurs

### TC-13: Ponder chat auto-starts

1. Create an entry via the modal
2. Navigate to the ponder entry page
3. **Expected**: A ponder session is active (running indicator or session content visible)

### TC-14: Error handling on slug conflict

1. Create a ponder entry with slug `conflict-test`
2. Open modal again, type a title that derives to `conflict-test`
3. Submit
4. **Expected**: Error message is shown below the form

---

## Pass Criteria

All 14 test cases pass. TC-13 may take a few seconds; wait up to 10 seconds for the session to appear.
