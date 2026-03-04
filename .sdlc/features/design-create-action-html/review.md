# Review: Feature create_design action produces HTML mockup

## Summary

This feature upgrades the agent-facing instruction layer so that the `create_design`
directive for UI features explicitly requires a companion `mockup.html` file alongside
`design.md`. All changes are confined to the instruction layer and the rule message
text — no state machine logic, CLI commands, or server routes were modified.

## Files Changed

| File | Change |
|---|---|
| `crates/sdlc-core/src/rules.rs` | Updated `needs_design` and `design_rejected` rule message closures |
| `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` | Updated `SDLC_NEXT_COMMAND`, `SDLC_NEXT_PLAYBOOK`, `SDLC_NEXT_SKILL` |

## Findings

### Finding 1 — `rules.rs` message is accurate and complete
**Status: Accept — no action needed**

The `needs_design` rule (rule 6) message now explicitly states:
- For UI features: write `mockup.html` with inline `<style>/<script>`, no CDN, `<nav>` bar,
  `<section id="screen-*">` blocks per UI state, and `[Mockup](mockup.html)` reference in
  `design.md`.
- For non-UI features: ASCII wireframes or plain Markdown are sufficient.
- Waive path is preserved.

The `design_rejected` rule (rule 8) carries the same guidance for rewrites.

Both messages match the spec's HTML mockup format contract exactly.

### Finding 2 — `SDLC_NEXT_COMMAND` (Claude Code) sub-step is well-placed
**Status: Accept — no action needed**

The new sub-step appears directly under "For artifact creation", immediately after step 3
("Write a thorough Markdown artifact to `output_path`"). Placement is logical — the
agent reads the general instruction and immediately sees the UI-feature exception. The
non-UI fallback sentence is present. The five bullet constraints (single file, valid HTML5,
nav bar, named sections, representative design) match the spec's format contract table.

### Finding 3 — `SDLC_NEXT_PLAYBOOK` (Gemini / OpenCode) is consistent
**Status: Accept — no action needed**

Step 4 in the playbook now includes the mockup bullet after the artifact-write step, before
the `sdlc artifact draft` call. The guidance is concise and consistent with the command
format's bullet list, appropriate for the more condensed playbook style.

### Finding 4 — `SDLC_NEXT_SKILL` (Agent Skills format) is consistent
**Status: Accept — no action needed**

Step 5 in the skill includes the `mockup.html` note for UI features. The guidance matches
the other two variants in substance while fitting the minimal SKILL.md format.

### Finding 5 — No state machine tests were broken
**Status: Accept — verified by test run**

All 814 tests across all crates pass. The `rules.rs` message changes do not affect any
test assertions (tests assert on `action` type and phase transitions, not message text).
`cargo clippy --all -- -D warnings` reports zero warnings.

### Finding 6 — Scope boundary correctly respected
**Status: Accept — no action needed**

The spec explicitly excludes changes to the state machine, CLI artifact commands, server
routes, and the React frontend. None of those were modified. The companion file is a
convention enforced through instructions, not a new artifact gate — consistent with the
spec's "Out of Scope" section.

### Finding 7 — Backward compatibility preserved
**Status: Accept — no action needed**

The waive path (`sdlc artifact waive <slug> design --reason "..."`) is preserved in the
`needs_design` message. Non-UI feature guidance explicitly allows ASCII wireframes. No
existing workflows are broken.

## Verdict

All findings are resolved (all accepted as correct). The implementation is clean, minimal,
and fully aligned with the spec. Tests pass and clippy is clean.

**Recommendation: Approve**
