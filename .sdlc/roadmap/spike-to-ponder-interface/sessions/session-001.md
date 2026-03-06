---
session: 1
timestamp: 2026-03-04T17:40:00Z
orientation:
  current: "Shape is clear — Spikes page (list+detail+promote) + CLI promote command + minimal Rust data layer"
  next: "Decide: is the idea converging toward commitment, or are there open questions to resolve first? If committing, run /sdlc-ponder-commit spike-to-ponder-interface"
  commit: "Jordan confirms the shape is right and V1 scope (read-only UI + promote CLI) is acceptable"
---

## Session 1 — Exploring the spike-to-ponder interface

**User brief:** "i want an interface to use the template for 'spike' - that can be used to turn a spike into a ponder"

### Landscape audit

Checked existing codebase:
- `/sdlc-spike` command exists in `crates/sdlc-cli/src/cmd/init/commands/sdlc_spike.rs` — full 5-phase workflow
- `.sdlc/spikes/` directory exists in the project but has no contents yet
- No REST routes for spikes in `crates/sdlc-server/src/routes/`
- No Rust data layer for spikes in `sdlc-core`
- No UI for spikes in `frontend/src/pages/`
- The spike command currently ends with text: "Next: /sdlc-ponder <slug>" for ADAPT/REJECT verdicts — but no actual conversion mechanism

⚑ Decided: The core gap is two things — **visibility** (can't see your spikes) and **handoff** (no seeding when converting to ponder).

### Thought partner voices

**Reza Patel · Developer Workflow Architect**
The handoff between investigation mode and commitment mode is high-friction. The design question is how much structure the spike needs before conversion. Lightest thing: `sdlc spike promote <slug>` CLI command that seeds a ponder. The spike's "Risks and Open Questions" section becomes the ponder's starting territory, not the full findings.

**Mara Osei · Product Builder**
When verdict is ADAPT, I need to understand *why* we're adapting and what the ponder should explore. "Risks and Open Questions" is the right fuel for the ponder — that's what the spike couldn't resolve. The validated things (what worked) are just background context.

**Dan Wick · Pragmatic Skeptic**
? Is the conversion actually hard, or just unfamiliar? The real problem I see is *discovery* — spikes disappear. Build listing first. But the promotion adds value too: it seeds the ponder with structured context, preventing information loss.

### Design produced

Three-view UI (mockup captured as spike-interface-mockup.html):

1. **Spikes List** — Sidebar in "Explore" group. Verdict badges. ADAPT/REJECT rows show "promote to ponder →" affordance.
2. **Spike Detail** — findings.md rendered with structured sections (The Question / Success Criteria / Candidates table / Risks). "Promote to Ponder" button (hidden for ADOPT).
3. **Promote to Ponder** — Shows what gets carried: spike-findings.md, open-questions.md, ponder title. Editable ponder slug. CLI equivalent hint.

Shape captured in design-shape.md.

### Key decisions

⚑ ADOPT spikes don't show "Promote to Ponder" — ADOPT → /sdlc-hypothetical-planning, not pondering.

⚑ "Risks and Open Questions" section is the ponder's fuel — extracted separately as open-questions.md.

⚑ Prototype code stays ephemeral — not carried to ponder, stays in /tmp.

⚑ V1 scope: read-only UI + promote action. No web-triggered spike runs, no spike sessions/chat.

? Should `sdlc spike promote` update the spike's manifest to record the ponder slug? (Traceability vs. complexity.)

### Implementation shape

**Rust**: `sdlc-core/src/spikes.rs` — `list()`, `load_findings()`, `promote_to_ponder()` (dumb data, no decisions)
**CLI**: `sdlc spike list | show | promote`
**REST**: `GET /api/spikes`, `GET /api/spikes/:slug`, `POST /api/spikes/:slug/promote`
**UI**: `SpikePage.tsx` — sidebar in Explore group

## Product Summary

### What we explored
How to surface spikes in the UI and create a structured handoff from spike findings to a ponder workspace. The current spike command produces findings in a folder that nothing reads.

### Key shifts
Before this session: spikes had no UI, no data layer, and the spike→ponder conversion was just a text suggestion with no mechanism. After: a clear three-layer design (Rust data, REST, UI) with a specific `promote` action that seeds the ponder with spike-findings.md and open-questions.md extracted from findings.

### Implications
This makes spikes a first-class artifact in the roadmap — discoverable from the UI, not just files in a folder. The "Promote to Ponder" action closes the loop between fast investigation and deeper exploration, preventing the institutional memory loss that happens when spike findings aren't carried forward.

### Still open
1. Should REJECT spikes also show "Promote to Ponder" — or is a rejected approach dead and the need should start fresh in ponder?
2. Should `sdlc spike promote` record the ponder slug back in the spike's manifest for traceability?
