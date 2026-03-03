# Spec: Finding-Closure Protocol — sdlc-next Template + CLAUDE.md Ethos

## Problem

When agents execute `approve_audit` or `approve_review` directives, they may approve artifacts
without addressing individual findings. The prior `sdlc-next` template only said "read the artifact,
verify it is complete and correct, approve." There was no protocol specifying what to do with the
enumerated findings — agents could silently skip findings or over-correct by running broad
`fix-all` / `remediate` sweeps instead of targeted fixes.

## Goal

Establish a mandatory three-action finding-closure protocol for `approve_audit` and
`approve_review` directives in:

1. The **`sdlc-next` slash command template** (primary agent execution path)
2. The **CLAUDE.md Ethos section** (project-level authority document)

Every finding must receive exactly one of three dispositions:
- **Fix now** — a targeted code change committed before approving
- **Track** — `sdlc task add` to create a follow-up task
- **Accept** — documented rationale written inline

## Requirements

1. The `sdlc-next` Claude command template (`SDLC_NEXT_COMMAND` in `sdlc_next.rs`) must:
   - Have a dedicated subsection for `approve_review` and `approve_audit` that is separate
     from the generic approval track
   - Enumerate the three dispositions explicitly
   - Prohibit silent skips ("No finding may be silently skipped")
   - Distinguish targeted fixes from broad `fix-all` / `remediate` sweeps

2. The `sdlc-next` Gemini/OpenCode playbook variant (`SDLC_NEXT_PLAYBOOK`) must:
   - Include a step 5a (or equivalent) covering the `approve_review` / `approve_audit` path
   - Reference all three dispositions

3. The **CLAUDE.md Ethos section** must contain the bullet:
   > **Audits and reviews close every finding.** When `approve_audit` or `approve_review`
   > is the directive, enumerate every finding and take one explicit action: fix it now
   > (targeted code change), track it (`sdlc task add`), or accept it (documented rationale).
   > Silence is not acceptance. Use targeted fixes for specific findings — `fix-all` and
   > `remediate` are for systemic codebase-wide patterns, not individual audit items.

## Out of Scope

- Changes to `.sdlc/guidance.md` — that is the responsibility of the `audit-review-guidance-section` feature
- Changes to the SDLC Rust state machine rules
- Changes to other slash command templates

## Acceptance Criteria

- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` contains a dedicated approval
  subsection for `approve_review` and `approve_audit` with fix/track/accept protocol
- `SDLC_NEXT_PLAYBOOK` references the three-disposition protocol for review/audit approvals
- `CLAUDE.md` contains the "Audits and reviews close every finding" ethos bullet
- No other sections in `sdlc_next.rs` or `CLAUDE.md` are unintentionally modified

## Status

The implementation changes described here are already present in the codebase:
- `sdlc_next.rs` was created in commit `8fb4295` with the full finding-closure protocol
- `CLAUDE.md` already has the "Audits and reviews close every finding" ethos bullet

This feature is being advanced through the SDLC lifecycle to formally track and close
the work that was implemented.
