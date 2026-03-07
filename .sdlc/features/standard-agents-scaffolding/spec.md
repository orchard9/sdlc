# Spec: Standard Agents Scaffolding

## Summary

Every sdlc-managed project gets two **standard agents** — `knowledge-librarian` and `cto-cpo-lens` — created by the Rust CLI during `sdlc init`, not by the AI during specialize. These agents power sdlc's own features (knowledge maintenance, beat analysis) and must exist regardless of project domain.

## Problem

Today the knowledge-librarian agent is created only when `sdlc knowledge librarian init` runs (embedded template in `knowledge.rs`). The cto-cpo-lens agent exists only in the sdlc project itself — it was hand-written and never templated. New projects get neither agent during init, so `/sdlc-knowledge` and `/sdlc-beat` workflows lack the agents they depend on.

## Solution

Add standard agent scaffolding to `sdlc init` (and `sdlc update`):

1. **Embed two agent templates** as `const` strings in the init module (new file `crates/sdlc-cli/src/cmd/init/standard_agents.rs` or inline in `mod.rs`)
2. **Write agents during init** — after AGENTS.md creation (step 6), write `.claude/agents/knowledge-librarian.md` and `.claude/agents/cto-cpo-lens.md` using `write_if_missing` so user edits are never clobbered
3. **Write agents during update** — same `write_if_missing` logic so running `sdlc update` seeds missing standard agents without overwriting existing ones
4. **Update specialize instructions** — add a note to the specialize template telling the AI: "Standard agents (knowledge-librarian, cto-cpo-lens) already exist. Design project-specific agents to complement them, not replace them."

## Agent Templates

### knowledge-librarian.md
- **Model:** `claude-sonnet-4-6`
- **Purpose:** Curate `.sdlc/knowledge/` — classify entries, fill summaries, cross-reference, publish
- **Template source:** Generalize the existing `LIBRARIAN_AGENT_TEMPLATE` from `knowledge.rs` — remove the project-specific `{CATALOG_YAML}` section and use a generic starter catalog placeholder
- **Tools:** Bash, Read, Write, Edit, Glob, Grep

### cto-cpo-lens.md
- **Model:** `claude-sonnet-4-6`
- **Purpose:** Evaluate product direction against vision, identify strategic drift
- **Template source:** Generalize from the existing `.claude/agents/cto-cpo-lens.md` in this project
- **Tools:** Bash, Read, Glob, Grep

## Scope

- `crates/sdlc-cli/src/cmd/init/mod.rs` — add standard agent writes to `run()` and `update()`
- New `const` templates (either in `mod.rs` or a new `standard_agents.rs`)
- Specialize command template text (minor addition)
- No changes to `sdlc-core` — the existing `knowledge.rs` librarian template remains for the `sdlc knowledge librarian init` command which writes a richer, project-aware version

## Out of Scope

- Changing the knowledge-librarian template in `knowledge.rs` (that's the project-aware version)
- Adding more standard agents beyond the two listed
- Agent auto-update or version migration

## Acceptance Criteria

1. After `sdlc init` on a fresh project, `.claude/agents/knowledge-librarian.md` and `.claude/agents/cto-cpo-lens.md` exist
2. Running `sdlc init` again does not overwrite manually edited agent files (`write_if_missing` semantics)
3. Running `sdlc update` creates missing standard agents without clobbering existing ones
4. The specialize workflow acknowledges standard agents and doesn't replace them
5. Both agent files have correct frontmatter (model, description, tools)
