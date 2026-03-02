---
session: 2
timestamp: 2026-03-01T01:00:00Z
orientation:
  current: "Complete spec written — librarian init, harvest protocol, query mode, maintenance loop, integration surface, phased build all defined"
  next: "Commit this ponder to features — the spec is ready"
  commit: "Done — spec is captured in knowledge-spec.md, ready for /sdlc-ponder-commit"
---

## Session 2: Spec Refinement + Open Questions

Continued from session 1. Resolved two open questions and developed the remaining design pieces.

---

**Resolved: Query mode**

⚑ Decided: CLI first (`sdlc knowledge ask "<question>"`), chat panel in Phase 3 alongside the
full KnowledgePage UI. Both serve real needs (mid-flow lookup vs. exploratory), but CLI is
cheaper and more urgent.

---

**Resolved: Write authority**

⚑ Decided: Librarian writes autonomously. All writes committed to git with "librarian:" prefix.
Audit trail in .sdlc/knowledge/maintenance-log.yaml. Lars was right — voluntary maintenance fails.

---

**The flywheel dynamic**

Named explicitly: the value is not linear, it compounds. Every investigation feeds the base.
Every entry makes the next investigation faster. Every gap query triggers research. Over 6 months
the knowledge base becomes the project's most valuable asset.

---

**Initial seeding problem — solved**

`sdlc knowledge librarian init` on an existing project doesn't start empty. It:
1. Harvests all completed investigations
2. Harvests all committed ponder entries
3. Creates linking entries for all published guidelines
4. Seeds foundational entries from VISION.md + ARCHITECTURE.md
5. Derives the initial catalog from the domain signals it finds

Day-one value on existing projects.

---

**Librarian init flow — detailed**

9-step flow defined: project scan → workspace harvest → guideline linking → domain extraction
→ catalog generation → agent file generation → hook registration → first maintenance pass → report.

The librarian agent file (.claude/agents/knowledge-librarian.md) gets the catalog + project
context injected at generation time. Not generic — tailored.

---

**Harvest protocol — defined**

Harvests durable insights only (not transient details). Checks for existing matching entries
before creating new ones. Appends to existing when a match is found. Records harvested_from.
Logs all actions to maintenance-log.yaml.

---

**Scope decision**

⚑ Decided: Project-level only for v1. User-level knowledge (cross-project patterns) is a
future phase. The value is in project-specific context.

---

**Integration surface mapped**

Six integration points: investigate complete → harvest, ponder commit → harvest,
guideline publish → harvest, advisory runs → knowledge as context,
sdlc-init → knowledge librarian init, sdlc-next → knowledge check step.

---

**Full spec captured in:** knowledge-spec.md

---

? Open for commit: 3 design questions flagged in spec that don't block the build:
1. sdlc-init: auto vs. opt-in for knowledge init
2. Advisory integration depth: tag match vs. semantic search
3. Cron management: config.yaml vs. CI system
