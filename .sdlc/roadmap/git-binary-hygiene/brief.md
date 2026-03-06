# Brief: Git Binary File Hygiene

**Origin:** Extracted from team conversation (Discord/Slack) — 2026-03-04

## Summary

The `.sdlc/` directory contains binary database files (`.redb`) that were committed without `.gitattributes` binary declarations or LFS configuration. This is a hidden landmine for any project that consumes sdlc — git diffs will be nonsense, merges will silently corrupt, and file size bloat will grow. Additionally, `orchestrator.db` uses a misleading `.db` extension when it is actually a redb-format file.

## Key Signals

- **[Strong / Engineering]** "This looks like a binary file. In git that means big trouble if .gitattributes is not correctly configured in a repo." — direct concern about git hygiene
- **[Strong / Engineering]** Both `telemetry.redb` and `orchestrator.db` were introduced in commit `98be467 Init Ponder SDLC` as raw binary with no LFS, no `.gitattributes` binary declaration
- **[Strong / Engineering]** "The `.db` extension on `orchestrator.db` is misleading; it's actually a redb database, not SQLite" — naming confusion for contributors
- **[Weak / Product]** "telemetry" — the filename may reveal implementation internals to users who inspect `.sdlc/` contents

## Relevant Excerpts (verbatim)

> This looks like a binary file. In git that means big trouble if .gitattributes is not correctly configured in a repo.

> | .sdlc/telemetry.redb | redb v2 database | 2.5 MB | redb\x1a\x0a\xa9\x0d\x0a |
> | .sdlc/orchestrator.db | redb database (misnamed!) | 3.5 MB | redb\x1a\x0a\xa9\x0d\x0a |

> Both were introduced in commit `98be467 Init Ponder SDLC` as raw binary — no LFS, no `.gitattributes` binary declaration.

> The `.db` extension on `orchestrator.db` is misleading; it's actually a redb database, not SQLite.

> it doesn't like db used in one place and redb used in another

## Open Questions

- Should `.sdlc/*.redb` and `.sdlc/*.db` be declared as `binary` in `.gitattributes`, or tracked via LFS?
- Is LFS realistic for projects consuming sdlc? Or should these files be in `.gitignore` with a migration path?
- Should `orchestrator.db` be renamed to `orchestrator.redb` for consistency? (Breaking change in path — needs migration logic)
- Does `sdlc init` need to write/update `.gitattributes` for new projects?
- For existing projects with binary files already in history — what's the recovery path?
- What's the right long-term answer: redb local (gitignored, regenerable) or postgres for cluster mode?
