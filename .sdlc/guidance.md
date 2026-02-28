# Engineering Guidance

Read this before any implementation, bug fix, or test action.

## 1. Build It Right

Do it the proper way — not the quick way. The correct solution is one that
will still be correct in six months. Favor proven patterns, clear
abstractions, and designs that are easy to understand and extend. Never
trade long-term correctness for short-term convenience.

## 2. Understand Bugs Before Fixing Them

Before touching a bug, trace its root cause holistically — read surrounding
code, follow the data flow, understand why it broke. Fix the cause, not the
symptom. A patch that introduces a new bug in three months is worse than
no fix.

## 3. Enterprise Quality Bar

We build enterprise-grade software. The bar is Steve Jobs: relentless
attention to detail, nothing ships that embarrasses us, correctness and
reliability are non-negotiable. If something isn't right, make it right.

## 4. Philosophy of Software Design

Follow John Ousterhout's principles: deep modules, minimal exposed
complexity, interfaces that hide implementation detail, and code readable
in isolation. Complexity is the enemy — fight it at every level.

## 5. Meaningful, Reliable, Fast Tests

Tests must earn their place. When a test breaks, choose deliberately:
- **Remove** — if it adds little value or tests implementation detail
- **Rewrite** — if it was poorly structured for the scenario
- **Refactor** — if the interface it tests changed legitimately
- **Quick-fix** — only if the fix is obvious and the test is clearly valuable

Never keep a flaky or low-value test just to preserve coverage numbers.

## 6. Using sdlc

All state lives in `.sdlc/` YAML files. **Never edit them directly** — use the CLI.
Direct edits cause deserialization failures and corrupt state.

| Action | Command |
|---|---|
| Create feature | `sdlc feature create <slug> --title "…"` |
| Get next action | `sdlc next --for <slug> --json` |
| Write artifact | Write Markdown to `output_path` from the directive |
| Submit draft | `sdlc artifact draft <slug> <type>` |
| Approve artifact | `sdlc artifact approve <slug> <type>` |
| Reject artifact | `sdlc artifact reject <slug> <type>` |
| Add task | `sdlc task add <slug> "title"` |
| Start task | `sdlc task start <slug> <task-id>` |
| Complete task | `sdlc task complete <slug> <task-id>` |
| Block task | `sdlc task block <slug> <task-id> "reason"` |
| Add comment | `sdlc comment create <slug> "body"` |
| Show feature | `sdlc feature show <slug> --json` |
| List tasks | `sdlc task list <slug>` |
| Project state | `sdlc state` |
| Survey milestone waves | `sdlc project prepare [--milestone <slug>]` |
| Project phase | `sdlc project status` |

Phases advance automatically from artifact approvals — never call `sdlc feature transition`.
The only files you write directly are Markdown artifacts to `output_path`.
