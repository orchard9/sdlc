# Review: SSH and make install in README

## Summary

This is a documentation-only change. Three targeted edits were made to the `### Install` section of `README.md`. No code, no tests, no config changes.

## Changes Reviewed

**File:** `README.md` (lines 27–51)

### T1 — SSH URL option

The `cargo install --git` block now shows two labeled options:

```bash
# Multi-SSH-key setups, corporate proxies — use SSH URL:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

- SSH URL appears first with a comment explaining when to use it. PASS.
- HTTPS URL follows as a secondary option. PASS.
- Labels are clear and actionable. PASS.

### T2 — Build from source subsection

```markdown
**Build from source** (after cloning):

```bash
git clone git@github.com:orchard9/sdlc.git
cd sdlc
make install
```

`make install` builds the frontend and installs the binary in one step.
See [DEVELOPER.md](DEVELOPER.md) for the full contributor setup.
```

- `git clone` uses SSH URL (consistent with T1 — good). PASS.
- `make install` is present and clearly labeled. PASS.
- `DEVELOPER.md` link is present. PASS.
- `DEVELOPER.md` exists at the repo root — link target is valid. PASS.

### T3 — DEVELOPER.md callout

```markdown
> For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md).
```

- Present at the end of the install subsection, before the `Verify:` block. PASS.
- Blockquote format renders well on GitHub. PASS.
- Link target verified to exist. PASS.

## No Regressions

- Prebuilt binary install paths (macOS/Linux, Windows, Homebrew) are unchanged.
- Existing HTTPS `cargo install` line preserved — only expanded.
- `The build script automatically compiles the frontend — no manual npm step needed.` line preserved.
- No other sections of README.md were touched.

## Verdict

All three tasks implemented correctly. All spec requirements met. No regressions. Ready to advance.
