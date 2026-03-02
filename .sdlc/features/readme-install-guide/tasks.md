# Tasks: SSH and make install in README

## T1 — Add SSH URL option to cargo install block

**File:** `README.md`

In the install section, locate the `cargo install --git https://github.com/orchard9/sdlc sdlc-cli` line (or block) and replace/expand it to present SSH URL as the primary labeled option and HTTPS as a secondary option:

```markdown
# Multi-SSH-key setups, corporate proxies — use SSH URL:
cargo install --git ssh://git@github.com/orchard9/sdlc sdlc-cli

# Or if HTTPS works in your environment:
cargo install --git https://github.com/orchard9/sdlc sdlc-cli
```

**Done when:** Both SSH and HTTPS variants appear, SSH first, with comments labeling each.

---

## T2 — Add "Build from source" subsection

**File:** `README.md`

After the `cargo install` block in the install section, add a new "Build from source" subsection:

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

**Done when:** The `git clone` + `make install` block is present and `DEVELOPER.md` is linked.

---

## T3 — Add DEVELOPER.md callout at end of install section

**File:** `README.md`

At the end of the install section (after the new subsections added in T1 and T2), add:

```markdown
> For the full contributor development setup (hot reload, tests, build targets), see [DEVELOPER.md](DEVELOPER.md).
```

**Done when:** The callout blockquote linking to DEVELOPER.md appears at the end of the install section.
