# QA Plan: SSH and make install in README

## Scope

This is a documentation-only change to `README.md`. All QA checks are manual review of the rendered Markdown and link correctness — no automated tests required.

## Checks

### 1. SSH URL option present and correctly formatted

- [ ] The `cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli` line appears in the install section
- [ ] The SSH URL is labeled with a comment (`# Multi-SSH-key setups, corporate proxies — use SSH URL:`)
- [ ] The HTTPS URL (`cargo install --git https://github.com/orchard9/sdlc sdlc-cli`) follows as a secondary option with a comment
- [ ] SSH URL appears before HTTPS URL in the code block

### 2. Build from source subsection present

- [ ] A "Build from source" heading or bold label exists in the install section
- [ ] `git clone git@github.com:orchard9/sdlc.git` is present
- [ ] `cd sdlc` and `make install` are present as sequential commands
- [ ] A sentence describing `make install` ("builds the frontend and installs the binary in one step") is present
- [ ] A Markdown link `[DEVELOPER.md](DEVELOPER.md)` appears in this subsection

### 3. DEVELOPER.md callout at end of install section

- [ ] A blockquote (`>`) callout referencing `[DEVELOPER.md](DEVELOPER.md)` appears at the end of the install section
- [ ] The callout text matches the spec: "For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md)."

### 4. No regressions

- [ ] Existing install instructions are not removed or broken
- [ ] No unrelated sections of README.md were modified
- [ ] The Markdown renders correctly (no unclosed fences, no broken headings)

### 5. Links resolve

- [ ] `DEVELOPER.md` exists at the repo root (confirming the link target is valid)

## Pass Criteria

All 5 check groups pass with no items failing.
