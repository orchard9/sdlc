# Spec: Add §12 Audit & Review Findings to .sdlc/guidance.md

## Problem

The CLAUDE.md ethos section states:

> **Audits and reviews close every finding.** When `approve_audit` or `approve_review` is the directive, enumerate every finding and take one explicit action: fix it now (targeted code change), track it (`sdlc task add`), or accept it (documented rationale). Silence is not acceptance. Use targeted fixes for specific findings — `fix-all` and `remediate` are for systemic codebase-wide patterns, not individual audit items.

This critical protocol exists in CLAUDE.md but is absent from `.sdlc/guidance.md`, which is the primary reference document that agents read before all implementation work. As a result, agents running `approve_audit` or `approve_review` directives may not follow the mandatory three-action protocol (fix / track / accept-with-rationale), leaving findings silently unaddressed.

## Goal

Add a §12 section to `.sdlc/guidance.md` that clearly instructs agents on how to handle audit and review findings. The section must match the authority and style of existing sections (e.g., §9 Escalating to the Human) and be immediately actionable without ambiguity.

## Requirements

1. The new section must be numbered §12 and titled "Audit & Review Findings".
2. It must state that every finding from `approve_audit` or `approve_review` requires one of exactly three dispositions:
   - **Fix now** — targeted code change, committed before approving
   - **Track** — `sdlc task add <slug> "<finding>"` to create a follow-up task
   - **Accept** — documented rationale written inline (silence is not acceptance)
3. It must distinguish targeted fixes (for individual audit/review items) from systemic fix-all/remediate commands (for codebase-wide patterns).
4. It must be positioned after the existing §11 Project Guidelines section.
5. The writing style must match the imperative, directive tone of existing guidance sections.
6. No other sections in guidance.md may be modified.

## Out of Scope

- Changes to CLAUDE.md (already has this content)
- Changes to agent skills or slash commands
- Changes to the SDLC state machine rules

## Acceptance Criteria

- `guidance.md` contains a §12 section with the required content
- All three dispositions (fix/track/accept) are clearly described
- The fix-all/remediate distinction is present
- The section is positioned at the end of the file after §11
- All existing sections (§1–§11) are unchanged
