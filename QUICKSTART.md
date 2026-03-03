# Ponder Quickstart

Get from zero to driving features with AI agents in 5 minutes.

---

## 1. Install

**macOS / Linux:**
```bash
curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
```

**Windows:**
```powershell
irm https://raw.githubusercontent.com/orchard9/sdlc/main/install.ps1 | iex
```

Installs `ponder` to `~/.local/bin` with a `sdlc` alias — both work interchangeably.

```bash
ponder --version   # or: sdlc --version
```

---

## 2. Initialize a project

```bash
cd your-project
sdlc init
```

Creates `.sdlc/` (state + config), injects `AGENTS.md`, and installs slash commands for Claude Code, Gemini CLI, OpenCode, and Codex.

---

## 3. Set up Vision and Architecture

```bash
sdlc ui
```

Navigate to **Setup** (`/setup`) and define:

- **Vision** — why the project exists and who it serves. Agents use this to make decisions aligned with your goals.
- **Architecture** — how the system works, key components, and technical constraints. Agents use this to understand boundaries.

Once set, you're ready to create features.

---

## 4. Two ways to start

### A. I have an idea — explore it first

```bash
/sdlc-ponder "I want to build a preference system with cohort layering"
```

Creates a ponder entry at `.sdlc/roadmap/<slug>/`. The session interrogates your idea, recruits thought partners, and captures artifacts. When ready:

```bash
/sdlc-ponder-commit <slug>
```

Synthesizes the scrapbook into milestones and features, then drops you into execution.

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

## 5. Drive a feature

**One step at a time:**
```bash
/sdlc-next auth-login
```

**Full autonomous run to next human gate:**
```bash
/sdlc-run auth-login
```

The agent reads `sdlc next --for auth-login --json`, executes the action, and loops — writing specs, designs, tasks, implementing code, reviewing, auditing, and running QA. It stops only when it hits a blocker or `done`.

---

## 6. Daily workflow

```bash
# See what needs attention across all features
sdlc next

# Get a machine-readable directive for a specific feature
sdlc next --for auth-login --json

# Approve an artifact to advance the phase
sdlc artifact approve auth-login spec

# See all features waiting for approval
sdlc query needs-approval

# See all blocked features
sdlc query blocked

# Open the web dashboard
sdlc ui
```

---

## 7. AI coding tool commands

After `sdlc init`, these slash commands are installed automatically:

**Claude Code:**
| Command | Purpose |
|---|---|
| `/sdlc-ponder [slug]` | Open ideation workspace — explore ideas, recruit thought partners |
| `/sdlc-ponder-commit <slug>` | Crystallize idea into milestones and features |
| `/sdlc-recruit <role>` | Recruit an expert thought partner as a persistent agent |
| `/sdlc-empathy <subject>` | Deep user perspective interviews before making decisions |
| `/sdlc-next <slug>` | Get the next directive and act on it |
| `/sdlc-run <slug>` | Autonomously drive a feature to the next human gate |
| `/sdlc-plan` | Distribute a plan into milestones, features, and tasks |
| `/sdlc-status` | Project overview |
| `/sdlc-pressure-test <milestone>` | Pressure-test a milestone against user perspectives |
| `/sdlc-milestone-uat <milestone>` | Run the acceptance test for a milestone |

**Gemini CLI:** `.gemini/commands/*.toml`
**OpenCode:** `.opencode/command/*.md`
**Codex:** `.agents/skills/*/SKILL.md`

---

## Feature lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

Each phase requires approved artifacts. The classifier always tells you what's next.

---

## Roadmap (ideation)

```bash
sdlc ponder list              # active ideas
sdlc ponder show <slug>       # scrapbook + team + status
sdlc ponder archive <slug>    # park an idea without deleting
```

---

## Milestones

```bash
sdlc milestone create v2 --title "Version 2.0"
sdlc milestone add-feature v2 auth-login
sdlc milestone info v2
/sdlc-pressure-test v2        # validate against user perspectives
/sdlc-milestone-uat v2        # run the acceptance test
```

---

## Updating

Re-run your install command to get the latest binary, then:

```bash
sdlc update
```

Refreshes the `/sdlc-*` slash commands in `~/.claude/commands/`, `~/.gemini/commands/`, etc. Run after every upgrade to keep your AI tools in sync.

---

## Next steps

- [`README.md`](README.md) — full CLI reference and core concepts
- [`DEVELOPER.md`](DEVELOPER.md) — contributor setup, dev loop, build targets
- [`docs/vision.md`](docs/vision.md) — design philosophy
- [`AGENTS.md`](AGENTS.md) — full agent instruction set
