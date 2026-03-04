# Design: Feature create_design action produces HTML mockup

## Overview

This feature upgrades the `create_design` action's instruction surface to make HTML
mockup production an explicit deliverable for UI features. The change is entirely in the
agent-facing instruction layer — the state machine, CLI, and server are untouched.

An interactive HTML prototype lives alongside `design.md` in the feature directory.
It is committed to git, requires no server, and is viewable by double-clicking.

See companion mockup: [mockup.html](mockup.html) — a live example of what agents should
produce when executing `create_design` for UI features.

---

## Architecture

```
Instruction layer change only
─────────────────────────────
crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs
  SDLC_NEXT_COMMAND    ← Claude Code slash command text
  SDLC_NEXT_PLAYBOOK   ← Gemini / OpenCode text
  SDLC_NEXT_SKILL      ← Agent Skills format

crates/sdlc-core/src/rules.rs
  needs_design rule    ← message field (surfaced in sdlc next --json output)

No changes to:
  classifier.rs, types.rs, feature.rs   (state machine unchanged)
  sdlc-server                            (no route changes)
  frontend/                              (no UI changes)
```

The two-file deliverable contract is:

```
.sdlc/features/<slug>/
├── design.md       ← primary artifact (approved by state machine)
└── mockup.html     ← companion HTML file (referenced from design.md)
```

---

## Change 1 — `rules.rs` `needs_design` message

**Current message (paraphrased):**
> For UI features, include an HTML prototype or ASCII wireframes alongside design.md.

**New message (explicit requirement):**
> For UI features, write `mockup.html` alongside `design.md`: a single self-contained
> HTML file (inline `<style>` / `<script>`, no CDN or external resources) with named
> sections for each major screen or state, and a navigation bar to jump between them.
> Reference the file from `design.md` with a relative link. For non-UI features,
> ASCII wireframes or plain Markdown are sufficient.

This surfaces in `sdlc next --for <slug> --json` as the `message` field, so any agent
that reads the directive will see the updated requirement.

---

## Change 2 — `sdlc_next.rs` instruction update

### `SDLC_NEXT_COMMAND` (Claude Code)

Current "artifact creation" section says:
```
3. Write a thorough Markdown artifact to `output_path`
```

Updated section adds an HTML mockup sub-step for `create_design`:

```markdown
For **artifact creation** (`create_spec`, `create_design`, `create_tasks`,
`create_qa_plan`, `create_review`, `create_audit`):
1. Run `sdlc feature show <slug> --json` for context
2. Read existing artifacts in `.sdlc/features/<slug>/`
3. Write a thorough Markdown artifact to `output_path`

For `create_design` on a **UI feature**, also write `mockup.html`:
- Single self-contained file — inline `<style>` and `<script>`, no external resources
- Valid HTML5 with a `<nav>` or tab bar to navigate between screens
- Named sections (`<section id="screen-*">`) for each major UI state
- Representative colors and typography; pixel-perfect fidelity not required
- Reference it from `design.md`: `[Mockup](mockup.html)`
- Place the file in the same directory as `design.md`

Non-UI features (backend, CLI, config): mockup is optional; ASCII wireframes in
`design.md` are sufficient.
```

### `SDLC_NEXT_PLAYBOOK` (Gemini / OpenCode)

Same guidance added as a concise bullet under step 4.

### `SDLC_NEXT_SKILL` (Agent Skills)

Same guidance added as a step note under "For creation actions".

---

## HTML Mockup Format Contract

Documented in both `rules.rs` message and `sdlc_next.rs` instructions:

| Constraint | Rationale |
|---|---|
| Single file `mockup.html` | Committable to git, no server required |
| Inline `<style>` block | No external CSS dependency |
| Inline `<script>` block (if used) | Screen navigation without a bundler |
| `<!DOCTYPE html>` + `<html lang="en">` | Valid HTML5 |
| `<nav>` or tab bar at top | Reviewers navigate without scrolling |
| `<section id="screen-*">` per view | Predictable structure across features |
| No CDN references (`https://cdn.*`) | Offline-friendly, audit-friendly |
| File placed in `.sdlc/features/<slug>/` | Alongside `design.md` |

---

## Rollout

1. Edit `crates/sdlc-core/src/rules.rs` — update `needs_design` message.
2. Edit `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — update all three
   instruction strings.
3. Run `SDLC_NO_NPM=1 cargo test --all` — verify no regressions.
4. Run `cargo clippy --all -- -D warnings` — verify no warnings.

No migration required. Existing features with approved designs are unaffected.
Agents running `create_design` on future features will see the updated instructions.

---

## Non-Goals

- New `mockup` artifact type in the state machine — deferred; mockup is companion, not gate.
- Server endpoint to serve mockup HTML — unnecessary; file:// opens directly.
- Validation that `mockup.html` exists before design approval — nice-to-have, deferred.
