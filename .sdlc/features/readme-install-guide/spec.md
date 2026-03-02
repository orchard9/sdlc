# Spec: SSH and make install in README

## Problem

The README install section has two gaps that cause first-time install failures for a significant class of users:

1. **HTTPS-only `cargo install` URL**: `cargo install --git https://github.com/orchard9/sdlc sdlc-cli` fails silently for users with multi-SSH-key setups, corporate SSL proxies, or firewalls that block raw HTTPS to GitHub. The SSH URL variant (`ssh://git@github.com/...`) works for all of these cases but is undocumented. Xist discovered it by accident.

2. **`make install` is invisible**: `make install` is the correct from-source install path (it handles both Rust backend and frontend build in one command), but it exists only in DEVELOPER.md, which README does not link to. Users who clone the repo (common in enterprise) have no documented path.

## Changes

### README.md — Install section

**File:** `README.md`

**Change 1: Add SSH URL as primary option in `cargo install --git` block**

Add the SSH URL as the first (labeled) option, with HTTPS as a secondary note:

```markdown
**From source** (requires Rust + Node.js ≥ 18):

```bash
# Multi-SSH-key setups, corporate proxies — use SSH URL:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```
```

**Change 2: Add `make install` as the primary clone-and-build path**

Add a "Build from source" subsection that shows `git clone` + `make install` as the recommended path for users who have cloned (or will clone) the repo:

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

**Change 3: Add one-sentence link from README to DEVELOPER.md**

At the end of the install section, add:
```markdown
> For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md).
```

## Scope

- **File:** `README.md` only
- **Changes:** 3 targeted edits to the install section — no code changes
- **Existing users:** No impact — additive documentation
- **Validation:** Manually verify both `cargo install --git ssh://...` and `make install` paths work in a clean environment before merging

## Non-Goals

- README structural restructure (install section ordering, prebuilt binary paths audit) — deferred to a separate feature
- DEVELOPER.md changes — DEVELOPER.md is already correct; this feature only adds a link from README
