---
session: 1
timestamp: 2026-03-06T23:05:00Z
orientation:
  current: "Shaped the integration approach — specialize replaces init Phase 6, runs after architecture approval"
  next: "Write the implementation spec: exact changes to sdlc_init.rs templates across all 4 platforms"
  commit: "Agreement on which init phases to cut/replace + the handoff mechanism between init and specialize"
---

**Xist · Owner**
auto-specialize-during-setup

when initializing ponder sdlc in an existing project add `/sdlc-specialize` step after the ARCHITECTURE is worked out. suggest agent specialists and coordinate with the user to tune them. activate the resulting specialists in the project.

---

## The Problem

Looking at the current `/sdlc-init` flow:

| Phase | What |
|-------|------|
| 1 | Orient (read existing files) |
| 2 | Quick Capture (interview) |
| 3 | Vision (draft + gate) |
| 4 | Architecture (draft + gate) |
| 5 | Config (patch config.yaml) |
| **6** | **Team (design 2-4 agents + gate)** |
| 7 | Seed First Milestone |

Phase 6 creates 2-4 lightweight agents. Meanwhile, `/sdlc-specialize` is a separate command that does the same job but deeper: 4-session flow, surveys the actual codebase, generates both agents AND skills, uses real file paths. The two commands overlap — init creates a basic team, specialize creates a thorough team.

The proposal: after init Phase 4 (Architecture approved), invoke specialize automatically. This means init's Phase 6 becomes redundant.

---

## Dan Reeves pushes back

**Dan:** "Hold on. You have two commands that create agents. That's already a smell. The question isn't 'how do we wire them together' — it's 'why do we have two?'"

He's right to ask. The real options are:

1. **Replace init Phase 6 with a specialize call** — init handles Vision + Architecture + Config, then delegates team creation to specialize
2. **Merge specialize INTO init** — make init's Phase 6 as thorough as specialize
3. **Keep both, wire sequentially** — init creates basic agents, specialize upgrades later

Option 3 is the worst — you'd overwrite agents, confuse the user about which team is "real", and double the setup time.

**Dan:** "Option 1. Init already gates at Architecture. Specialize already surveys the project. Let init do what it's good at (interview, vision, architecture) and specialize do what it's good at (survey codebase, design team with real paths). Clean boundary."

⚑ Decided: Option 1 — replace init Phase 6 with a specialize handoff.

---

## Maya Goldberg on the UX seam

**Maya:** "I care about the transition. Right now init is one continuous conversation with the user. If you suddenly say 'now I'm running specialize', the user doesn't know what changed. The flow should feel like one continuous setup, not two tools stitched together."

This is a real tension. Two approaches:

**A. Inline the specialize logic** — init Phase 6 literally becomes the specialize flow (survey, roster gate, generate). No separate command invocation. The init template absorbs the specialize template's logic.

**B. Explicit handoff** — init finishes Phase 5 (Config), says "Now let's set up your AI team", and the specialize flow runs as a distinct phase. Same conversation, but the logic lives in a separate template.

**Maya:** "Option A feels seamless but makes the init template enormous. Option B is fine IF the handoff is invisible — don't say 'now running /sdlc-specialize', just say 'Now let's design your team.'"

**Dan:** "Option B. Keep the templates separate. The specialize template already works standalone — people who skip init should still be able to run it. Just call it from init's instructions."

? Open: How does init "call" specialize? Three mechanisms:
1. **Instruction-level**: init's Phase 6 says "Follow the /sdlc-specialize flow now" — the agent reads both templates in sequence
2. **Next command**: init finishes at Phase 5, outputs `**Next:** /sdlc-specialize` — user triggers the next step
3. **Programmatic**: init template literally says "invoke /sdlc-specialize" as a tool call

Option 2 (Next command) breaks the "one continuous setup" feel. Option 3 doesn't exist in the slash command system. Option 1 is the only real choice — the init template's Phase 6 becomes "execute the specialize flow inline."

⚑ Decided: Init Phase 6 instructions will say "Follow the /sdlc-specialize workflow" with a brief inline summary. The agent reads both templates and executes specialize's survey + roster + generate flow as init's team phase.

---

## Xist's perspective — what does a new user experience?

**Xist:** "When I ran init, I answered questions about my project. It wrote Vision, Architecture. Then it suggested some agents. I didn't really understand what agents were for — I just said 'sure'. The agent names were generic. They didn't know my codebase."

This is exactly the gap. Init's current Phase 6 designs agents from the *interview* — what the user said. Specialize designs agents from the *codebase* — what actually exists. After Architecture is written, the agent has enough context to survey real files.

**Xist:** "If it had said 'I see you have a Rust workspace with these crates, a React frontend, and a server — here's a team that covers those areas', I would have understood what agents do. The agents would have been about MY code, not abstract roles."

⚑ Decided: The specialize step must happen AFTER Architecture is written (not just approved) — it needs the Architecture document as input alongside the actual codebase survey.

---

## What changes concretely

### In `sdlc_init.rs` (all 4 platform variants):

**Remove:**
- Phase 6 (Team) entirely — the roster design, gate, agent generation, AGENTS.md update

**Replace with:**
- Phase 6: "Specialize — AI Team" that says:

> Now that Vision and Architecture are in place, survey the codebase and design a tailored AI team.
> Follow the `/sdlc-specialize` workflow: survey the project (it will read the Vision and Architecture you just wrote), design a roster of 3-5 specialists matched to the actual codebase structure, gate the roster with the user, generate agent files and skills, and update AGENTS.md.

**Keep intact:**
- Phases 1-5 (Orient, Quick Capture, Vision, Architecture, Config)
- Phase 7 (Seed First Milestone) — renumber to Phase 7 after the new Phase 6

### In `sdlc_specialize.rs`:

**No changes needed.** It already:
- Surveys project files (VISION.md, ARCHITECTURE.md, source dirs)
- Runs `sdlc state`
- Gates at each stage
- Generates agents + skills with real file paths

The specialize template is already designed to work post-architecture. It just needs to be called at the right time.

---

## Edge cases

**What if init is run on a greenfield project with no source code?**
Specialize surveys source dirs. If there are none, it falls back to designing agents from Vision + Architecture alone. This is fine — the agents won't have `This Codebase` sections with real paths, but they'll have domain-appropriate roles.

**What about the existing Phase 6 agents?**
People who already ran init have agents. Running specialize later would create new agents alongside old ones. The specialize flow should check for existing `.claude/agents/` and either update or replace.

? Open: Should specialize detect existing agents and offer to replace/merge? Or always start fresh?

**Dan:** "Start fresh. The init-generated agents were placeholders. The specialize agents are the real ones. Don't merge — that's complexity for zero value."

⚑ Decided: Specialize overwrites. Init's Phase 6 note should say "this replaces any previously generated agents."

---

## Blast radius

The change touches:
1. `sdlc_init.rs` — all 4 template variants (COMMAND, PLAYBOOK, SKILL + the Gemini/OpenCode variants)
2. Nothing in Rust logic — this is purely template text
3. CLAUDE.md command table — update the `/sdlc-init` description to mention specialize

Low risk. Template-only change. No state machine changes. No new CLI commands.

---

## Summary of decisions

| Item | Decision |
|------|----------|
| Integration approach | Replace init Phase 6 with specialize handoff |
| Handoff mechanism | Instruction-level — init says "follow the /sdlc-specialize workflow" |
| Timing | After Architecture is written to disk (post Phase 4) |
| Existing agents | Specialize overwrites, no merge |
| Specialize changes | None — it already works post-architecture |
| Blast radius | Template text only, 4 platform variants |

? Open: Should we add a "skip specialize" escape hatch for users who want fast init without the team design? E.g., a flag or a "skip team setup?" gate.

**Maya:** "No. The team IS the value. If you skip it, you get a tool with no specialists. That's like installing an IDE without extensions. Make it fast, don't make it skippable."

**Dan:** "Agree. If someone wants to skip, they can just Ctrl+C. Don't add a flag for a hypothetical."

⚑ Decided: No skip flag. Keep it streamlined.
