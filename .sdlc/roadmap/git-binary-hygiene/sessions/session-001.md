---
session: 1
timestamp: 2026-03-04T00:00:00Z
orientation:
  current: "Raw signal — binary files in git without .gitattributes, plus misleading file extension on orchestrator.db"
  next: "Confirm the scope: what files need .gitattributes treatment, should they be gitignored or LFS'd, and should orchestrator.db be renamed?"
  commit: "Decision on gitignore vs LFS + naming fix + sdlc init behavior + migration path for existing repos"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation dump.

### Signals extracted

- `telemetry.redb` (2.5 MB) and `orchestrator.db` (3.5 MB) are both redb binary databases
- Both committed in the initial `98be467 Init Ponder SDLC` commit as raw binary — no `.gitattributes` binary declaration, no LFS
- `orchestrator.db` extension misleads contributors — it reads as SQLite, but it's a redb database
- "it doesn't like db used in one place and redb used in another" — naming inconsistency is already causing friction
- "In git that means big trouble" — the concern is real: binary files without `.gitattributes` binary flag produce useless diffs and fragile merges

### Why this might matter

Every project that consumes sdlc will hit this. When `sdlc init` creates the first feature and the orchestrator/telemetry stores are touched, those binary files will appear in `git status` and eventually get committed without the right declarations. Contributors will see garbage diffs. The `.db` extension also makes developers assume they can open these with SQLite tooling, causing confusion during debugging.

Two distinct problems:
1. **Git hygiene**: `.sdlc/*.redb` should be declared as binary (or gitignored if they're regenerable)
2. **Naming**: `orchestrator.db` → `orchestrator.redb` for consistency and correctness

### Open questions

- Are these files regenerable on next run? If yes, they should be gitignored entirely, not LFS'd
- If gitignored: does `sdlc init` need to update `.gitignore` (and is there a migration for existing projects)?
- If tracked: does sdlc need to set up LFS as part of `sdlc init`?
- Renaming `orchestrator.db` → `orchestrator.redb`: is there a migration step needed, or can code just try both paths?
- Should `sdlc init` write `.gitattributes` entries for `*.redb` as part of first-run setup?

### Suggested first exploration

Start with the key question: **are these files regenerable?** If telemetry and orchestrator databases are rebuilt from scratch on next run with no data loss, gitignore is the right answer — simple, no LFS overhead. If they carry essential state that can't be reconstructed, LFS is appropriate. Answer this first, then the rest falls into place.
