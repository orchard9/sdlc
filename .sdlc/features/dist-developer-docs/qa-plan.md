# QA Plan: DEVELOPER.md Cross-Platform Dev Install Path

## Scope

Verify that `DEVELOPER.md` has been updated correctly per the acceptance criteria.

## Test Cases

### TC1 — No make references

**Step:** Search `DEVELOPER.md` for the word "make" (case-insensitive).
**Command:** `grep -i "make" DEVELOPER.md`
**Expected:** Zero matches.

### TC2 — just install instruction completeness

**Step:** Check the `just` install line in `DEVELOPER.md`.
**Expected:** The install instruction for `just` shows all three managers on one line:
`cargo install just   # or: brew install just  |  winget install just`

### TC3 — "Without just" fallback section exists

**Step:** Verify `DEVELOPER.md` contains a fallback section for contributors without `just`.
**Expected:**
- Section is clearly labeled (e.g., "**Without `just`**" or "**Manual install (no just):**")
- Contains `cargo install --path crates/sdlc-cli --locked` or equivalent
- Contains frontend build step

### TC4 — just recipe table accuracy

**Step:** Compare the just recipe table in `DEVELOPER.md` against the `justfile`.
**Expected:** Every recipe listed in the table exists in the Justfile. Commonly: `build`, `test`, `lint`, `clean`.

### TC5 — Windows dev loop variant present

**Step:** Check that the Dev Loop section in `DEVELOPER.md` includes a Windows PowerShell variant.
**Expected:** A PowerShell code block showing `$env:SDLC_ROOT = "C:\path\to\your-project"` and `cargo watch`.

### TC6 — Document renders cleanly

**Step:** Review `DEVELOPER.md` for markdown formatting issues (unclosed code fences, broken headers).
**Expected:** Clean, well-structured Markdown.
