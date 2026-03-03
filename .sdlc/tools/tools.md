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

Runs checks from .sdlc/tools/quality-check/config.yaml and reports pass/fail.

**Run:** `sdlc tool run quality-check`
**Setup required:** No
_Edit `.sdlc/tools/quality-check/config.yaml` to add your project's checks_

---

## dev-driver — Dev Driver

Finds the next development action and dispatches it — advances the project one step per tick.

**Run:** `sdlc tool run dev-driver`
**Setup required:** No
_Configure via orchestrator: Label=dev-driver, Tool=dev-driver, Input={}, Recurrence=14400. See `.sdlc/tools/dev-driver/README.md` for full docs._

---

## telegram-recap — Telegram Recap

Fetch and email a Telegram chat digest — pulls messages from the configured window and sends via SMTP.

**Run:** `sdlc tool run telegram-recap --input '{}'`
**Setup required:** Yes — `sdlc tool run telegram-recap --setup`
_Requires 7 secrets: TELEGRAM_BOT_TOKEN, SMTP_HOST, SMTP_PORT, SMTP_USERNAME, SMTP_PASSWORD, SMTP_FROM, SMTP_TO. Schedule with orchestrator (--every 86400) for a daily digest._

---

## Adding a Custom Tool

Run `sdlc tool scaffold <name> "<description>"` to create a new tool skeleton.
Then implement the `run()` function in `.sdlc/tools/<name>/tool.ts` and run `sdlc tool sync`.
