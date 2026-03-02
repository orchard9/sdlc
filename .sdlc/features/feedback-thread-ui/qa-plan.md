# QA Plan: FeedbackThread UI

## Approach

Manual browser verification + TypeScript build check. No new Rust changes so no cargo tests needed for this feature. The frontend is verified by:

1. `npm run build` — TypeScript must compile cleanly with zero errors
2. Browser walkthrough against a running `sdlc serve` instance with the `feedback-thread-core` REST API available (or mocked via stub data)

---

## Test Cases

### TC-1: Sidebar navigation

**Steps:**
1. Open the sdlc UI
2. Look at the `plan` group in the sidebar

**Expected:** "Threads" item appears with an icon; "Feedback" (MessageSquarePlus) is replaced or no longer visible in the plan group. "Threads" is active (highlighted) when the current path is `/threads` or `/threads/*`.

---

### TC-2: Threads list — empty state

**Steps:**
1. Navigate to `/threads` when no threads exist

**Expected:** Left pane shows "Threads" heading and "New thread" button. Right pane shows a friendly empty-state message (e.g., "Select a thread or create one"). No errors in console.

---

### TC-3: Threads list — populated

**Steps:**
1. Create several threads via the API or use a seeded dataset
2. Navigate to `/threads`

**Expected:** Left pane lists threads with title, status badge (correct colour: open=green, synthesized=blue, promoted=grey), comment count, and author. Threads are scrollable if they overflow.

---

### TC-4: Select a thread

**Steps:**
1. Click a thread in the list

**Expected:**
- URL changes to `/threads/:slug`
- Thread item shows selected (highlighted background)
- Right pane loads the thread: title, status badge, meta (author · date · comment count)
- Core element card visible (or "No core element yet" placeholder if `body` is null)
- Comment list renders all comments

---

### TC-5: Agent vs human comment avatar

**Steps:**
1. Open a thread that has both human and agent comments (e.g., author `jordan` and `agent:advisor`)

**Expected:** Human comments have a green avatar with the author's initial. Agent comments have a blue-purple avatar.

---

### TC-6: Incorporated comments render correctly

**Steps:**
1. Open a thread that has comments where `incorporated: true`

**Expected:** Incorporated comments are visually dimmed (reduced opacity), have a dashed border, and show an "absorbed" badge. Unincorporated comments appear at full opacity.

---

### TC-7: Send a comment

**Steps:**
1. Open a thread
2. Type a comment in the compose area
3. Click Send (or press Cmd+Enter)

**Expected:**
- `POST /api/threads/:slug/comments` is called
- New comment immediately appears in the list (optimistic)
- Compose textarea is cleared
- No page reload

---

### TC-8: Compose keyboard shortcut

**Steps:**
1. Type text in the compose textarea
2. Press `Cmd+Enter` (Mac) or `Ctrl+Enter` (non-Mac)

**Expected:** Comment is submitted (same behaviour as clicking Send).

---

### TC-9: Create new thread — title only

**Steps:**
1. Click "New thread"
2. Fill in title, leave core element blank
3. Click "Create thread"

**Expected:**
- Modal closes
- Browser navigates to `/threads/:newSlug`
- New thread appears in the list
- Thread detail shows title with no core element (placeholder shown)

---

### TC-10: Create new thread — with body

**Steps:**
1. Click "New thread"
2. Fill in title and core element text
3. Click "Create thread"

**Expected:**
- Thread detail shows the core element card with the body content
- Version chip shows `v1`

---

### TC-11: Cancel modal with Escape and Cancel button

**Steps:**
1. Open "New thread" modal
2. Press Escape

**Expected:** Modal closes without creating a thread.

Repeat with the Cancel button.

---

### TC-12: Mobile layout — list → detail navigation

**Steps:**
1. Resize browser to mobile width (< 768px)
2. Navigate to `/threads`

**Expected:** Only the thread list is visible (full width). Clicking a thread navigates to `/threads/:slug` which shows only the detail pane. The mobile header shows a back-chevron button.

---

### TC-13: TypeScript build is clean

**Steps:**
1. Run `cd frontend && npm run build`

**Expected:** Build completes with exit code 0 and no TypeScript errors.

---

## Pass Criteria

All 13 test cases pass. TC-13 (TypeScript build) is a hard gate — the feature does not ship with TypeScript errors.
