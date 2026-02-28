# sdlc Quickstart

Get from zero to driving features with AI agents in 5 minutes.

---

## 1. Install

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/orchard9/sdlc-releases/releases/latest/download/sdlc-installer.sh | sh
```

**Windows:**
```powershell
powershell -ExecutionPolicy ByPass -c "irm https://github.com/orchard9/sdlc-releases/releases/latest/download/sdlc-installer.ps1 | iex"
```

**Homebrew:**
```bash
brew install orchard9/tap/sdlc
```

Verify:
```bash
sdlc --version
```

---

## 2. Initialize a project

```bash
cd your-project
sdlc init
```

This creates `.sdlc/` (state + config), injects `AGENTS.md`, and installs slash commands for Claude Code, Gemini CLI, OpenCode, and Codex.

---

## 3. Two ways to start

### A. I have an idea — explore it first

```bash
# Open the ideation workspace (in Claude Code)
/sdlc-ponder "I want to build a preference system with cohort layering"
```

This creates a ponder entry at `.sdlc/roadmap/<slug>/`. The session interrogates your idea, recruits thought partners, and captures artifacts in a scrapbook.

When ready, commit the idea into the state machine:
```bash
/sdlc-ponder-commit <slug>
```

This synthesizes the scrapbook into milestones and features via `/sdlc-plan`, then drops you into execution.

### B. I know what I'm building — create a feature directly

```bash
sdlc feature create auth-login --title "User authentication with OAuth"
sdlc next --for auth-login
```

Output:
```json
{
  "feature": "auth-login",
  "action": "create_spec",
  "message": "No spec exists. Write the feature specification for 'auth-login'.",
  "output_path": ".sdlc/features/auth-login/spec.md"
}
```

---

## 4. Drive a feature

**One step at a time (human controls cadence):**
```bash
/sdlc-next auth-login
```

**Full autonomous run to next human gate:**
```bash
/sdlc-run auth-login
```

The agent reads `sdlc next --for auth-login --json`, executes the action, and loops — writing specs, designs, tasks, implementing code, reviewing, auditing, and running QA. It stops only when it hits a blocker or `done`.

---

## 5. Check project state

```bash
sdlc state                    # project overview
sdlc next                     # next action for every active feature
sdlc query needs-approval     # features waiting for artifact approval
sdlc query blocked            # features with blockers
sdlc ui                       # open web dashboard
```

---

## Feature lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

Each phase requires approved artifacts. The classifier always tells you what's next.

---

## Roadmap (ideation)

Ideas before they're features live in `.sdlc/roadmap/`:

```
ponder entry (exploring)
  → exploring / converging
  → /sdlc-ponder-commit → milestones + features
  → /sdlc-pressure-test → validate against user needs
  → /sdlc-run → execute
```

```bash
sdlc ponder list              # active ideas
sdlc ponder show <slug>       # scrapbook + team + status
sdlc ponder archive <slug>    # park an idea without deleting
```

---

## Milestones

Group features under a shared goal:

```bash
sdlc milestone create v2 --title "Version 2.0"
sdlc milestone add-feature v2 auth-login
sdlc milestone info v2
```

Validate the milestone before executing:
```bash
/sdlc-pressure-test v2        # empathy interviews against milestone scope
/sdlc-milestone-uat v2        # run the acceptance test
```

---

## Next steps

- See [`README.md`](README.md) for the full CLI reference and core concepts
- See [`docs/vision.md`](docs/vision.md) for the design philosophy
- See [`AGENTS.md`](AGENTS.md) for the full agent instruction set
- Run `sdlc init` in any project — sdlc works with any codebase and any AI coding tool
