# QA Plan: Feature create_design action produces HTML mockup

## Scope

This feature modifies agent-facing instruction text only. QA focuses on:

1. Correctness of the instruction changes
2. Propagation through the `sdlc update` flow
3. Test suite integrity (no regressions)

---

## Test Cases

### QA-1: `rules.rs` message includes HTML mockup requirement

**How:** Read the `needs_design` rule in `crates/sdlc-core/src/rules.rs`.

**Pass:** The `message` closure for rule `needs_design` contains "mockup.html" and
explicitly distinguishes UI vs non-UI feature guidance.

**Pass:** The `message` closure for rule `design_rejected` also references `mockup.html`
for UI features.

---

### QA-2: `SDLC_NEXT_COMMAND` includes HTML mockup sub-step

**How:** Read `SDLC_NEXT_COMMAND` constant in
`crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`.

**Pass:** The "artifact creation" section contains a subsection for
`create_design` on UI features that mentions:
- `mockup.html`
- inline `<style>` and `<script>`
- no external resources
- named `<section id="screen-*">` blocks
- `[Mockup](mockup.html)` reference from `design.md`

---

### QA-3: `SDLC_NEXT_PLAYBOOK` includes HTML mockup guidance

**How:** Read `SDLC_NEXT_PLAYBOOK` constant in the same file.

**Pass:** Step 4 "For creation actions" includes a bullet about `mockup.html` for
`create_design` on UI features.

---

### QA-4: `SDLC_NEXT_SKILL` includes HTML mockup guidance

**How:** Read `SDLC_NEXT_SKILL` constant in the same file.

**Pass:** "For creation actions" step note mentions `mockup.html`, self-contained HTML,
nav bar, and screen sections.

---

### QA-5: `cargo test` passes with no regressions

**How:** Run `SDLC_NO_NPM=1 cargo test --all 2>&1`.

**Pass:** All tests pass. No test assertions rely on the literal message text of the
`needs_design` rule (tests only check `action` type and `transition_to`).

---

### QA-6: `cargo clippy` clean

**How:** Run `cargo clippy --all -- -D warnings`.

**Pass:** Zero warnings.

---

### QA-7: `sdlc next` directive message contains HTML requirement

**How:** Create a test feature, advance it to `specified` phase, run
`sdlc next --for <slug> --json`, and inspect the `message` field.

**Pass:** `message` contains "mockup.html" or equivalent language instructing the agent
to produce an HTML file.

---

### QA-8: HTML mockup format contract is documented

**How:** Read the updated `rules.rs` message and `sdlc_next.rs` instruction text.

**Pass:** At minimum the following constraints are documented:
- Single file (no external dependencies)
- Inline CSS and JS
- Valid HTML5
- Navigation between screens
- Named section elements

---

## Out of Scope

- Manual UAT in a browser (mockup.html is a sample; format compliance is by reading)
- Test coverage of the HTML mockup content itself
- Integration tests for `/design-feature` command (no such command exists in the test suite)
