# Commands Page Implementation Spec

## Feature: commands-docs-page

Implement the /docs/commands page in the web UI — a browseable catalog of all 34 /sdlc-* commands, grouped by category, with copy-on-click and 1-line descriptions.

---

## Backend: GET /api/commands

Add to `crates/sdlc-server/src/routes/` a new `commands.rs` module.

### CommandMeta struct

```rust
#[derive(serde::Serialize)]
pub struct CommandMeta {
    pub slug: &'static str,
    pub description: &'static str,
    pub hint: &'static str,
    pub category: &'static str,
}
```

### Static catalog (34 commands with categories)

| slug | category |
|---|---|
| sdlc-init | onboarding |
| sdlc-specialize | onboarding |
| sdlc-ponder | ideation |
| sdlc-ponder-commit | ideation |
| sdlc-suggest | ideation |
| sdlc-empathy | ideation |
| sdlc-recruit | ideation |
| sdlc-convo-mine | ideation |
| sdlc-guideline | ideation |
| sdlc-run | execution |
| sdlc-next | execution |
| sdlc-approve | execution |
| sdlc-status | execution |
| sdlc-plan | planning |
| sdlc-prepare | planning |
| sdlc-run-wave | planning |
| sdlc-pressure-test | planning |
| sdlc-milestone-uat | quality |
| sdlc-cookbook | quality |
| sdlc-cookbook-run | quality |
| sdlc-quality-fix | quality |
| sdlc-setup-quality-gates | quality |
| sdlc-enterprise-readiness | quality |
| sdlc-tool-audit | quality |
| sdlc-tool-uat | quality |
| sdlc-tool-build | tooling |
| sdlc-tool-run | tooling |
| sdlc-skill-build | tooling |
| sdlc-vision-adjustment | adjustment |
| sdlc-architecture-adjustment | adjustment |
| sdlc-spike | research |
| sdlc-hypothetical-planning | research |
| sdlc-hypothetical-do | research |
| sdlc-convo-mine | ideation |

### Route

```
GET /api/commands  →  Vec<CommandMeta> (JSON)
```

Register in `crates/sdlc-server/src/lib.rs`.

---

## Fix: 5 missing gemini_descriptions

Edit the source `.rs` files for these commands:

| Command | New description |
|---|---|
| sdlc-convo-mine | Mine conversations for actionable signal and launch ponder sessions per theme group |
| sdlc-enterprise-readiness | Analyze project for enterprise-grade production readiness and distribute findings into milestones |
| sdlc-guideline | Build an evidence-backed guideline from five research lenses and real codebase evidence |
| sdlc-init | Bootstrap vision, architecture, config, and team through a conversational interview |
| sdlc-prepare | Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark ready |

---

## Frontend: CommandsPage

### New files
- `frontend/src/pages/CommandsPage.tsx` — full page component
- `frontend/src/hooks/useCommands.ts` — data fetching hook

### Component structure

```tsx
// Tab bar: All | Onboarding | Ideation | Execution | Planning | Quality | Tooling | Adjustment | Research
// Command cards: /slug (copy button) + description + hint chip
```

### Wire-up

In `frontend/src/pages/DocsPage.tsx`, replace the placeholder div with:
```tsx
{section === 'commands' ? <CommandsPage /> : <placeholder />}
```

---

## Follow-up tasks (not V1 blockers)

- [ ] Add 'when to use' examples for ambiguous commands (sdlc-approve, sdlc-hypothetical-*)
- [ ] Move CommandDef + ALL_COMMANDS to sdlc-core for single source of truth
- [ ] Cross-implementation comparison view (Agy vs Claude vs OpenCode)
