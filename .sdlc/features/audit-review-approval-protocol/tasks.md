# Tasks: Finding-Closure Protocol — sdlc-next Template + CLAUDE.md Ethos

## T1: Verify sdlc_next.rs SDLC_NEXT_COMMAND contains the finding-closure protocol

Confirm that `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` has:
- A dedicated approval subsection for `approve_review` and `approve_audit` (separate from the planning-artifact approval track)
- The three dispositions: fix now / track / accept
- Prohibition on silent skips
- Distinction between targeted fixes and broad `fix-all` / `remediate` sweeps

If any element is missing, implement it.

## T2: Verify sdlc_next.rs SDLC_NEXT_PLAYBOOK contains the finding-closure protocol

Confirm that `SDLC_NEXT_PLAYBOOK` in `sdlc_next.rs` has:
- A step covering `approve_review` and `approve_audit`
- Reference to the three-disposition protocol (fix now / track / accept)
- Prohibition on silent skips

If any element is missing, implement it.

## T3: Verify CLAUDE.md Ethos section contains the finding-closure bullet

Confirm that `CLAUDE.md` contains the bullet:
> **Audits and reviews close every finding.** ...

Verify exact content matches the spec. If missing or incomplete, add/fix it.

## T4: Build and test

Run `SDLC_NO_NPM=1 cargo test --all` to confirm no regressions were introduced.
