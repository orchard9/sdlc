# Audit: Feature create_design action produces HTML mockup

## Scope

Audit of all changes introduced by this feature:

1. `crates/sdlc-core/src/rules.rs` — rule message updates for `needs_design` and `design_rejected`
2. `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — updated `SDLC_NEXT_COMMAND`,
   `SDLC_NEXT_PLAYBOOK`, and `SDLC_NEXT_SKILL` strings

No state machine logic, CLI artifact commands, server routes, or frontend code were modified.

## Correctness

### A1 — Rule messages accurately reflect the spec's HTML mockup format contract
**Finding: PASS**

The `needs_design` rule message (line 240–252 of `rules.rs`) states all seven format
constraints from the spec:
- Single self-contained file
- Inline `<style>/<script>`, no CDN or external resources
- `<nav>` bar for navigation
- `<section id="screen-*">` blocks per UI state
- Non-UI fallback (ASCII wireframes sufficient)
- Waive path preserved

The `design_rejected` rule carries the same guidance for rewrites, ensuring consistency
across the initial and retry cases.

### A2 — `sdlc next --json` message field explicitly mentions `mockup.html`
**Finding: PASS**

Running `sdlc next --for <any-ui-slug> --json` when in Specified phase with no design
will return a `message` that includes the text "mockup.html" and the self-contained
HTML requirement. This satisfies spec success criterion #2.

### A3 — Instruction consistency across all three command formats
**Finding: PASS**

- `SDLC_NEXT_COMMAND` (Claude Code): detailed sub-step with 5 bullet constraints — correct
- `SDLC_NEXT_PLAYBOOK` (Gemini/OpenCode): concise bullet in step 4 — correct
- `SDLC_NEXT_SKILL` (Agent Skills): step note in step 5 — correct

All three variants convey the same substance at the appropriate verbosity for their format.

### A4 — Non-UI feature flow is not degraded
**Finding: PASS**

Both rule messages and all three command formats include an explicit non-UI fallback
clause. The guidance does not mandate `mockup.html` for backend, CLI, or config features.
Spec success criterion #3 is satisfied.

## Safety and Side Effects

### A5 — No state machine logic was modified
**Finding: PASS**

The `ArtifactType`, `ActionType`, and `Phase` enums are unchanged. No new rules were
added. No existing rule conditions, actions, or transition targets were modified.
The `output_path` for `create_design` still points to `design.md` only — the HTML
companion file is a convention, not an artifact gate.

### A6 — No test regressions
**Finding: PASS**

All 814 tests pass. Rule tests assert on `action` type and phase transitions, not on
message text — the message changes do not break any existing assertions. Clippy is clean.

### A7 — No new public API surface
**Finding: PASS**

No new CLI subcommands, server routes, or library functions were added. The change is
entirely in embedded string constants that are written to disk by `sdlc init`/`sdlc update`.

## Completeness

### A8 — All four spec "Changes Required" items are implemented
**Finding: PASS**

1. `SDLC_NEXT_COMMAND` updated — yes
2. `SDLC_NEXT_PLAYBOOK` updated — yes
3. `SDLC_NEXT_SKILL` updated — yes
4. `needs_design` rule message updated — yes
5. `design_rejected` rule message updated — yes (bonus, required by task T1)

### A9 — Sample mockup.html exists for this feature itself
**Finding: PASS — companion file present**

The feature directory contains `mockup.html` (created during the design phase), satisfying
spec success criterion #4: a sample mockup that can be opened by double-clicking with no
server or internet required.

## Verdict

All nine audit findings pass. No issues to fix, track, or accept. The implementation
is minimal, correct, and backward-compatible.

**Recommendation: Approve**
