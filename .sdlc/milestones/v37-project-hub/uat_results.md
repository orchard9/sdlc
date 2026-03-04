# UAT Run — v37-project-hub
**Date:** 2026-03-04T22:05:00Z
**Agent:** claude-sonnet-4-6
**Verdict:** PASS WITH TASKS

---

- [x] Within 35s of starting sdlc serve, project cards appear in hub listing _(2026-03-04T22:05:00Z)_
- [x] Each card shows the correct project name _(2026-03-04T22:05:00Z)_
- [x] Each card shows the correct URL _(2026-03-04T22:05:00Z)_
- [x] Green status dot on both cards _(2026-03-04T22:05:00Z)_
- [x] Typing in the filter box narrows cards in real time _(2026-03-04T22:05:00Z)_
- [x] Filter matches on project name (partial match, case-insensitive) _(2026-03-04T22:05:00Z)_
- [x] Count shown updates as filter changes ("2 projects" → "1 of 2 projects") _(2026-03-04T22:05:00Z)_
- [x] Clearing filter restores all cards _(2026-03-04T22:05:00Z)_
- [x] Active milestone slug appears on the card _(2026-03-04T22:05:00Z)_
- [x] "agent running" badge appears when agent_running: true _(2026-03-04T22:05:00Z)_
- [x] Badge absent when agent_running: false _(2026-03-04T22:05:00Z)_
- [x] Empty state renders with hub config hint — no errors in console _(2026-03-04T22:05:00Z)_
- [x] Hub API returns valid JSON when no projects registered _(2026-03-04T22:05:00Z)_
- [ ] ~~Browser tab for hub shows "sdlc hub"~~ _(✗ task page-title-fix#T4 — embed.rs compute_title() uses project state.yaml in hub mode, returns "sdlc — sdlc")_

---

**Tasks created:** page-title-fix#T4
**13/14 steps passed**
