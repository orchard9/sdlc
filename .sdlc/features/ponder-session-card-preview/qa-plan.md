# QA Plan: ponder-session-card-preview

## Feature
Session card preview from Product Summary (API + UI)

## Scope
This QA plan covers the three-layer change that surfaces a short text preview of the latest ponder session on each entry card in the list view:
1. `extract_session_preview` pure function in `workspace.rs`
2. `last_session_preview` field added to `GET /api/roadmap` JSON response
3. Frontend `PonderSummary` type update and `EntryRow` render change

---

## Test Cases

### TC-1 — Unit: `extract_session_preview` — empty input

**What:** Verify the function returns `None` for empty string input.

**Steps:**
1. Call `extract_session_preview("")`
2. Assert result is `None`

**Expected:** `None`

---

### TC-2 — Unit: `extract_session_preview` — frontmatter-only content

**What:** Verify the function returns `None` when content consists only of YAML frontmatter.

**Steps:**
1. Call `extract_session_preview` with content containing only `---\ntitle: foo\nbar: baz\n---\n`
2. Assert result is `None`

**Expected:** `None`

---

### TC-3 — Unit: `extract_session_preview` — heading-only content

**What:** Verify the function skips heading lines (starting with `#`) and returns `None` when only headings are present.

**Steps:**
1. Call `extract_session_preview` with content `# Session Start\n## Notes\n`
2. Assert result is `None`

**Expected:** `None`

---

### TC-4 — Unit: `extract_session_preview` — first narrative line extraction

**What:** Verify the function returns the first non-meta narrative line.

**Steps:**
1. Call `extract_session_preview` with content:
   ```
   ---
   title: test
   ---

   # Heading

   We are exploring whether memory is the right frame.
   ```
2. Assert result is `Some("We are exploring whether memory is the right frame.")`

**Expected:** `Some("We are exploring whether memory is the right frame.")`

---

### TC-5 — Unit: `extract_session_preview` — truncation at 140 chars

**What:** Verify lines longer than 140 characters are truncated with `…`.

**Steps:**
1. Create a string of 160 `a` characters
2. Call `extract_session_preview` with that string as content
3. Assert result is `Some(s)` where `s` has length 141 (140 chars + `…`) and ends with `…`

**Expected:** `Some("aaa...aaa…")` — exactly 141 characters, ending with `…`

---

### TC-6 — Unit: `extract_session_preview` — tool/artifact comment block skipping

**What:** Verify lines starting with `<!--` or `-->` (tool/artifact block markers) are skipped.

**Steps:**
1. Call `extract_session_preview` with content:
   ```
   <!-- tool: search -->
   result block
   <!-- /tool -->
   This is the real content.
   ```
2. Assert result contains "This is the real content."

**Expected:** `Some("This is the real content.")`

---

### TC-7 — Unit: `extract_session_preview` — partner message bold marker stripping

**What:** Verify leading `**...**` bold markers are stripped from the first line.

**Steps:**
1. Call `extract_session_preview` with content `**Partner Name:**\nActual message text`
2. Assert result is `Some("Actual message text")`

**Steps (alternate):** If the bold marker is on the same line as content, e.g. `**Alice:** This is interesting`, the result should strip the bold prefix and return the remainder.

**Expected:** Stripped bold markers, meaningful text returned.

---

### TC-8 — Unit: `extract_session_preview` — `Recruited:` prefix skipping

**What:** Verify lines starting with `Recruited:` are skipped.

**Steps:**
1. Call `extract_session_preview` with content:
   ```
   Recruited: Alice
   Here is the actual session content.
   ```
2. Assert result is `Some("Here is the actual session content.")`

**Expected:** `Some("Here is the actual session content.")`

---

### TC-9 — API: `GET /api/roadmap` — entry with no sessions

**What:** Verify `last_session_preview` is `null` or absent for entries with no sessions.

**Steps:**
1. Create a ponder entry with no session files
2. Call `GET /api/roadmap`
3. Inspect the JSON for that entry

**Expected:** `last_session_preview` is `null` or the key is absent

---

### TC-10 — API: `GET /api/roadmap` — entry with sessions

**What:** Verify `last_session_preview` is populated for entries with at least one session.

**Steps:**
1. Create a ponder entry with at least one session file containing meaningful narrative content
2. Call `GET /api/roadmap`
3. Inspect the JSON for that entry

**Expected:**
- `last_session_preview` is a non-empty string
- Content matches text from the session file (first meaningful narrative line)
- Length is ≤ 141 characters (140 + possible ellipsis)

---

### TC-11 — API: `GET /api/roadmap` — corrupt/missing session file is graceful

**What:** Verify the list endpoint does not fail when session metadata exists but the session file is missing or corrupt.

**Steps:**
1. Create a ponder entry with session metadata referencing a non-existent session file
2. Call `GET /api/roadmap`
3. Verify the response is HTTP 200 with the entry included

**Expected:**
- HTTP 200 returned
- Entry is present in the response
- `last_session_preview` is `null` or absent (not an error)

---

### TC-12 — Frontend: TypeScript compilation

**What:** Verify `PonderSummary` type includes `last_session_preview?: string | null` and TypeScript compiles without errors.

**Steps:**
1. Run `cd frontend && npm run build` (or `npx tsc --noEmit`)
2. Check for TypeScript errors

**Expected:** No TypeScript compilation errors.

---

### TC-13 — Frontend: Card renders preview when present

**What:** Verify `EntryRow` renders the preview text below the session/team metadata row when `last_session_preview` is set.

**Steps:**
1. Load the ponder page in a browser
2. Identify a ponder entry that has sessions with meaningful content
3. Inspect the entry card

**Expected:**
- Preview text appears below the session/team count row
- Text is styled: `text-xs`, `italic`, `text-muted-foreground/50`, single line with overflow ellipsis (`line-clamp-1`)
- Spacing is `mt-0.5` below the metadata row

---

### TC-14 — Frontend: Card layout unchanged for entries without sessions

**What:** Verify `EntryRow` layout is unchanged (no empty space, no placeholder text) for entries with no sessions.

**Steps:**
1. Load the ponder page in a browser
2. Identify a ponder entry with no sessions
3. Inspect the entry card

**Expected:**
- No third row visible below the session/team metadata row
- Card height is the same as before this feature
- No empty or whitespace placeholder text

---

### TC-15 — Build: Tests and linter pass

**What:** Verify all Rust tests and Clippy linter pass after implementation.

**Steps:**
1. Run `SDLC_NO_NPM=1 cargo test --all`
2. Run `cargo clippy --all -- -D warnings`

**Expected:**
- All tests pass (0 failures)
- Clippy reports 0 warnings

---

## Pass/Fail Criteria

This QA run **passes** if:
- All 15 test cases pass
- No TypeScript compilation errors
- `cargo test --all` and `cargo clippy` both exit 0

This QA run **fails** if:
- Any unit test for `extract_session_preview` returns unexpected output
- `GET /api/roadmap` returns a non-200 response due to session reading errors
- Frontend renders preview for entries with no sessions
- TypeScript compilation fails
- Any cargo test fails or clippy reports warnings
