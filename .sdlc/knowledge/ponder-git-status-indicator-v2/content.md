---
session: 1
timestamp: 2026-03-07T23:35:00Z
orientation:
  current: "Two clear problems identified and designed — broken commit button, missing hover detail tile. Scope is tight (2-3 features). Ready to commit."
  next: "Run /sdlc-ponder-commit git-status-indicator-v2 to crystallize into milestone + features"
  commit: "Met — owner's brief is specific and actionable, code audit confirms feasibility, designs are concrete"
---

**Xist · Owner**
git-status-indicator-v2

followup to milestones/git-status-indicator

when mousing over the git info, or on touch on mobile, open a git details hover tile showing more detail about the state of the git repo and what the user needs to do if anything.

when clicking the commit button, it doesn't seem to do anything. what is supposed to happen?  i thought it would spawn an agent executing the /sdlc-commit skill and showing me agent activity.

---

**Facilitator**

Two clear asks here. Let me audit the code before we discuss.

### Finding 1: The commit button is broken

The `GitStatusChip.tsx` (line 30-45) has a `handleCommit` that POSTs to `/api/git/commit`. But looking at `lib.rs`, only one git route is registered:

```
.route("/api/git/status", get(routes::git::get_git_status))
```

There is **no `/api/git/commit` endpoint**. The button silently fails — the error is swallowed by `console.warn`. This is a real bug from v1 that shipped with the UI but no backend.

### Finding 2: Rich data, tiny display

The API already returns `dirty_count`, `staged_count`, `untracked_count`, `ahead`, `behind`, `has_conflicts`, `conflict_count`, `severity`, and `summary`. But the chip compresses all of this into a single line like "main — 6 modified". Most of the data is invisible.

---

**Facilitator (channeling a UX perspective)**

The owner's instinct is right — a hover tile is the correct pattern here. The chip is a glanceable indicator (traffic light), and the tile is the "what does this mean?" layer. This is a well-established progressive disclosure pattern: indicator → detail → action.

The key insight is **actionable guidance**. Don't just show counts — tell the user what to do:
- Conflicts? "Resolve merge conflicts before anything else"
- Behind? "Pull upstream changes"
- Staged? "Ready to commit" + commit button
- Dirty but nothing staged? "Stage your changes first"
- Clean? Show the GitGreenQuote (already exists!)

This guidance logic is pure frontend — derived from the API fields, no new backend needed.

---

**Facilitator (channeling an architecture perspective)**

For the commit button, the answer is clear: `spawn_agent_run`. This is the server's standard pattern for agent-driven work. The commit skill (`/sdlc-commit`) does intelligent staging, message generation, fetch, and history reconciliation — much more than a bare `git commit`.

The flow:
1. POST `/api/git/commit` → `spawn_agent_run` with `/sdlc-commit` prompt
2. Return `{ run_id }` immediately
3. Frontend navigates to run detail (or shows inline activity)
4. SSE streams agent progress
5. On completion, git status auto-refreshes

This matches every other agent-driven endpoint in the codebase (advisory, knowledge, tools, etc.).

---

**Facilitator**

### Tensions surfaced

**? Open: Should the hover tile show file names or just counts?**

Counts are clean and fast. But "6 modified files" is less useful than seeing which 6 files. The API currently doesn't return file names — adding them would need either extending the existing endpoint or a new `GET /api/git/files` endpoint. Proposal: start with counts only (uses existing API), add file names as an optional enhancement.

**? Open: Where does commit activity show?**

Three options: navigate to `/runs/:id` (works today), inline panel in the tile (richer, more work), or toast with link (lightweight). For v2, navigating to the run page is the pragmatic choice — it already works and shows SSE-streamed agent activity. An inline panel could be a v3 enhancement.

**⚑ Decided: Scope is two required features + one optional**

1. **git-commit-agent** — fix the broken button (highest priority, real user pain)
2. **git-details-hover-tile** — hover/touch popover with structured detail and guidance
3. **git-status-files-api** — optional enhancement to show file names in the tile

### Commit signal

This idea is ready to commit. The owner's brief is specific and actionable. The code audit confirms both problems and the feasibility of both solutions. The designs are concrete and follow established patterns (spawn_agent_run, progressive disclosure). No open questions block implementation.

**Next:** `/sdlc-ponder-commit git-status-indicator-v2`
