---
session: 1
timestamp: 2026-03-07T20:15:00Z
orientation:
  current: "Idea fully shaped — three milestones defined, API contract drafted, tech stack chosen, edge cases mapped"
  next: "Commit to milestones via /sdlc-ponder-commit git-good"
  commit: "All key decisions resolved. Commit signal met — ready to build."
---

**Xist · Owner**
Git good

ponder expects that its host project is itself a git repository. the git repo can be in some different basic states that should be clearly identified in the ui:

1. no pending changes and fully sync'd with origin, everything is clean/green
2. local changes are pending, not yet committed
3. local commits have not been pushed
4. origin commits have been pushed that we have not yet pulled

We need a multi-part project here. Initially, we want high level status button that is active when there are changes pending or commits pending for push, and clicking the button runs `/sdlc-commit`.  When clean/green show something fun instead of the button in that case.

The longer term project is we need a new section in the UI like Ponder, Root Cause, Evolve, etc called Git.  Put it in the Integrate area above Network. The Git management UI shows a file browser column that visualizes all the files in the workspace, allowing for filters of all files, only modified files, and other useful filters. The view should be easily togglable from flat view with full paths to tree view with hierarchical representations of the files.

Clicking on any given file opens in the main content view the diff viewer.  This is a side-by-side diff view if the viewport is wide enough, otherwise it's a patch-style chunk diff view. Use colors to accentuate diffs of actual content compared to whitespace.

---

## Team

- **Maya Chen** — Frontend Architect (dev tools at GitHub/VS Code, responsive layouts, accessibility)
- **Kai Müller** — Git Internals Expert (plumbing, edge cases, pragmatic signal-vs-noise)
- **Priya Sharma** — Daily User / IC Developer (wants effortless common path, hates information overload)

---

## Discussion

### Git Status Model

**Kai:** The four states in the brief aren't mutually exclusive. A repo can have uncommitted changes AND unpushed commits AND upstream commits — all at once. Proposed a composite struct with independent boolean/numeric fields instead of an enum.

**Maya:** Collapsed the composite model into a traffic-light severity for UI: Green (clean), Yellow (local work pending), Red (diverged/conflict). Status chip belongs in the sidebar header — visible from every screen.

**Priya:** The 80% use case is "glance and know am I clean?" One click to commit if not. The status indicator with file count ("3 files changed") is the perfect information density.

⚑ Decided: Composite state model — fields are independent, not a single enum
⚑ Decided: Green/Yellow/Red severity computed server-side
⚑ Decided: Status chip in sidebar header for global visibility

### File Browser Design

**Maya:** Build the file browser in-house rather than pulling a tree view library. Tree/flat toggle is trivial (split path, build tree client-side). Need tight control over git status badges and our Tailwind design system.

**Priya:** "Modified only" must be the default filter. When about to commit, nobody cares about 500 unchanged files — just the 3 you touched.

**Kai:** Provided edge case grid: binary files, large diffs, renames, permission changes, submodules, detached HEAD, no remote, shallow clones. Each needs explicit handling.

⚑ Decided: Custom file browser component (no library)
⚑ Decided: Modified-only as default filter
⚑ Decided: Fixed 300px panel width initially (resizable later)

### Diff Viewer

**Maya:** Recommended `@git-diff-view/react` — actively maintained, GitHub-style UI, split/unified modes, accepts standard unified diff format, syntax highlighting built-in, ~35KB gzipped.

**Priya:** Responsive breakpoint at 900px content width for side-by-side vs unified. Side-by-side needs at least 80 chars visible per side to be useful.

**Kai:** Backend must handle untracked files (synthetic all-addition diffs), binary files ("binary changed" with size delta), and truncation for large diffs (>2000 lines).

⚑ Decided: `@git-diff-view/react` for diff rendering
⚑ Decided: 900px breakpoint for split↔unified toggle
⚑ Decided: Keyboard shortcuts for power users (j/k, [/], f, m, a)

### Staging UI

**Maya:** View-only for now — no `git add/reset` in the UI. The ponder philosophy is "commit and move on." Agents handle git operations via `/sdlc-commit`. Adding staging UI introduces complexity that fights the autonomous ethos.

**Priya:** Agreed — "I don't want to stage individual files in a web UI. The diff viewer IS the review step. The commit button IS the action."

⚑ Decided: No staging UI — view-only. `/sdlc-commit` handles the action.

### API Contract

Three endpoints designed:
- `GET /api/git/status` — composite status with severity, polled every 5s
- `GET /api/git/files` — file list with status badges for file browser
- `GET /api/git/diff?path=<file>` — single file unified diff

All backed by `git` CLI via `std::process::Command` (no libgit2).

⚑ Decided: git CLI over libgit2 — always available in our environments

### Milestones

1. **Git Status Indicator** (small) — sidebar status chip + commit button
2. **Git Page — File Browser** (medium) — new section, file list with filters/tree toggle
3. **Git Page — Diff Viewer** (medium) — responsive diff rendering with `@git-diff-view/react`

---

## Open Questions

? Open: What "fun" thing to show for green/clean state — subtle animation? rotating messages?
? Open: Should we add `GET /api/git/log` for commit history in a future milestone?
? Open: Whitespace diff coloring — exact color palette for content vs whitespace changes

## Artifacts Captured

- `git-status-model.md` — composite state model and severity mapping
- `milestone-breakdown.md` — three milestones with scope and complexity
- `technical-decisions.md` — library choices and rationale
- `edge-cases-ux.md` — edge case grid, keyboard shortcuts, responsive breakpoints
- `api-contract.md` — full API endpoint definitions with examples
