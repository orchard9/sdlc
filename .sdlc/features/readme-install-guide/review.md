# Review: SSH and make install in README

## Changes Applied

Three targeted edits were made to `README.md` in the Install section. No other files were modified.

### T1: SSH URL added to `cargo install --git` block

The existing HTTPS-only block was updated to show both options, with SSH labeled first for users who need it:

```bash
# Multi-SSH-key setups, corporate proxies — use SSH URL:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

Rationale confirmed: `cargo install --git` accepts the `ssh://` scheme and this is the correct form for SSH-keyed git access.

### T2: `make install` subsection added

A new "Build from source" subsection was added immediately after the "From source" block:

```bash
git clone git@github.com:orchard9/sdlc.git
cd sdlc
make install
```

With the explanatory note: "`make install` builds the frontend and installs the binary in one step."

Verified against `Makefile`: the `install` target runs `frontend` (npm ci + npm run build) then `cargo install --path crates/sdlc-cli` — accurately described.

### T3: DEVELOPER.md blockquote added

A blockquote callout was added at the end of the install block:

```
> For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md).
```

`DEVELOPER.md` exists at the repo root. The relative path resolves correctly.

## Verification Checklist

- [x] SSH URL is correct format for `cargo install --git`
- [x] HTTPS URL is preserved as fallback (unchanged)
- [x] `make install` target exists in `Makefile` and does what the doc claims
- [x] `git clone` SSH URL is correct for orchard9/sdlc
- [x] `DEVELOPER.md` exists at repo root
- [x] All code blocks properly fenced
- [x] Existing install options (prebuilt binary, Windows, Homebrew) unchanged
- [x] `sdlc --version` verify step still present
- [x] "Building from Source" section at bottom of README untouched
- [x] No regressions to subsequent README sections

## Finding Resolution

No findings. The change is additive and minimal — exactly the 3 edits specified in the spec.

## Verdict

Approved. The changes are accurate, well-scoped, and correctly address the two gaps identified in the spec.
