# QA Results: SSH and make install in README

## Result: PASSED

## Checks

### 1. Content accuracy

- [x] SSH URL `ssh://git@github.com/orchard9/sdlc` present at README line 31
- [x] HTTPS URL `https://github.com/orchard9/sdlc` preserved at README line 34
- [x] `make install` target exists in `Makefile` line 4 — runs `frontend` then `cargo install --path crates/sdlc-cli` (accurate to doc)
- [x] `git clone git@github.com:orchard9/sdlc.git` at README line 42 — correct SSH clone syntax
- [x] `DEVELOPER.md` exists at repo root — relative link resolves correctly

### 2. Formatting

- [x] Code blocks properly triple-backtick fenced (bash shell blocks)
- [x] Comment lines inside bash blocks use `#` prefix
- [x] DEVELOPER.md blockquote uses `>` prefix at README line 50
- [x] Bold `**...**` heading style consistent with existing install sub-options

### 3. No regressions

- [x] prebuilt binary (macOS/Linux) install block unchanged (lines 9-13)
- [x] Windows PowerShell install block unchanged (lines 15-19)
- [x] Homebrew install block unchanged (lines 21-25)
- [x] `sdlc --version` verify step present at line 55
- [x] "Building from Source" section at bottom of README (line 426+) untouched
- [x] All sections after "Initialize a project" unchanged

### 4. Markdown rendering

- [x] DEVELOPER.md link relative path resolves — file exists
- [x] No broken fences or unclosed backtick blocks
- [x] New subsection fits naturally in the install flow with correct heading hierarchy

## Summary

All 4 QA categories passed. The three changes (T1: SSH URL, T2: `make install` subsection, T3: DEVELOPER.md blockquote) are present, accurate, and introduce no regressions.
