# SDLC Tools

Project-specific tools installed by sdlc. Use `sdlc tool run <name>` to invoke.

Run `sdlc tool sync` to regenerate this file from live tool metadata.

---

## ama — AMA — Ask Me Anything

Answers questions about the codebase by searching a pre-built keyword index.

**Run:** `sdlc tool run ama --question "..."`
**Setup required:** Yes — `sdlc tool run ama --setup`
_Indexes source files for keyword search (run once, then re-run when files change significantly)_

---

## quality-check — Quality Check

Runs platform shell commands from .sdlc/config.yaml and reports pass/fail.

**Run:** `sdlc tool run quality-check`
**Setup required:** No
_Add checks under `platform.commands` in `.sdlc/config.yaml`_

---

## Adding a Custom Tool

Run `sdlc tool scaffold <name> "<description>"` to create a new tool skeleton.
Then implement the `run()` function in `.sdlc/tools/<name>/tool.ts` and run `sdlc tool sync`.
