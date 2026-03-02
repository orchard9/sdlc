---
session: 3
timestamp: 2026-03-02T09:06:00Z
orientation:
  current: "Design is complete. All open questions resolved. Jordan has issued the commit signal."
  next: "Run /sdlc-ponder-commit rethink-the-dashboard to crystallize into milestones and features."
  commit: "DONE — commit signal issued by jordan."
---

**jordan · Owner**
wave execution can happen in the milestone board, but the orchestrator will typically
3 is fine

commit it

---

**Facilitator**

Three signals in one message. Each one closes an open question from session 2.

---

**"wave execution can happen in the milestone board"**

This resolves the open question from session 2: *"who triggers wave execution in full orchestrator mode — determines whether State A banner has a button."*

⚑  **Decided:** The "Run wave" / wave execution trigger UI belongs on the **milestone board page**, not the dashboard. The dashboard's State A amber banner ("Orchestrator idle — N milestones ready") is **status-only** — no execution button. The banner links to the milestone board where wave controls live.

This is architecturally clean: the dashboard is a digest surface, not a control panel. The milestone board is the action surface. Dashboard says "something's ready"; milestone board lets you act.

---

**"3 is fine"**

This resolves the second open question: *"how many exploring ponders to show before truncating."*

⚑  **Decided:** Cap the "Exploring" subgroup in Horizon at **3 items**. A 4th+ item gets a "View all →" link to `/ponder`. No cap on "Ready to Commit" (converging) ponders — those are action items the human must handle, not passive status.

---

**"commit it"**

The commit signal has been met. All design decisions are in:

| Decision | Session |
|---|---|
| Mental model: digest, not control panel | 1 |
| Zone order: Attention → Shipped → In Flight → Horizon | 1 |
| No feature cards on dashboard (drill into milestone) | 1 |
| No velocity estimates on In Flight | 2 |
| Horizon includes ponders (not just milestones) | 2 |
| Empty state uses /sdlc-suggest output | 2 |
| Ideation Strip eliminated → merged into Horizon header | 2 |
| Horizon subgroups: Queued / Ready to Commit / Exploring | 2 |
| Parked ponders excluded from Horizon | 2 |
| State A banner: status-only, link to milestone board | 3 |
| Wave execution UI lives on milestone board | 3 |
| Exploring ponder cap in Horizon: 3 items | 3 |

**The ponder is ready to crystallize into a milestone.**

---

**Dana Cho · Product Skeptic**

One implementation note before we hand off: the milestone scope should be kept tight. Don't ship partial zones. The zones are load-bearing — "In Flight without Recently Shipped" would feel incomplete. Ship the full layout swap atomically: old dashboard out, new zones in.

If there's a risk this is too large for one wave, split by zone into tasks — but keep it one milestone. The design is cohesive.

⚑  **Decided:** One milestone. Atomic dashboard replacement. Feature cards and CommandBlocks removed as part of the same ship.
