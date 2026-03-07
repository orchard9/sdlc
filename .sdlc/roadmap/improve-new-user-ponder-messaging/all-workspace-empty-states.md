## All Workspace Pages — Rich Empty States

Implemented rich onboarding empty states for all remaining workspace pages, matching the pattern established for Ponder in session 1.

### Pages updated

| Page | Hero tagline | Steps | Extra section | CTA |
|------|-------------|-------|---------------|-----|
| Root Cause | "Find the root cause." | 4 (Symptoms → Investigate → Synthesis → Action) | — | New Root Cause button |
| Guidelines | "Codify what works." | 4 (Evidence → Principles → Draft → Publish) | — | New Guideline button |
| Spikes | "Answer one question fast." | 3 (Question → Investigate → Verdict) | Verdict strip (ADOPT/ADAPT/REJECT) | CLI command display |
| Knowledge | "What the team knows." | 3 (Catalog → Research → Staleness) | — | CLI command display |
| Evolve | (already had onboarding) | 5 phases | — | Start an Evolution button |
| Ponder | (session 1) | 4 steps | Lifecycle strip | Suggest + New idea |

### Mobile behavior

All pages use responsive breakpoints correctly:
- **WorkspaceShell pages** (Ponder, Root Cause, Evolve, Guidelines): List pane takes full screen on mobile; detail pane (with onboarding) shows at `lg:` breakpoint only
- **Custom layout pages** (Spikes, Knowledge): List pane(s) visible on mobile; detail pane hidden until item selected

This is the correct UX — on mobile, users see the list and tap to navigate. The onboarding content fills the otherwise-empty detail pane on tablet/desktop where both panes are visible.

### Design consistency

All empty states follow the same structure:
1. Icon badge (colored bg circle)
2. Tagline heading (`text-xl font-semibold`)
3. Description paragraph
4. "HOW IT WORKS" section header
5. Step cards with icons (border cards, `bg-card/50`)
6. Optional extra section (lifecycle, verdicts)
7. CTA (button or CLI command)
8. Contextual hint when entries exist