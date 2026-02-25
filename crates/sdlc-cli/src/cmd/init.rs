use anyhow::Context;
use sdlc_core::{
    config::{Config, PlatformArg, PlatformCommand, PlatformConfig},
    io, paths,
    state::State,
};
use std::collections::HashMap;
use std::path::Path;

pub fn run(root: &Path, platform: Option<&str>) -> anyhow::Result<()> {
    let project_name = root
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "project".to_string());

    println!("Initializing SDLC in: {}", root.display());

    // 1. Create .sdlc directory structure
    let dirs = [
        paths::SDLC_DIR,
        paths::FEATURES_DIR,
        paths::PATTERNS_DIR,
        paths::AUDITS_DIR,
        paths::BRANCHES_DIR,
        paths::ARCHIVES_DIR,
        paths::ROADMAP_DIR,
    ];
    for dir in dirs {
        let p = root.join(dir);
        io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
    }

    // 2. Write config.yaml if missing
    let config_path = paths::config_path(root);
    if !config_path.exists() {
        let cfg = Config::new(&project_name);
        cfg.save(root).context("failed to write config.yaml")?;
        println!("  created: .sdlc/config.yaml");
    } else {
        println!("  exists:  .sdlc/config.yaml");
    }

    // 3. Write state.yaml if missing
    let state_path = paths::state_path(root);
    if !state_path.exists() {
        let state = State::new(&project_name);
        state.save(root).context("failed to write state.yaml")?;
        println!("  created: .sdlc/state.yaml");
    } else {
        println!("  exists:  .sdlc/state.yaml");
    }

    // 4. Create .ai knowledge base skeleton
    let ai_lookup_dirs = [
        ".ai",
        ".ai/patterns",
        ".ai/decisions",
        ".ai/gotchas",
        ".ai/architecture",
        ".ai/conventions",
    ];
    for dir in ai_lookup_dirs {
        let p = root.join(dir);
        io::ensure_dir(&p)?;
    }

    let index_path = root.join(paths::AI_LOOKUP_INDEX);
    io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())?;

    // 5. Write AGENTS.md SDLC section
    write_agents_md(root, &project_name)?;

    // 6. Write .claude/commands
    write_claude_commands(root)?;

    // 7. Scaffold platform if requested
    if let Some(platform_name) = platform {
        scaffold_platform(root, platform_name)?;
    }

    println!("\nSDLC initialized successfully.");
    println!("Next: sdlc feature create <slug> --title \"...\"");

    Ok(())
}

fn scaffold_platform(root: &Path, platform_name: &str) -> anyhow::Result<()> {
    match platform_name {
        "masquerade" => scaffold_masquerade(root),
        other => anyhow::bail!(
            "unknown platform '{}'; supported platforms: masquerade",
            other
        ),
    }
}

fn scaffold_masquerade(root: &Path) -> anyhow::Result<()> {
    let platform_dir = root.join(".sdlc/platform");
    io::ensure_dir(&platform_dir)?;

    let scripts: &[(&str, &str)] = &[
        ("deploy.sh", MASQ_DEPLOY_SCRIPT),
        ("logs.sh", MASQ_LOGS_SCRIPT),
        ("dev-start.sh", MASQ_DEV_START_SCRIPT),
        ("dev-stop.sh", MASQ_DEV_STOP_SCRIPT),
        ("dev-quality.sh", MASQ_DEV_QUALITY_SCRIPT),
        ("dev-migrate.sh", MASQ_DEV_MIGRATE_SCRIPT),
    ];

    for (filename, content) in scripts {
        let path = platform_dir.join(filename);
        let created = io::write_if_missing(&path, content.as_bytes())?;
        if created {
            println!("  created: .sdlc/platform/{filename}");
        } else {
            println!("  exists:  .sdlc/platform/{filename}");
        }
    }

    // Update config.yaml with platform section if not already present
    let mut config = Config::load(root).context("failed to load config")?;
    if config.platform.is_none() {
        config.platform = Some(masquerade_platform_config());
        config.save(root).context("failed to save config")?;
        println!("  updated: .sdlc/config.yaml (platform section added)");
    } else {
        println!("  exists:  .sdlc/config.yaml (platform section already present)");
    }

    println!("\nPlatform 'masquerade' scaffolded.");
    println!("Edit .sdlc/platform/*.sh to wire up real commands.");
    println!("Run: sdlc platform list");

    Ok(())
}

fn masquerade_platform_config() -> PlatformConfig {
    let services = vec![
        "auth-service".to_string(),
        "creator-api".to_string(),
        "websocket-service".to_string(),
        "creator-studio-web".to_string(),
        "admin-portal-web".to_string(),
    ];

    let mut commands: HashMap<String, PlatformCommand> = HashMap::new();

    commands.insert(
        "deploy".to_string(),
        PlatformCommand {
            description: "Build and deploy a service to staging or production".to_string(),
            script: ".sdlc/platform/deploy.sh".to_string(),
            args: vec![
                PlatformArg {
                    name: "service".to_string(),
                    required: true,
                    choices: services.clone(),
                },
                PlatformArg {
                    name: "environment".to_string(),
                    required: true,
                    choices: vec!["staging".to_string(), "production".to_string()],
                },
            ],
            subcommands: HashMap::new(),
        },
    );

    commands.insert(
        "logs".to_string(),
        PlatformCommand {
            description: "Get logs from a deployed service".to_string(),
            script: ".sdlc/platform/logs.sh".to_string(),
            args: vec![PlatformArg {
                name: "service".to_string(),
                required: false,
                choices: services,
            }],
            subcommands: HashMap::new(),
        },
    );

    let mut dev_subcommands: HashMap<String, String> = HashMap::new();
    dev_subcommands.insert(
        "start".to_string(),
        ".sdlc/platform/dev-start.sh".to_string(),
    );
    dev_subcommands.insert("stop".to_string(), ".sdlc/platform/dev-stop.sh".to_string());
    dev_subcommands.insert(
        "quality".to_string(),
        ".sdlc/platform/dev-quality.sh".to_string(),
    );
    dev_subcommands.insert(
        "migrate".to_string(),
        ".sdlc/platform/dev-migrate.sh".to_string(),
    );

    commands.insert(
        "dev".to_string(),
        PlatformCommand {
            description: "Development environment management".to_string(),
            script: String::new(),
            args: Vec::new(),
            subcommands: dev_subcommands,
        },
    );

    PlatformConfig { commands }
}

fn write_agents_md(root: &Path, project_name: &str) -> anyhow::Result<()> {
    let agents_path = paths::agents_md_path(root);
    let sdlc_section = format!(
        "\n\n## SDLC\n\n\
        This project uses `sdlc` (state machine) + `xadk` (AI orchestrator) for autonomous feature development.\n\n\
        ### Setup\n\n\
        1. Install xadk: `uv pip install /path/to/xadk`\n\
        2. Create `.env` with `XADK_ROOT=$(pwd)` and `GEMINI_API_KEY=<key>` (or Vertex AI credentials)\n\
        3. Run agents: `python -m xadk <agent_id> --interactive`\n\
        4. List available agents: `python -m xadk list`\n\n\
        ### Commands\n\n\
        - `sdlc feature create <slug>` — create a new feature\n\
        - `sdlc next --for <slug>` — classify the next action\n\
        - `sdlc artifact approve <slug> <type>` — approve an artifact to advance the phase\n\
        - `sdlc state` — show project state\n\
        - `sdlc feature list` — list all features and their phases\n\n\
        ### Phases\n\n\
        draft → specified → planned → ready → implementation → review → audit → qa → merge → released\n\n\
        ### Artifact Types\n\n\
        `spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`\n\n\
        Project: {project_name}\n"
    );

    if agents_path.exists() {
        let existing = std::fs::read_to_string(&agents_path)?;
        if !existing.contains("## SDLC") {
            io::append_text(&agents_path, &sdlc_section)?;
            println!("  updated: AGENTS.md (added SDLC section)");
        } else {
            println!("  exists:  AGENTS.md (SDLC section already present)");
        }
    } else {
        let content =
            format!("# AGENTS.md\n\nAgent instructions for {project_name}.{sdlc_section}");
        io::atomic_write(&agents_path, content.as_bytes())?;
        println!("  created: AGENTS.md");
    }

    Ok(())
}

fn write_claude_commands(root: &Path) -> anyhow::Result<()> {
    let commands_dir = paths::claude_commands_dir(root);
    io::ensure_dir(&commands_dir)?;

    let commands = [
        ("sdlc-next.md", SDLC_NEXT_COMMAND),
        ("sdlc-status.md", SDLC_STATUS_COMMAND),
        ("sdlc-approve.md", SDLC_APPROVE_COMMAND),
    ];

    for (filename, content) in commands {
        let path = commands_dir.join(filename);
        let created = io::write_if_missing(&path, content.as_bytes())?;
        if created {
            println!("  created: .claude/commands/{filename}");
        } else {
            println!("  exists:  .claude/commands/{filename}");
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Platform script templates
// ---------------------------------------------------------------------------

const MASQ_DEPLOY_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform deploy <service> <environment>
set -e

SERVICE="$1"
ENVIRONMENT="$2"

echo "Deploying $SERVICE to $ENVIRONMENT..."
# TODO: wire up real deploy command
# Example: kubectl set image deployment/$SERVICE $SERVICE=$REGISTRY/$SERVICE:latest -n $ENVIRONMENT
echo "Deploy complete: $SERVICE -> $ENVIRONMENT"
"#;

const MASQ_LOGS_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform logs [service]
set -e

SERVICE="${1:-}"

if [ -n "$SERVICE" ]; then
    echo "Fetching logs for $SERVICE..."
    # TODO: kubectl logs -n production deployment/$SERVICE --tail=100 -f
else
    echo "Fetching logs for all services..."
    # TODO: kubectl logs -n production --all-containers=true
fi
"#;

const MASQ_DEV_START_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev start
set -e

echo "Starting development environment..."
# TODO: docker compose up -d
echo "Dev environment started."
"#;

const MASQ_DEV_STOP_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev stop
set -e

echo "Stopping development environment..."
# TODO: docker compose down
echo "Dev environment stopped."
"#;

const MASQ_DEV_QUALITY_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev quality
set -e

echo "Running quality checks..."
# TODO: run linters, type checks, and unit tests
echo "Quality checks complete."
"#;

const MASQ_DEV_MIGRATE_SCRIPT: &str = r#"#!/usr/bin/env sh
# sdlc platform dev migrate
set -e

echo "Running database migrations..."
# TODO: run migration tool against local dev database
echo "Migrations complete."
"#;

// ---------------------------------------------------------------------------
// Static file content
// ---------------------------------------------------------------------------

const AI_LOOKUP_INDEX_CONTENT: &str = r#"# .ai Index

Project knowledge base. Entries are organized by category.

## Categories

- **patterns/** — How we do things (coding patterns, architectural conventions)
- **decisions/** — Why we chose X over Y (ADRs, trade-off notes)
- **gotchas/** — Non-obvious pitfalls and workarounds
- **architecture/** — How the system works (data flow, component relationships)
- **conventions/** — Naming, style, standards

## Usage

Entries are harvested automatically after each SDLC artifact is approved.
Each entry follows the format:

```
---
category: patterns
title: How we handle X
learned: YYYY-MM-DD
source: spec|design|review|human
confidence: high|medium|low
---

## Summary
...

## Key Facts
- ...

## File Pointer
`path/to/file.go:line-range`
```
"#;

const SDLC_NEXT_COMMAND: &str = r#"---
description: Classify the next SDLC action for a feature and drive it forward
argument-hint: <feature-slug>
allowed-tools: Bash
---

# sdlc-next

Drive the next SDLC action for a feature.

## Steps

1. **Get the slug** from $ARGUMENTS. If none provided, run `sdlc next` (no flags)
   to show all active features, then ask the user which one to drive.

2. **Classify** the next action:
   ```bash
   sdlc next --for <slug> --json
   ```
   Parse the JSON. Key fields: `action`, `message`, `output_path`, `is_heavy`, `current_phase`.

3. **Warn if heavy** (`is_heavy: true`): Tell the user this is a long-running task
   (implementation or QA) and ask for confirmation before proceeding.

4. **Dispatch** based on `action`:

   **`create_spec`**
   Ask the user for a brief feature description if not already provided, then run:
   ```bash
   python -m xadk sdlc_spec --prompt "Create a spec for the '<slug>' feature. <description>"
   ```

   **`approve_spec`**
   Read and display the spec. Use `output_path` from the JSON; if null, fall back to
   `.sdlc/features/<slug>/spec.md`. Show the full file contents, then say:
   > "Review the spec above. Run `/sdlc-approve <slug> spec` when ready to advance."
   Do not approve automatically — this is a human gate.

   **`create_design`**
   The `sdlc_design` agent is not yet implemented. Do NOT run any agent.
   Instead, inform the user:
   > "The next action is `create_design`. No automated agent exists for this phase yet.
   > Write `.sdlc/features/<slug>/design.md` manually, then run `/sdlc-approve <slug> design`."

   **`WaitForApproval`** or **`ApproveMerge`**
   This is a human gate. Surface the message:
   > "⏸ Human gate for '<slug>': <message>"
   Wait for the user to explicitly act before continuing.

   **`Done`**
   > "✓ All SDLC phases complete for '<slug>'."

   **Any other action**
   Show the raw classification JSON and say:
   > "Next action for '<slug>' is `<action>`. No automated agent for this phase yet.
   > See `.sdlc/features/<slug>/` for current artifacts."

5. **After dispatch**, run `sdlc next --for <slug>` (no --json) to show the updated state.
"#;

const SDLC_STATUS_COMMAND: &str = r#"# sdlc-status

Show the current SDLC state for this project.

## Usage

```
/sdlc-status [feature-slug]
```

## Behavior

1. Run `sdlc state --json` to get project state
2. If a feature slug is provided, run `sdlc feature show <slug> --json`
3. Display a clean summary of phases, artifacts, and pending actions
"#;

const SDLC_APPROVE_COMMAND: &str = r#"# sdlc-approve

Approve an artifact to unblock the next SDLC phase.

## Usage

```
/sdlc-approve <feature-slug> <artifact-type>
```

Artifact types: spec, design, tasks, qa_plan, review, audit, qa_results

## Behavior

1. Run `sdlc artifact approve <slug> <type>`
2. Run `sdlc next --for <slug>` to show what happens next
3. If a phase transition occurred, report the new phase
"#;
