# UAT Run — v12-knowledge-ui
**Date:** 2026-03-02T09:58:00Z
**Verdict:** PassWithTasks
**Tests:** 13/18 passed (5 code bugs → tasks)
**Tasks created:** knowledge-page-ui#T13, knowledge-page-ui#T14, knowledge-page-ui#T15, knowledge-page-ui#T16, knowledge-page-ui#T17

## Results
Suite: v12-knowledge-ui Acceptance Tests (Mode B — generated from checklist)
Agent: claude-sonnet-4-6
Passed: 13 | Failed: 5 | Skipped: 0

## Failures

| Test | Classification | Resolution |
|---|---|---|
| Search bar in left pane filters entries | Code bug — no search input in KnowledgePage.tsx | Task knowledge-page-ui#T13 created |
| Center pane shows staleness badges on entries | Code bug — KnowledgeEntrySummary type lacks staleness_flags | Task knowledge-page-ui#T14 created |
| Right pane renders content as Markdown | Code bug — content shown in `<pre>` raw text, not react-markdown | Task knowledge-page-ui#T15 created |
| Related entries are clickable links | Code bug — related entries rendered as `<span>` not `<button>`/`<a>` | Task knowledge-page-ui#T16 created |
| Source provenance shows captured_at | Code bug — captured_at timestamp not rendered in sources section | Task knowledge-page-ui#T17 created |

## Passing Tests
1. Knowledge page loads at `/knowledge`
2. Sidebar shows Knowledge link with Library icon in plan group
3. Left pane catalog tree with expandable class buttons (100–600)
4. Clicking a catalog class filters center pane to entries in that class
5. Clicking an entry opens detail in right pane (URL changes to /knowledge/:slug)
6. Source provenance shows type and url/path (web link clickable) — partial pass
7. Research More button triggers visible agent run in agent panel
8. Advisory route generates `## Project Knowledge` section (verified from code)
9. Advisory instructs agent to cite knowledge slugs as `[kb: <slug>]` (verified from code)
10. `/sdlc-knowledge` slash command exists at `~/.claude/commands/sdlc-knowledge.md`
11. Slash command supports catalog overview, query/synthesis, init, research, maintain modes
12. GUIDANCE_MD_CONTENT includes `sdlc knowledge *` command table
13. `SDLC_NO_NPM=1 cargo test --all` — passed (exit code 0)
14. `SDLC_NO_NPM=1 cargo clippy --all -- -D warnings` — passed (exit code 0)
