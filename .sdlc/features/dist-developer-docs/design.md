# Design: DEVELOPER.md Cross-Platform Dev Install Path

## Overview

This is a pure documentation change. No Rust source files, Justfile, or configuration files are
modified. All changes are confined to `DEVELOPER.md` at the repository root.

## Document Structure Changes

The updated `DEVELOPER.md` will have the following Install section structure:

```
## Install

**All platforms — using `just` (recommended):**
  - just install command
  - just installation instructions (cargo / brew / winget)

**Without `just` — manual cargo install:**
  - Step-by-step cargo build and install commands

**Other just recipes:**
  - Table of build, test, lint, clean
```

### Install Section Detail

1. **Primary path** — `just install` with a note that `just` must be installed first.
   Install instruction shows all three managers on one line:
   ```
   cargo install just   # or: brew install just  |  winget install just
   ```

2. **Fallback path (no just)** — A dedicated subsection for contributors who prefer not to install `just`:
   ```bash
   cd frontend && npm ci && npm run build  # build frontend
   cargo install --path crates/sdlc-cli --locked
   # create sdlc alias (unix: ln -sf; windows: hardlink or batch)
   ```

3. **Other recipes table** — Keep existing table but verify all recipes are current.

## No-Design Rationale (why this is simple)

- No API surface changes.
- No data model changes.
- No UI changes.
- Single file: `DEVELOPER.md`.
- Changes are additive (adding a fallback section) and clarifying (verifying existing content).
