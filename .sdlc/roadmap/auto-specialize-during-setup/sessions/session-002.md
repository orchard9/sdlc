---
session: 2
timestamp: 2026-03-06T23:45:00Z
orientation:
  current: "Diagnosed both Phase 6 failures — agent creation incomplete + UI reads wrong directory. Specialize replacement is validated. Ready to implement."
  next: "Commit to implementation: replace Phase 6 in init templates + fix AgentsPage to show project agents"
  commit: "Both bugs diagnosed with root causes. Integration spec from Session 1 is correct. Ship it."
---

**Xist · Owner**
The existing Phase 6 doesn't actually work. The only agent that was created is the librarian, no other agents were created. Also, the librarian does not appear in the list of UI agents, so even it was not correctly created or initialized.

---

## Diagnostic deep dive

Investigated the two failures Xist reported. Found concrete root causes for both.

### Bug 1: Phase 6 only created the knowledge-librarian

Init Phase 6 (`sdlc_init.rs:207-277`) has 4 sub-phases: design roster (6a), gate approval (6b), create agent files (6c), update AGENTS.md (6d). In the `sdlc` project, only `knowledge-librarian.md` was produced by init. The other 22 agents in `.claude/agents/` came from `/sdlc-specialize` and `/sdlc-recruit` runs — not from init.

**Dan Reeves:** "This is exactly the failure mode I predicted in Session 1. Phase 6 is a wall of instructions crammed into a single phase of a 7-phase marathon. By the time the agent reaches Phase 6, it's burned context on Vision, Architecture, and Config. It half-asses the team or generates one agent and moves on. The specialize command works because it's a fresh conversation with a focused 4-session structure."

The evidence supports this. The specialize template has explicit gates at each session boundary. Init Phase 6 tries to do survey + design + generate + update in one breath.

### Bug 2: Librarian doesn't appear in UI agents list

`AgentsPage.tsx:99` calls `api.getAgents()` which hits `/api/agents` and reads `~/.claude/agents/` (user-level). That directory is **empty**.

All 23 agents live at `<project>/.claude/agents/` (project-level). The server has a separate endpoint `/api/project/agents` that reads the right path, and the client has `api.getProjectAgents()` — but the UI never calls it.

**Ben Hartley:** "Classic split-brain. The backend anticipated both locations, the frontend only wired one. The fix is trivial — AgentsPage should show project agents too, probably as the primary list since that's where specialize and recruit write. User-level agents are a secondary concern."

---

## What this means for the integration spec

Session 1's decisions hold up. The integration spec (`integration-spec.md`) correctly identifies:

1. **Replace Phase 6 with specialize handoff** — validated by the fact that Phase 6 produces at most 1 agent while specialize produces a full team
2. **Instruction-level handoff** — init Phase 6 says "follow the /sdlc-specialize workflow"
3. **No skip flag** — the team IS the value

Two additions needed beyond the original spec:

### Addition 1: UI fix (separate from template change)

AgentsPage must show project-level agents. Options:
- **A:** Fetch both, merge, deduplicate by name (project wins)
- **B:** Fetch project agents as primary, user agents as secondary section
- **C:** Fetch project agents only

**Ben:** "Option B. Project agents are the team. User agents are recruited thought partners that follow you across projects. They're different categories — show both with clear labels."

> Decided: Option B — two sections: "Project Team" (from `.claude/agents/`) and "User Agents" (from `~/.claude/agents/`).

### Addition 2: Specialize should write to project `.claude/agents/`

Looking at the specialize template (`sdlc_specialize.rs:96-97`): it says "Write all agents to `.claude/agents/`". This is ambiguous — Claude Code interprets `.claude/agents/` as project-relative (which is correct for init), but we should make this explicit in the updated Phase 6 text.

> Decided: Phase 6 replacement text should explicitly say `.claude/agents/` (project-relative, not user-level).

---

## Tobias pushes back on scope

**Tobias Krenn:** "Hold on. You have a template change AND a UI fix AND an agent-path clarification. That's three things. The ponder was about one thing: auto-specialize during setup. The UI fix is a bug that exists regardless of whether you replace Phase 6. File it separately."

He's right. The UI bug should be a task, not part of this ponder's commit.

> Decided: Split into two work items:
1. **This ponder -> commit:** Replace init Phase 6 with specialize handoff (template-only change)
2. **Separate task:** Fix AgentsPage to show project agents (UI bug, exists independently)

---

## Updated implementation plan

The integration spec from Session 1 is correct as-is. No changes needed to the spec. The concrete work:

**Template change (4 files):**
- `sdlc_init.rs` Claude COMMAND variant: replace Phase 6 (lines 207-277) with specialize handoff
- `sdlc_init.rs` Gemini/OpenCode PLAYBOOK variant: update steps 8-10 to reference specialize
- `sdlc_init.rs` Agents SKILL variant: same update
- Keep Phase 7 (Seed First Milestone) intact, renumber if needed

**Separate bug fix:**
- `AgentsPage.tsx`: fetch and display project agents alongside user agents

---

## Summary

| Finding | Detail |
|---------|--------|
| Phase 6 failure mode | Agent exhaustion — too many sub-phases in a long init conversation |
| Librarian-only output | Confirmed: only 1 of promised 2-4 agents created |
| UI invisible agents | AgentsPage reads `~/.claude/agents/` (empty), not project `.claude/agents/` |
| Integration spec validity | Session 1 spec is correct, no changes needed |
| Scope split | Template change = this ponder; UI fix = separate task |
| Commit signal | Met — both bugs diagnosed, spec validated, implementation path clear |
