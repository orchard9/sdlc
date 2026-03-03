# Design: Finding-Closure Protocol — sdlc-next Template + CLAUDE.md Ethos

## Summary

This is a documentation-only change. Two files are modified:

1. `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — the Rust source that embeds the sdlc-next slash command template
2. `CLAUDE.md` — the project ethos document

Both changes add the three-action finding-closure protocol for `approve_audit` and `approve_review` directives.

## Change 1: sdlc_next.rs — SDLC_NEXT_COMMAND

Split the approval section into two tracks:

**Track A — Planning artifacts** (`approve_spec`, `approve_design`, `approve_tasks`, `approve_qa_plan`, `approve_merge`):
- Read artifact, verify it is complete and correct
- Run `sdlc artifact approve <slug> <artifact_type>` autonomously

**Track B — Review and audit artifacts** (`approve_review`, `approve_audit`):
1. Read the artifact and enumerate every finding
2. For each finding, take exactly one action before approving:
   - **Fix now** — implement a targeted code change (not a broad `/fix-all` or `/remediate` sweep)
   - **Track** — `sdlc task add <slug> "finding: <summary>"` to address in a future cycle
   - **Accept** — document why no action is needed
3. No finding may be silently skipped — every one must be explicitly resolved
4. `sdlc artifact approve <slug> <artifact_type>` only after all findings are accounted for

## Change 2: sdlc_next.rs — SDLC_NEXT_PLAYBOOK

Add step 5a after the generic approval step:

> **5a. For `approve_review` and `approve_audit`:** enumerate every finding.
> Fix now (targeted), track (`sdlc task add`), or accept (document rationale).
> No silent skips. Approve only after all findings are resolved.

## Change 3: CLAUDE.md Ethos

Add bullet to the Ethos section:

> **Audits and reviews close every finding.** When `approve_audit` or `approve_review`
> is the directive, enumerate every finding and take one explicit action: fix it now
> (targeted code change), track it (`sdlc task add`), or accept it (documented rationale).
> Silence is not acceptance. Use targeted fixes for specific findings — `fix-all` and
> `remediate` are for systemic codebase-wide patterns, not individual audit items.

## No Alternatives Considered

The three-action protocol is already authoritative — it was decided in the
`ponder-audits-reviews-fix-all-and-remediate` ponder session. This design simply
documents where those changes land and what exact wording is used.

## Files Modified

- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs`
- `CLAUDE.md`

## Implementation Note

As of commit `8fb4295`, all three changes are already present in the codebase. The task
list for this feature will verify the implementation matches the spec and confirm no
regressions were introduced.
