# Final Synthesis: Deliverables

## Milestone: UAT Failure Pathways (v34 or next available)

### Vision
When a milestone UAT fails, the agent never stalls. It triages every failure into one of three pathways — fix-and-retry, escalate, or recap-and-ponder — and takes the action before ending. The human returns to a clean state with concrete next steps, not a dead-end "fix it yourself" message.

### Features

**Feature 1: uat-failure-pathways**
- Modify `SDLC_MILESTONE_UAT_COMMAND` template in `sdlc_milestone_uat.rs`
- Add failure triage classification (fixable / escalation / complex)
- Add Pathway 1: fix code + rerun (max 2 retries, then fall through)
- Add Pathway 2: create escalation + tasks + signal failure
- Add Pathway 3: synthesize recap + create ponder entries + commit + signal failure
- Update playbook and skill variants
- Update Step 6 final report table

**Feature 2: sdlc-recap-command**
- New `sdlc_recap.rs` in `crates/sdlc-cli/src/cmd/init/commands/`
- Four platform variants (Claude, Gemini, OpenCode, Agent Skills)
- State-aware: reads `sdlc status`, milestone info, recent git history
- Produces: Working On / Completed / Remaining / Forward Motion
- Forward Motion creates real artifacts (tasks, escalations, ponder entries)
- Register in `write_user_*` functions + guidance table + migration

**Feature 3 (optional, wave 2): recap-server-integration**
- Server endpoint for recap runs
- Frontend "Run Recap" button on UAT failure UI
- Parked unless the template-only approach proves insufficient

### Acceptance Test
1. Run `/sdlc-milestone-uat <slug>` on a milestone with known failing tests
2. Agent classifies failures and enters appropriate pathway
3. On Pathway 1: agent fixes code and reruns, milestone completes
4. On Pathway 3: agent creates ponder entries, commits work, outputs next steps
5. Run `/sdlc-recap` standalone — produces state-aware recap with forward motion
6. Every recap ends with exactly one **Next:** line

### Definition of done
- UAT template contains triage + 3 pathways
- `/sdlc-recap` command is installed via `sdlc update`
- No session ends without a concrete next action
