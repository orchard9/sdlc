# QA Plan: Ponder Design Artifact Protocol

## Scope

This feature adds skill instruction text to `sdlc_ponder.rs`. QA verifies:
1. The text changes are present and correct
2. The build compiles without errors
3. No regressions in existing tests

## Test Cases

### QC-1 — SDLC_PONDER_COMMAND contains Design Artifact Protocol section

**How to verify:** Read `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs` and
confirm `SDLC_PONDER_COMMAND` contains:
- The heading `### Design Artifact Protocol`
- The UI trigger condition (screen, panel, modal, widget, layout, interaction flow)
- The HTML format requirements: Tailwind CDN, `bg-gray-950`, prototype banner
- The filename convention `<name>-mockup.html`
- The two-step capture procedure with `/tmp/` staging
- The "When NOT to use HTML" list (data models, CLI syntax, API contracts, algorithms)
- The section is inserted after the capture examples and before
  `### Recruiting additional partners`

**Pass:** All elements present and correctly placed.

### QC-2 — SDLC_PONDER_PLAYBOOK contains condensed HTML protocol

**How to verify:** Read `SDLC_PONDER_PLAYBOOK` in the same file and confirm step 5
(artifact capture) mentions:
- HTML mockup for UI designs with `<name>-mockup.html`
- Tailwind CDN and dark theme
- `/tmp/` staging then capture
- Non-UI stays Markdown

**Pass:** All elements present.

### QC-3 — SDLC_PONDER_SKILL contains minimal HTML note

**How to verify:** Read `SDLC_PONDER_SKILL` in the same file and confirm the capture
step mentions HTML mockups for UI designs, `/tmp/` staging, and that non-UI stays
Markdown.

**Pass:** Elements present in minimal form.

### QC-4 — Build succeeds

```bash
SDLC_NO_NPM=1 cargo build --all
```

**Pass:** Exit code 0, no errors.

### QC-5 — Tests pass

```bash
SDLC_NO_NPM=1 cargo test --all
```

**Pass:** Exit code 0, all tests green.

### QC-6 — Clippy clean

```bash
cargo clippy --all -- -D warnings
```

**Pass:** Exit code 0, no warnings.

## Out of Scope

- End-to-end agent behavior testing (agents using the skill to produce mockups)
- UI rendering of HTML scrapbook artifacts (separate feature)
