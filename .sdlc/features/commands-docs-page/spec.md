# Spec: Commands Catalog — /docs/commands

## Problem

The `/docs/commands` page in the web UI currently shows a placeholder ("Templated agent commands — /sdlc-run, /sdlc-next, /sdlc-ponder, /sdlc-plan, and more."). There is no actual content. Users navigating to the docs section have no discoverable reference for the 34 `sdlc-*` slash commands that ship with the product. The information exists in Rust source (`crates/sdlc-cli/src/cmd/init/commands/`) but is not surfaced in the UI.

## Goal

Render a real, browseable commands catalog at `/docs/commands`. Each command entry shows its name, a one-line description, usage hint, and a copy button for the invocation string. Commands are grouped by workflow category. No backend endpoint is required — the data is embedded in the frontend at build time, derived from the same source-of-truth that ships the commands.

## User Stories

- **As a new user** browsing the web UI, I can go to Docs → Commands and see every available `sdlc-*` command with its purpose, so I know what tools I have.
- **As a returning user**, I can quickly find the right command by scanning the categories (lifecycle, planning, workspace, analysis, tooling, project) without reading all 34 entries.
- **As any user**, I can click a copy button next to a command name to copy the invocation string to my clipboard.

## Scope

### In scope

1. Replace the placeholder `DocsPage` commands section with a real catalog component (`CommandsCatalog`).
2. Define a static `COMMANDS` data array in the frontend, typed and categorized — no API call needed.
3. Each command entry includes:
   - `slug` — e.g. `sdlc-run`
   - `invocation` — e.g. `/sdlc-run <feature-slug>`
   - `description` — one-sentence purpose statement
   - `hint` — optional argument hint shown in muted text
   - `category` — one of: `lifecycle`, `planning`, `workspace`, `analysis`, `tooling`, `project`
4. Commands are grouped by category with a section header; within each category commands are listed alphabetically.
5. Each row has a `CopyButton` (already exists at `frontend/src/components/shared/CopyButton.tsx`) that copies the invocation string.
6. The catalog is searchable: a search input at the top filters commands by name or description (client-side, no API).
7. The catalog renders within the existing `DocsPage` section routing — no new route needed, just replacing the placeholder content for `section === 'commands'`.

### Out of scope

- A backend `/api/commands` endpoint — data is static in the frontend.
- Rendering the full command markdown body (that is a future feature).
- Command versioning or changelog — out of scope for this feature.
- Any other docs sections (quickstart, planning-flow, etc.) — those remain placeholders.

## Command Inventory (34 commands)

Derived from `crates/sdlc-cli/src/cmd/init/commands/mod.rs` `ALL_COMMANDS`:

### Lifecycle (core feature loop)
| Command | Invocation | Description |
|---|---|---|
| sdlc-next | `/sdlc-next <feature-slug>` | Get the next directive for a feature and act on it |
| sdlc-run | `/sdlc-run <feature-slug>` | Autonomously drive a feature to completion |
| sdlc-approve | `/sdlc-approve <feature-slug>` | Approve the current pending artifact for a feature |
| sdlc-status | `/sdlc-status` | Show project and feature status overview |

### Planning
| Command | Invocation | Description |
|---|---|---|
| sdlc-plan | `/sdlc-plan` | Distribute a plan into milestones, features, and tasks |
| sdlc-prepare | `/sdlc-prepare <milestone-slug>` | Pre-flight a milestone — align features with vision, fix gaps, write wave plan |
| sdlc-run-wave | `/sdlc-run-wave <milestone-slug>` | Execute Wave 1 features in parallel, advance to next wave |
| sdlc-pressure-test | `/sdlc-pressure-test <milestone-slug>` | Pressure-test a milestone against user perspectives |
| sdlc-milestone-uat | `/sdlc-milestone-uat <milestone-slug>` | Run acceptance test for a milestone |
| sdlc-specialize | `/sdlc-specialize <feature-slug>` | Shape a feature with specialized domain knowledge |

### Workspace (ideation & exploration)
| Command | Invocation | Description |
|---|---|---|
| sdlc-ponder | `/sdlc-ponder [slug or new idea]` | Open the ideation workspace with recruited thought partners |
| sdlc-ponder-commit | `/sdlc-ponder-commit <slug>` | Crystallize a pondered idea into milestones and features |
| sdlc-recruit | `/sdlc-recruit <role>` | Recruit an expert thought partner as a persistent agent |
| sdlc-empathy | `/sdlc-empathy <subject>` | Deep user perspective interviews before making decisions |
| sdlc-spike | `/sdlc-spike <topic>` | Time-boxed technical investigation on an uncertain area |
| sdlc-hypothetical-planning | `/sdlc-hypothetical-planning <scenario>` | Plan a hypothetical scenario without committing state |
| sdlc-hypothetical-do | `/sdlc-hypothetical-do <scenario>` | Execute a hypothetical scenario |
| sdlc-convo-mine | `/sdlc-convo-mine` | Extract insights and tasks from conversation history |

### Analysis & Quality
| Command | Invocation | Description |
|---|---|---|
| sdlc-enterprise-readiness | `/sdlc-enterprise-readiness` | Production readiness analysis |
| sdlc-quality-fix | `/sdlc-quality-fix` | Fix failing quality-check results — triage by failure count, fix-forward / fix-all / remediate |
| sdlc-setup-quality-gates | `/sdlc-setup-quality-gates` | Set up pre-commit hooks |
| sdlc-vision-adjustment | `/sdlc-vision-adjustment` | Adjust the project vision document |
| sdlc-architecture-adjustment | `/sdlc-architecture-adjustment` | Adjust the architecture document |
| sdlc-guideline | `/sdlc-guideline [slug]` | Open the guideline workspace — gather evidence and publish engineering guidelines |

### Tooling (tool & skill management)
| Command | Invocation | Description |
|---|---|---|
| sdlc-tool-run | `/sdlc-tool-run <tool-name>` | Run a custom tool |
| sdlc-tool-build | `/sdlc-tool-build <tool-name>` | Build a new custom tool |
| sdlc-skill-build | `/sdlc-skill-build <skill-name>` | Build a new agent skill |
| sdlc-tool-audit | `/sdlc-tool-audit <tool-name>` | Audit a tool for correctness and quality |
| sdlc-tool-uat | `/sdlc-tool-uat <tool-name>` | User acceptance test a custom tool |
| sdlc-cookbook | `/sdlc-cookbook` | Browse and run cookbook recipes |
| sdlc-cookbook-run | `/sdlc-cookbook-run <recipe>` | Run a specific cookbook recipe |
| sdlc-suggest | `/sdlc-suggest` | Get suggestions for what to work on next |

### Project Setup
| Command | Invocation | Description |
|---|---|---|
| sdlc-init | `/sdlc-init` | Interview to bootstrap vision, architecture, config, and team for a new project |

## Data Shape (TypeScript)

```typescript
type CommandCategory =
  | 'lifecycle'
  | 'planning'
  | 'workspace'
  | 'analysis'
  | 'tooling'
  | 'project'

interface CommandEntry {
  slug: string           // "sdlc-run"
  invocation: string     // "/sdlc-run <feature-slug>"
  description: string    // one-sentence
  category: CommandCategory
}
```

## Component Structure

```
DocsPage (existing)
  └── CommandsCatalog (new, rendered when section === 'commands')
        ├── search input (client-side filter)
        └── per-category group
              ├── category header (h3, label + count badge)
              └── CommandRow (per command)
                    ├── invocation (monospace, bold)
                    ├── description (muted text)
                    └── CopyButton (copies invocation)
```

## Acceptance Criteria

1. `/docs/commands` renders a list of all 34 commands with name, description, and copy button — no placeholder dashes.
2. Commands are grouped into at least 5 categories with a visible section header.
3. Typing in the search box filters commands by name or description in real-time.
4. Clicking the copy button on any command copies the invocation string to the clipboard.
5. The page uses existing design system tokens (no new Tailwind classes beyond utilities already in use in the app).
6. No new API routes are added — all data is static in the frontend.
7. Build passes: `npm run build` in `frontend/` succeeds with no TypeScript errors.
