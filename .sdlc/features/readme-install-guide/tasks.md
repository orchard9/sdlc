# Tasks: SSH and make install in README

## T1: Add SSH URL option to `cargo install --git` block

**File:** `README.md`

In the "From source" install block, add the SSH URL as the primary labeled option with HTTPS as a secondary fallback. The SSH URL (`ssh://git@github.com/orchard9/sdlc`) works for users with multi-SSH-key setups, corporate SSL proxies, and firewalls that block HTTPS to GitHub.

**Before:**
```markdown
**From source** (requires [Rust](https://rustup.rs) and [Node.js ≥ 18](https://nodejs.org)):

```bash
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```
```

**After:**
```markdown
**From source** (requires [Rust](https://rustup.rs) and [Node.js ≥ 18](https://nodejs.org)):

```bash
# Multi-SSH-key setups, corporate proxies — use SSH URL:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```
```

---

## T2: Add `make install` as the primary clone-and-build path

**File:** `README.md`

After the "From source" cargo install block, add a "Build from source" subsection documenting `git clone` + `make install` as the recommended path for users who clone the repo. `make install` is the correct command — it builds the frontend and installs the binary in one step.

**Add after the existing "From source" block:**
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

---

## T3: Add link from README install section to DEVELOPER.md

**File:** `README.md`

At the end of the install section (before "Initialize a project"), add a one-sentence callout linking to DEVELOPER.md for users who want the full contributor development setup.

**Add at the end of the Install section:**
```markdown
> For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md).
```
