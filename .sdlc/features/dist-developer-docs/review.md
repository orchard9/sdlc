# Review: dist-developer-docs

## Summary

Single-file documentation change: `DEVELOPER.md` at the repository root.

**Change:** Added a "Without `just` — manual install" fallback section between the `just install`
section and the "Other recipes" table.

## Changes Reviewed

### DEVELOPER.md (lines 23–38, new section added)

```markdown
**Without `just` — manual install:**

If you prefer not to install `just`, you can install manually:

```bash
# 1. Build the frontend
cd frontend && npm ci && npm run build && cd ..

# 2. Install the binary
cargo install --path crates/sdlc-cli --locked

# 3. Create the sdlc alias
# macOS / Linux:
ln -sf "$(which ponder)" "$(dirname "$(which ponder)")/sdlc"
# Windows (PowerShell): create a hardlink manually or use the ponder binary directly
```
```

## Findings

### F1 — Commands match Justfile (PASS)

The `cd frontend && npm ci && npm run build` sequence matches the `_frontend` recipe's behavior in
the Justfile. The `cargo install --path crates/sdlc-cli --locked` matches the `install` recipe.
The `ln -sf` alias step matches the `_symlink` recipe.

### F2 — No make references (PASS)

`grep -i make DEVELOPER.md` returns zero matches.

### F3 — just install line complete (PASS)

Line 20: `cargo install just   # or: brew install just  |  winget install just` — all three
managers present.

### F4 — Windows dev loop present (PASS)

Lines 86–90: PowerShell variant with `$env:SDLC_ROOT` documented.

### F5 — Fallback section present and correct (PASS)

New section is clearly labeled "Without `just` — manual install", contains all three steps
(frontend build, binary install, alias creation), and includes a Windows note for the alias step.

### F6 — Recipe table accurate (PASS)

The "Other recipes" table lists `build`, `test`, `lint`, `clean`, and `just` (list). All five
recipes exist in the Justfile.

## Decision

**APPROVE** — All acceptance criteria met. No issues found. Change is additive and non-breaking.
