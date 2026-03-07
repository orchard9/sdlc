# Design: Standard Agents Scaffolding

## Approach

Add a `write_standard_agents(root)` function to `crates/sdlc-cli/src/cmd/init/mod.rs` that writes two agent files using `write_if_missing`. Call it from both `run()` (init) and export it for `update.rs`.

## File Changes

### 1. `crates/sdlc-cli/src/cmd/init/mod.rs`

**New constants:**

```rust
const STANDARD_AGENT_KNOWLEDGE_LIBRARIAN: &str = r#"---
model: claude-sonnet-4-6
description: Knowledge librarian — classifies, cross-references, and maintains the project knowledge base
tools: Bash, Read, Write, Edit, Glob, Grep
---

# Knowledge Librarian
..."#;

const STANDARD_AGENT_CTO_CPO_LENS: &str = r#"---
model: claude-sonnet-4-6
description: Strategic CTO/CPO lens — evaluates product direction against vision, surfaces drift
tools: Bash, Read, Glob, Grep
---

# CTO/CPO Lens
..."#;
```

**New function:**

```rust
pub fn write_standard_agents(root: &Path) -> anyhow::Result<()> {
    let agents_dir = root.join(".claude/agents");
    io::ensure_dir(&agents_dir)?;

    let agents: &[(&str, &str)] = &[
        ("knowledge-librarian.md", STANDARD_AGENT_KNOWLEDGE_LIBRARIAN),
        ("cto-cpo-lens.md", STANDARD_AGENT_CTO_CPO_LENS),
    ];

    for (filename, content) in agents {
        let path = agents_dir.join(filename);
        let created = io::write_if_missing(&path, content.as_bytes())?;
        if created {
            println!("  created: .claude/agents/{filename}");
        } else {
            println!("  exists:  .claude/agents/{filename}");
        }
    }
    Ok(())
}
```

**Integration in `run()`:** Insert call between step 6 (AGENTS.md) and step 7 (user scaffolding):

```rust
// 6.5. Write standard agents (.claude/agents/)
println!("\nInstalling standard agents:");
write_standard_agents(root)?;
```

### 2. `crates/sdlc-cli/src/cmd/update.rs`

Add `write_standard_agents` to the import list and call it after `write_agents_md`:

```rust
println!("\nInstalling standard agents:");
write_standard_agents(root)?;
```

### 3. Specialize template update

In the specialize command template (`sdlc_specialize.rs` or wherever the specialize prompt lives), add a note:

> Standard agents `knowledge-librarian` and `cto-cpo-lens` are pre-installed by `sdlc init`. Do not replace them. Design project-specific agents to complement them.

## Template Content

The knowledge-librarian template will be a simplified version of the one in `knowledge.rs` — no `{CATALOG_YAML}` placeholder, no project-specific catalog. Generic instructions for curating `.sdlc/knowledge/`.

The cto-cpo-lens template will match the existing `.claude/agents/cto-cpo-lens.md` content with proper frontmatter.

## Key Decisions

- **`write_if_missing`** — never clobber user edits
- **`.claude/agents/`** — project-level, shared via git (same location specialize uses)
- **No `{PROJECT_NAME}` substitution** — keep templates static and simple; the agent reads VISION.md at runtime for project context
- **Separate from `knowledge.rs` template** — that template is richer (includes catalog YAML) and is used by `sdlc knowledge librarian init`. The standard agent is a simpler starter version.
