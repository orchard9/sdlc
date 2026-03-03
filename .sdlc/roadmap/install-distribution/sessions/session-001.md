---
session: 1
timestamp: 2026-03-02T00:00:00Z
orientation:
  current: "Raw signal from conversation — install is broken on Windows and Ubuntu, git config overriding is a concern, envault distribution pattern is the proposed solution"
  next: "Interrogate the brief: what exactly is the envault pattern? What does cross-platform look like? What are the TypeScript errors and are they already tracked?"
  commit: "Clear problem statement + concrete distribution mechanism + decision on build system replacement"
---

## Session 1: Initial Signal Load

Bootstrapped by /sdlc-convo-mine from a team conversation on 2026-03-02.

### Signals extracted

- **make doesn't work on Windows** — Xist (a real contributor) could not install sdlc at all on Windows. cmake was suggested as the alternative, but this is symptomatic: the real fix is a distribution mechanism that doesn't require a build toolchain.
- **make install fails on Ubuntu** — 4 TypeScript errors in the frontend block the build: `RunsHeatmap.tsx:40` (unused `startMs` var), `ActionsPage.tsx:516` (`string | undefined` type), `ActionsPage.tsx:713` (`action_id` missing from type), `ActionsPage.tsx:722` (`path` missing from type). These are bugs making the install experience fragile.
- **SSH hostname alias problem** — Xist uses `github-rdp:orchard9/sdlc` format (ssh config aliases for multi-account GitHub). sdlc must not try to rewrite or canonicalize git URLs.
- **Envault distribution pattern** — jx12n confirmed he wants to copy whatever distribution approach was used for envault. That pattern should be applied to sdlc and standardized across projects.
- **Replace make everywhere** — jx12n said "i can switch my reliance on makefiles across all the projects to just a cli for that project" — the broader intent is to eliminate make as a required tool.

### Why this might matter

A tool that can't be installed is a tool that doesn't exist for real users. Xist is a technically sophisticated user (knows about ssh hostname aliases, has Ubuntu + Windows environments) and was still blocked. If he's blocked, less technical adopters have no chance. The distribution experience is the top of the funnel for every new sdlc user.

The TypeScript errors are an immediate reliability regression — they mean the main branch ships broken, which erodes trust in the project's overall quality signal.

### Open questions

1. What is the envault distribution channel? (binary releases? Homebrew? cargo-binstall? install script?)
2. Should cross-platform install ship as: pre-built binaries + install script? Or is cargo install the baseline?
3. Are the 4 TypeScript errors tracked anywhere? Are they regressions from recent changes?
4. Should sdlc detect when git remote URLs use SSH aliases and leave them untouched?
5. What does "cmake install" mean in Xist's context — is he literally suggesting cmake, or just using it as a verb meaning "something that works on Windows"?

### Suggested first exploration

Look at the envault project's distribution approach. Compare against cargo-binstall, GitHub Releases binary download, and Homebrew tap patterns. Then check whether the TypeScript errors are already tracked as features/tasks or are new regressions.
