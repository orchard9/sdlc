---
session: 1
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Core design decided — librarian is a project-generated agent with two modes (maintain + query) and a harvest hook that forces maintenance on investigation completion"
  next: "Refine the spec: add librarian to the knowledge base spec, define the librarian init flow and the harvest hook protocol in detail"
  commit: "When the updated spec covers librarian init, harvest hooks, query mode CLI, and updated entry data model — ready to commit to a feature"
---

## Session 1: Knowledge Librarian — Collaborative Design

**Context:** Continuing from a knowledge base spec discussion where the user surfaced the insight
that the knowledge base should be *constantly maintained by a librarian template*. The spec I
had written described passive storage; this session explores what active custodianship means.

---

**Recruited team:**
- Recruited: Yuki Tanaka · Information Architect & PKM Systems Designer
- Recruited: Remi Okonkwo · Senior Engineer & Institutional Memory Advocate
- Recruited: Lars Andersen · Engineering Manager & Knowledge System Skeptic

---

**Yuki Tanaka · Information Architect**

Challenged the 10-class Dewey-inspired taxonomy in the original spec. Classic library science
mistake — imposing taxonomy before understanding the knowledge shape. The librarian's primary
job is sense-making, not filing.

Key insight: taxonomy should emerge from accumulated knowledge, not be designed upfront. The
librarian watches what accumulates and proposes new catalog categories when clusters form.

⚑ Decided: Catalog is emergent. Default catalog ships sparse (5–7 categories). Librarian proposes
new categories and merges thin ones. The initial catalog is generated from the project's own
domain signals.

---

**Remi Okonkwo · Institutional Memory Advocate**

Surfaced the browse vs. query distinction. The spec's UI is a taxonomy browser (archivist interface).
What developers need is query mode: `sdlc knowledge ask "why hexagonal architecture here?"` — the
librarian synthesizes from multiple entries and cites sources.

Side effect: when a query hits a gap in the knowledge base, the librarian initiates a research run
to fill it. Query becomes acquisition.

⚑ Decided: Two modes — maintain (scheduled/hooks, curates) and query (on-demand, answers questions,
triggers research for gaps). Same agent template, different invocation context.

? Open: Is query mode a CLI command or a chat UI panel? Probably both — CLI for mid-flow,
UI for exploratory.

---

**Lars Andersen · Engineering Manager & Skeptic**

Named the death spiral: nobody queries because it's stale; nobody updates because nobody queries.
Maintenance has to be structurally forced, not voluntary.

The harvest hook: when any investigation reaches `done`, a hook fires to extract durable insights
into the knowledge base. The investigation's completion becomes the forcing function.

⚑ Decided: Librarian runs on schedule (weekly) AND on workspace-completion hooks. Maintenance is
never voluntary.

? Open: Full autonomous write authority vs. propose-and-review? Project ethos ("autonomous by
default", "git is the undo button") says autonomous writes.

---

**Synthesis**

The knowledge base is a library with a *living librarian* — a project-generated agent that:
1. Classifies incoming entries (auto-code on ingest)
2. Curates the catalog (proposes categories, merges thin ones)
3. Cross-references (links related entries, surfaces clusters)
4. Freshens (detects stale URLs, dead code refs, aged-out entries)
5. Harvests (extracts knowledge from completed investigations automatically)
6. Answers (responds to queries, initiates research when gaps found)

`sdlc knowledge librarian init` generates the agent tailored to this project — scanning
VISION.md, ARCHITECTURE.md, code, and .sdlc/ state to derive the initial catalog and
bake project context into the agent definition.

This is the knowledge member of the team that `sdlc-specialize` would generate.

---

**Artifacts captured:** synthesis.md
