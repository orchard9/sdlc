# QA Plan: SSH and make install in README

## Scope

README.md documentation — 3 targeted edits to the install section. No code changes.

## Verification Checks

### 1. Content accuracy

- [ ] SSH URL comment is accurate: `ssh://git@github.com/orchard9/sdlc` is the correct SSH URL format for `cargo install --git`
- [ ] HTTPS URL is unchanged: `https://github.com/orchard9/sdlc` is still present as a fallback
- [ ] `make install` target exists in `Makefile` and does what the README claims (builds frontend + installs binary)
- [ ] `git clone git@github.com:orchard9/sdlc.git` is the correct SSH clone URL
- [ ] DEVELOPER.md exists at the repo root and is linked correctly (relative path `DEVELOPER.md`)

### 2. Formatting

- [ ] All code blocks use correct triple-backtick fencing
- [ ] Comment lines inside bash blocks use `#` prefix
- [ ] Blockquote for the DEVELOPER.md link uses `>` prefix
- [ ] Section headings match the existing README heading style (bold `**...**` for install sub-options)

### 3. No regressions

- [ ] Existing install options (prebuilt binary, Windows PowerShell, Homebrew) are unchanged
- [ ] The "Verify" step (`sdlc --version`) is still present after the install section
- [ ] "Initialize a project" and subsequent sections are unchanged
- [ ] "Building from Source" section at the bottom of README is untouched (it documents the build from source differently — the new section in Install is for `make install` convenience)

### 4. Markdown rendering

- [ ] All links resolve correctly in rendered Markdown (DEVELOPER.md exists)
- [ ] No broken fences or unclosed backtick blocks
- [ ] The new subsection fits naturally in the install flow without breaking the existing heading hierarchy

## Pass Criteria

All 4 categories pass. Since this is documentation-only, no automated tests need to run. Manual diff review of `README.md` is sufficient.
