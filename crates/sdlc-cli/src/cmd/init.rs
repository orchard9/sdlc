use anyhow::Context;
use sdlc_core::{
    config::{Config, PlatformArg, PlatformCommand, PlatformConfig},
    io, paths,
    state::State,
};
use std::collections::HashMap;
use std::path::Path;

/// Version of the sdlc binary embedded at compile time.
pub const SDLC_BINARY_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Marker that delimits the managed SDLC section in AGENTS.md.
pub const SDLC_SECTION_START: &str = "<!-- sdlc:start -->";
/// Closing marker for the managed SDLC section in AGENTS.md.
pub const SDLC_SECTION_END: &str = "<!-- sdlc:end -->";

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

    // 4. Write / refresh engineering guidance (always overwritten — managed content)
    write_guidance_md(root)?;

    // 4.5. Write / refresh core tool suite (.sdlc/tools/)
    println!("\nInstalling core tool suite:");
    write_core_tools(root)?;

    // 5. Create .ai knowledge base skeleton
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

    // 6. Write / refresh AGENTS.md SDLC section
    write_agents_md(root, &project_name)?;

    // 7. Write agent scaffolding to user home (Claude, Gemini, OpenCode, Agents)
    println!("\nInstalling user-level command scaffolding:");
    install_user_scaffolding()?;

    // 8. Remove any legacy project-level sdlc files from prior versions
    migrate_legacy_project_scaffolding(root)?;

    // 9. Stamp sdlc_version in config.yaml
    stamp_sdlc_version(root)?;

    // 10. Scaffold platform if requested
    if let Some(platform_name) = platform {
        scaffold_platform(root, platform_name)?;
    }

    println!("\nSDLC initialized successfully.");
    println!("Next: sdlc feature create <slug> --title \"...\"");

    Ok(())
}

/// Install (or refresh) all user-level agent scaffolding.
/// Called by both `sdlc init` and `sdlc update`.
pub fn install_user_scaffolding() -> anyhow::Result<()> {
    write_user_claude_commands()?;
    write_user_gemini_commands()?;
    write_user_opencode_commands()?;
    write_user_agents_skills()?;
    Ok(())
}

/// Stamp the current binary version into `.sdlc/config.yaml`.
/// Idempotent — only writes if the stored version differs.
pub fn stamp_sdlc_version(root: &Path) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config.yaml")?;
    if config.sdlc_version.as_deref() != Some(SDLC_BINARY_VERSION) {
        let previous = config.sdlc_version.as_deref().unwrap_or("none").to_string();
        let mut updated = config;
        updated.sdlc_version = Some(SDLC_BINARY_VERSION.to_string());
        updated.save(root).context("failed to save config.yaml")?;
        println!("  stamped: .sdlc/config.yaml (sdlc_version {previous} → {SDLC_BINARY_VERSION})");
    }
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

/// Write (or overwrite) `.sdlc/guidance.md` — the engineering principles file.
///
/// Always overwrites: guidance is managed content, not user-editable config.
/// Called by both `sdlc init` and `sdlc update`.
pub fn write_guidance_md(root: &Path) -> anyhow::Result<()> {
    let path = paths::guidance_md_path(root);
    let existed = path.exists();
    io::atomic_write(&path, GUIDANCE_MD_CONTENT.as_bytes())?;
    if existed {
        println!("  updated: .sdlc/guidance.md");
    } else {
        println!("  created: .sdlc/guidance.md");
    }
    Ok(())
}

/// Write or refresh the SDLC section in AGENTS.md.
///
/// - Creates AGENTS.md with markers if it doesn't exist.
/// - Replaces content between `<!-- sdlc:start -->` / `<!-- sdlc:end -->` markers if present.
/// - Migrates a legacy unmarked `## SDLC` section to the marker format.
/// - Appends with markers if no SDLC section exists yet.
pub fn write_agents_md(root: &Path, project_name: &str) -> anyhow::Result<()> {
    let agents_path = paths::agents_md_path(root);
    let marked_section = build_sdlc_marked_section(project_name);

    if !agents_path.exists() {
        let content =
            format!("# AGENTS.md\n\nAgent instructions for {project_name}.\n\n{marked_section}\n");
        io::atomic_write(&agents_path, content.as_bytes())?;
        println!("  created: AGENTS.md");
        return Ok(());
    }

    let existing = std::fs::read_to_string(&agents_path)?;

    if existing.contains(SDLC_SECTION_START) {
        // Markers present — replace in-place
        if io::replace_between_markers(
            &agents_path,
            SDLC_SECTION_START,
            SDLC_SECTION_END,
            &marked_section,
        )? {
            println!("  updated: AGENTS.md (SDLC section refreshed)");
        } else {
            println!("  warning: AGENTS.md has sdlc:start but no sdlc:end marker — skipped");
        }
    } else if existing.contains("## SDLC") {
        // Legacy format without markers — migrate
        let new_content = replace_legacy_sdlc_section(&existing, &marked_section);
        io::atomic_write(&agents_path, new_content.as_bytes())?;
        println!("  updated: AGENTS.md (SDLC section converted to markers)");
    } else {
        // No SDLC section at all — append
        io::append_text(&agents_path, &format!("\n\n{marked_section}\n"))?;
        println!("  updated: AGENTS.md (SDLC section added)");
    }

    Ok(())
}

/// Build the full marked SDLC section string (start marker + content + end marker).
fn build_sdlc_marked_section(project_name: &str) -> String {
    let inner = build_sdlc_section_inner(project_name);
    format!("{SDLC_SECTION_START}{inner}{SDLC_SECTION_END}")
}

fn build_sdlc_section_inner(project_name: &str) -> String {
    format!(
        "\n\n## SDLC\n\n\
        > **Required reading:** `.sdlc/guidance.md` — engineering principles that govern \
        all implementation decisions on this project. <!-- sdlc:guidance -->\n\n\
        This project uses `sdlc` as its SDLC state machine. `sdlc` manages feature lifecycle, \
        artifacts, tasks, and milestones. It emits structured directives via `sdlc next --json` \
        that any consumer (Claude Code, custom scripts, or humans) acts on to decide what to do next.\n\n\
        Consumer scaffolding is installed globally under `~/.claude/commands/`, `~/.gemini/commands/`, \
        `~/.opencode/command/`, and `~/.agents/skills/` — available across all projects. \
        Use `/sdlc-specialize` in Claude Code to generate a project-specific AI team \
        (agents + skills) tailored to this project's tech stack and roles.\n\n\
        ### Key Commands\n\n\
        - `sdlc feature create <slug> --title \"...\"` — create a new feature\n\
        - `sdlc next --for <slug> --json` — get the next action directive (JSON)\n\
        - `sdlc next` — show all active features and their next actions\n\
        - `sdlc artifact approve <slug> <type>` — approve an artifact to advance the phase\n\
        - `sdlc state` — show project state\n\
        - `sdlc feature list` — list all features and their phases\n\
        - `sdlc task list [<slug>]` — list tasks for a feature (or all tasks)\n\n\
        ### Lifecycle\n\n\
        draft → specified → planned → ready → implementation → review → audit → qa → merge → released\n\n\
        Treat this lifecycle as the default pathway. You can use explicit manual transitions when needed, \
        but approvals/artifacts are the recommended way to keep quality and traceability.\n\n\
        ### Artifact Types\n\n\
        `spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`\n\n\
        ### CRITICAL: Never edit .sdlc/ YAML directly\n\n\
        All state changes go through `sdlc` CLI commands. See §6 of `.sdlc/guidance.md` \
        for the full command reference. Direct YAML edits corrupt state.\n\n\
        ### Directive Interface\n\n\
        Use `sdlc next --for <slug> --json` to get the next directive. The JSON output tells the \
        consumer what to do next (action, message, output_path, is_heavy, gates).\n\n\
        ### Consumer Commands\n\n\
        - `/sdlc-next <slug>` — execute one step, then stop (human controls cadence)\n\
        - `/sdlc-run <slug>` — run autonomously to completion\n\
        - `/sdlc-status [<slug>]` — show current state\n\
        - `/sdlc-plan` — distribute a plan into milestones, features, and tasks\n\
        - `/sdlc-milestone-uat <milestone-slug>` — run the acceptance test for a milestone\n\
        - `/sdlc-pressure-test <milestone-slug>` — pressure-test a milestone against user perspectives\n\
        - `/sdlc-vision-adjustment [description]` — align all docs, sdlc state, and code to a vision change\n\
        - `/sdlc-architecture-adjustment [description]` — align all docs, code, and sdlc state to an architecture change\n\
        - `/sdlc-enterprise-readiness [--stage <stage>]` — analyze production readiness\n\
        - `/sdlc-setup-quality-gates` — set up pre-commit hooks and quality gates\n\
        - `/sdlc-cookbook <milestone-slug>` — create developer-scenario cookbook recipes\n\
        - `/sdlc-cookbook-run <milestone-slug>` — execute cookbook recipes and record results\n\
        - `/sdlc-ponder [slug]` — open the ideation workspace for exploring and committing ideas\n\
        - `/sdlc-ponder-commit <slug>` — crystallize a pondered idea into milestones and features\n\
        - `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent\n\
        - `/sdlc-empathy <subject>` — deep user perspective interviews before decisions\n\n\
        ### Tool Suite\n\n\
        <!-- sdlc:tools -->\n\
        Project-scoped TypeScript tools in `.sdlc/tools/` — callable by agents and humans\n\
        during any lifecycle phase. Read `.sdlc/tools/tools.md` for the full help menu.\n\n\
        - `sdlc tool list` — show installed tools\n\
        - `sdlc tool run <name> [args]` — run a tool; pass `--json '{{...}}'` for complex input\n\
        - `sdlc tool sync` — regenerate `tools.md` after adding a custom tool\n\
        - `sdlc tool scaffold <name> \"desc\"` — create a new tool skeleton\n\n\
        **Core tools:** `ama` (codebase Q&A), `quality-check` (runs platform shell gates)\n\n\
        Use `/sdlc-tool-run`, `/sdlc-tool-build`, `/sdlc-tool-audit`, `/sdlc-tool-uat` in Claude Code for guided tool workflows.\n\
        <!-- /sdlc:tools -->\n\n\
        Project: {project_name}\n\n"
    )
}

/// Replace the legacy unmarked `## SDLC` section with `marked_section`.
/// Finds the heading, replaces through the next `## ` heading or EOF.
fn replace_legacy_sdlc_section(existing: &str, marked_section: &str) -> String {
    let heading = "## SDLC";
    let Some(heading_pos) = existing.find(heading) else {
        return format!("{}\n\n{}\n", existing.trim_end(), marked_section);
    };

    // Walk backwards over blank lines to find where to start the replacement
    let section_start = existing[..heading_pos].trim_end_matches('\n').len();

    // Find where this section ends: next level-2 heading or EOF
    let after_heading = heading_pos + heading.len();
    let section_end = existing[after_heading..]
        .find("\n## ")
        .map(|i| after_heading + i)
        .unwrap_or(existing.len());

    let before = existing[..section_start].trim_end();
    let after = existing[section_end..].trim_start_matches('\n');

    if after.is_empty() {
        format!("{before}\n\n{marked_section}\n")
    } else {
        format!("{before}\n\n{marked_section}\n\n{after}")
    }
}

/// Upsert command files into a user-level commands directory.
/// Prints "created:" or "updated:" based on whether the file existed before.
fn write_user_command_scaffold(
    commands_dir: &Path,
    display_prefix: &str,
    commands: &[(&str, &str)],
) -> anyhow::Result<()> {
    io::ensure_dir(commands_dir)?;

    for (filename, content) in commands {
        let path = commands_dir.join(filename);
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())?;
        if existed {
            println!("  updated: {display_prefix}/{filename}");
        } else {
            println!("  created: {display_prefix}/{filename}");
        }
    }

    Ok(())
}

/// Upsert command files (owned Strings) into a user-level commands directory.
fn write_user_command_scaffold_owned(
    commands_dir: &Path,
    display_prefix: &str,
    commands: &[(&str, String)],
) -> anyhow::Result<()> {
    io::ensure_dir(commands_dir)?;

    for (filename, content) in commands {
        let path = commands_dir.join(filename);
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())?;
        if existed {
            println!("  updated: {display_prefix}/{filename}");
        } else {
            println!("  created: {display_prefix}/{filename}");
        }
    }

    Ok(())
}

/// Upsert SKILL.md files into a user-level agents/skills directory.
fn write_user_skill_scaffold(
    skills_dir: &Path,
    display_prefix: &str,
    skills: &[(&str, &str)],
) -> anyhow::Result<()> {
    io::ensure_dir(skills_dir)?;

    for (skill_name, content) in skills {
        let skill_dir = skills_dir.join(skill_name);
        io::ensure_dir(&skill_dir)?;
        let path = skill_dir.join("SKILL.md");
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())?;
        if existed {
            println!("  updated: {display_prefix}/{skill_name}/SKILL.md");
        } else {
            println!("  created: {display_prefix}/{skill_name}/SKILL.md");
        }
    }

    Ok(())
}

fn remove_if_exists(dir: &Path, filenames: &[&str]) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for filename in filenames {
        let path = dir.join(filename);
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }

    Ok(())
}

fn gemini_command_toml(description: &str, prompt: &str) -> String {
    format!(
        r#"description = "{description}"

prompt = """
{prompt}
"""
"#
    )
}

fn opencode_command_md(description: &str, argument_hint: &str, body: &str) -> String {
    format!(
        r#"---
description: {description}
argument-hint: {argument_hint}
---

{body}
"#
    )
}

fn write_user_claude_commands() -> anyhow::Result<()> {
    let commands_dir = paths::user_claude_commands_dir()?;
    write_user_command_scaffold(
        &commands_dir,
        "~/.claude/commands",
        &[
            ("sdlc-next.md", SDLC_NEXT_COMMAND),
            ("sdlc-status.md", SDLC_STATUS_COMMAND),
            ("sdlc-approve.md", SDLC_APPROVE_COMMAND),
            ("sdlc-specialize.md", SDLC_SPECIALIZE_COMMAND),
            ("sdlc-run.md", SDLC_RUN_COMMAND),
            ("sdlc-plan.md", SDLC_PLAN_COMMAND),
            ("sdlc-milestone-uat.md", SDLC_MILESTONE_UAT_COMMAND),
            ("sdlc-pressure-test.md", SDLC_PRESSURE_TEST_COMMAND),
            (
                "sdlc-enterprise-readiness.md",
                SDLC_ENTERPRISE_READINESS_COMMAND,
            ),
            (
                "sdlc-setup-quality-gates.md",
                SDLC_SETUP_QUALITY_GATES_COMMAND,
            ),
            ("sdlc-cookbook.md", SDLC_COOKBOOK_COMMAND),
            ("sdlc-cookbook-run.md", SDLC_COOKBOOK_RUN_COMMAND),
            ("sdlc-ponder.md", SDLC_PONDER_COMMAND),
            ("sdlc-ponder-commit.md", SDLC_PONDER_COMMIT_COMMAND),
            ("sdlc-recruit.md", SDLC_RECRUIT_COMMAND),
            ("sdlc-empathy.md", SDLC_EMPATHY_COMMAND),
            ("sdlc-prepare.md", SDLC_PREPARE_COMMAND),
            ("sdlc-run-wave.md", SDLC_RUN_WAVE_COMMAND),
            ("sdlc-tool-run.md", SDLC_TOOL_RUN_COMMAND),
            ("sdlc-tool-build.md", SDLC_TOOL_BUILD_COMMAND),
            ("sdlc-tool-audit.md", SDLC_TOOL_AUDIT_COMMAND),
            ("sdlc-tool-uat.md", SDLC_TOOL_UAT_COMMAND),
            ("sdlc-quality-fix.md", SDLC_QUALITY_FIX_COMMAND),
            ("sdlc-vision-adjustment.md", SDLC_VISION_ADJUSTMENT_COMMAND),
            (
                "sdlc-architecture-adjustment.md",
                SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND,
            ),
        ],
    )
}

fn write_user_gemini_commands() -> anyhow::Result<()> {
    let commands = vec![
        (
            "sdlc-next.toml",
            gemini_command_toml(
                "Get the next SDLC directive for a feature and act on it",
                SDLC_NEXT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-status.toml",
            gemini_command_toml(
                "Show SDLC state for the project or a feature",
                SDLC_STATUS_PLAYBOOK,
            ),
        ),
        (
            "sdlc-approve.toml",
            gemini_command_toml("Review and approve an SDLC artifact", SDLC_APPROVE_PLAYBOOK),
        ),
        (
            "sdlc-run.toml",
            gemini_command_toml(
                "Autonomously drive a feature to completion",
                SDLC_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-plan.toml",
            gemini_command_toml(
                "Distribute a plan into sdlc milestones, features, and tasks",
                SDLC_PLAN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-milestone-uat.toml",
            gemini_command_toml(
                "Run the acceptance test for a milestone",
                SDLC_MILESTONE_UAT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-pressure-test.toml",
            gemini_command_toml(
                "Pressure-test a milestone against user perspectives",
                SDLC_PRESSURE_TEST_PLAYBOOK,
            ),
        ),
        (
            "sdlc-enterprise-readiness.toml",
            gemini_command_toml(
                "Analyze project for enterprise readiness and distribute findings into sdlc",
                SDLC_ENTERPRISE_READINESS_PLAYBOOK,
            ),
        ),
        (
            "sdlc-setup-quality-gates.toml",
            gemini_command_toml(
                "Set up pre-commit hooks and quality gates for this project",
                SDLC_SETUP_QUALITY_GATES_PLAYBOOK,
            ),
        ),
        (
            "sdlc-specialize.toml",
            gemini_command_toml(
                "Survey this project and generate a tailored AI team (agents + skills)",
                SDLC_SPECIALIZE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-cookbook.toml",
            gemini_command_toml(
                "Create developer-scenario cookbook recipes for a milestone",
                SDLC_COOKBOOK_PLAYBOOK,
            ),
        ),
        (
            "sdlc-cookbook-run.toml",
            gemini_command_toml(
                "Execute cookbook recipes and record results for a milestone",
                SDLC_COOKBOOK_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-ponder.toml",
            gemini_command_toml(
                "Open the ideation workspace for exploring ideas with thought partners",
                SDLC_PONDER_PLAYBOOK,
            ),
        ),
        (
            "sdlc-ponder-commit.toml",
            gemini_command_toml(
                "Crystallize a pondered idea into milestones and features",
                SDLC_PONDER_COMMIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-recruit.toml",
            gemini_command_toml(
                "Recruit an expert thought partner as a persistent agent",
                SDLC_RECRUIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-empathy.toml",
            gemini_command_toml(
                "Interview user perspectives deeply before making decisions",
                SDLC_EMPATHY_PLAYBOOK,
            ),
        ),
        (
            "sdlc-prepare.toml",
            gemini_command_toml(
                "Pre-flight a milestone — align features, fix gaps, write wave plan, mark prepared",
                SDLC_PREPARE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-run-wave.toml",
            gemini_command_toml(
                "Execute Wave 1 features in parallel, then advance to the next wave",
                SDLC_RUN_WAVE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-run.toml",
            gemini_command_toml(
                "Run an SDLC tool and act on the JSON result",
                SDLC_TOOL_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-build.toml",
            gemini_command_toml(
                "Scaffold and implement a new SDLC tool",
                SDLC_TOOL_BUILD_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-audit.toml",
            gemini_command_toml(
                "Audit an SDLC tool against the full quality contract",
                SDLC_TOOL_AUDIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-uat.toml",
            gemini_command_toml("Run UAT scenarios for an SDLC tool", SDLC_TOOL_UAT_PLAYBOOK),
        ),
        (
            "sdlc-quality-fix.toml",
            gemini_command_toml(
                "Fix failing quality-check results by triage and targeted fix",
                SDLC_QUALITY_FIX_PLAYBOOK,
            ),
        ),
        (
            "sdlc-vision-adjustment.toml",
            gemini_command_toml(
                "Align all project docs, sdlc state, and code to a vision change",
                SDLC_VISION_ADJUSTMENT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-architecture-adjustment.toml",
            gemini_command_toml(
                "Align all project docs, code, and sdlc state to an architecture change",
                SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK,
            ),
        ),
    ];

    write_user_command_scaffold_owned(
        &paths::user_gemini_commands_dir()?,
        "~/.gemini/commands",
        &commands,
    )
}

fn write_user_opencode_commands() -> anyhow::Result<()> {
    let commands = vec![
        (
            "sdlc-next.md",
            opencode_command_md(
                "Get the next SDLC directive for a feature and act on it",
                "<feature-slug>",
                SDLC_NEXT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-status.md",
            opencode_command_md(
                "Show SDLC state for the project or a specific feature",
                "[feature-slug]",
                SDLC_STATUS_PLAYBOOK,
            ),
        ),
        (
            "sdlc-approve.md",
            opencode_command_md(
                "Review and approve an SDLC artifact",
                "<feature-slug> <artifact-type>",
                SDLC_APPROVE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-run.md",
            opencode_command_md(
                "Autonomously drive a feature to completion",
                "<feature-slug>",
                SDLC_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-plan.md",
            opencode_command_md(
                "Distribute a plan into sdlc milestones, features, and tasks",
                "[--file <path>]",
                SDLC_PLAN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-milestone-uat.md",
            opencode_command_md(
                "Run the acceptance test for a milestone",
                "<milestone-slug>",
                SDLC_MILESTONE_UAT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-pressure-test.md",
            opencode_command_md(
                "Pressure-test a milestone against user perspectives",
                "<milestone-slug>",
                SDLC_PRESSURE_TEST_PLAYBOOK,
            ),
        ),
        (
            "sdlc-enterprise-readiness.md",
            opencode_command_md(
                "Analyze project for enterprise readiness and distribute findings into sdlc",
                "[--stage <stage>] [--into <milestone-slug>]",
                SDLC_ENTERPRISE_READINESS_PLAYBOOK,
            ),
        ),
        (
            "sdlc-setup-quality-gates.md",
            opencode_command_md(
                "Set up pre-commit hooks and quality gates for this project",
                "<setup|update|check|fix>",
                SDLC_SETUP_QUALITY_GATES_PLAYBOOK,
            ),
        ),
        (
            "sdlc-specialize.md",
            opencode_command_md(
                "Survey this project and generate a tailored AI team (agents + skills)",
                "[project-description]",
                SDLC_SPECIALIZE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-cookbook.md",
            opencode_command_md(
                "Create developer-scenario cookbook recipes for a milestone",
                "<milestone-slug>",
                SDLC_COOKBOOK_PLAYBOOK,
            ),
        ),
        (
            "sdlc-cookbook-run.md",
            opencode_command_md(
                "Execute cookbook recipes and record results for a milestone",
                "<milestone-slug>",
                SDLC_COOKBOOK_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-ponder.md",
            opencode_command_md(
                "Open the ideation workspace for exploring ideas with thought partners",
                "[slug or new idea]",
                SDLC_PONDER_PLAYBOOK,
            ),
        ),
        (
            "sdlc-ponder-commit.md",
            opencode_command_md(
                "Crystallize a pondered idea into milestones and features",
                "<ponder-slug>",
                SDLC_PONDER_COMMIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-recruit.md",
            opencode_command_md(
                "Recruit an expert thought partner as a persistent agent",
                "<role or domain context>",
                SDLC_RECRUIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-empathy.md",
            opencode_command_md(
                "Interview user perspectives deeply before making decisions",
                "<feature, system, or decision>",
                SDLC_EMPATHY_PLAYBOOK,
            ),
        ),
        (
            "sdlc-prepare.md",
            opencode_command_md(
                "Pre-flight a milestone — align features, fix gaps, write wave plan, mark prepared",
                "<milestone-slug>",
                SDLC_PREPARE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-run-wave.md",
            opencode_command_md(
                "Execute Wave 1 features in parallel, then advance to the next wave",
                "<milestone-slug>",
                SDLC_RUN_WAVE_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-run.md",
            opencode_command_md(
                "Run an SDLC tool and act on the JSON result",
                "<tool-name> [args]",
                SDLC_TOOL_RUN_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-build.md",
            opencode_command_md(
                "Scaffold and implement a new SDLC tool",
                "<name> \"<description>\"",
                SDLC_TOOL_BUILD_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-audit.md",
            opencode_command_md(
                "Audit an SDLC tool against the full quality contract",
                "<tool-name>",
                SDLC_TOOL_AUDIT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-tool-uat.md",
            opencode_command_md(
                "Run UAT scenarios for an SDLC tool",
                "<tool-name>",
                SDLC_TOOL_UAT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-quality-fix.md",
            opencode_command_md(
                "Fix failing quality-check results by triage and targeted fix",
                "[tool-name]",
                SDLC_QUALITY_FIX_PLAYBOOK,
            ),
        ),
        (
            "sdlc-vision-adjustment.md",
            opencode_command_md(
                "Align all project docs, sdlc state, and code to a vision change",
                "[describe the vision change]",
                SDLC_VISION_ADJUSTMENT_PLAYBOOK,
            ),
        ),
        (
            "sdlc-architecture-adjustment.md",
            opencode_command_md(
                "Align all project docs, code, and sdlc state to an architecture change",
                "[describe the architecture change]",
                SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK,
            ),
        ),
    ];

    write_user_command_scaffold_owned(
        &paths::user_opencode_commands_dir()?,
        "~/.opencode/command",
        &commands,
    )
}

fn write_user_agents_skills() -> anyhow::Result<()> {
    write_user_skill_scaffold(
        &paths::user_agents_skills_dir()?,
        "~/.agents/skills",
        &[
            ("sdlc-next", SDLC_NEXT_SKILL),
            ("sdlc-status", SDLC_STATUS_SKILL),
            ("sdlc-approve", SDLC_APPROVE_SKILL),
            ("sdlc-run", SDLC_RUN_SKILL),
            ("sdlc-plan", SDLC_PLAN_SKILL),
            ("sdlc-milestone-uat", SDLC_MILESTONE_UAT_SKILL),
            ("sdlc-pressure-test", SDLC_PRESSURE_TEST_SKILL),
            ("sdlc-enterprise-readiness", SDLC_ENTERPRISE_READINESS_SKILL),
            ("sdlc-setup-quality-gates", SDLC_SETUP_QUALITY_GATES_SKILL),
            ("sdlc-specialize", SDLC_SPECIALIZE_SKILL),
            ("sdlc-cookbook", SDLC_COOKBOOK_SKILL),
            ("sdlc-cookbook-run", SDLC_COOKBOOK_RUN_SKILL),
            ("sdlc-ponder", SDLC_PONDER_SKILL),
            ("sdlc-ponder-commit", SDLC_PONDER_COMMIT_SKILL),
            ("sdlc-recruit", SDLC_RECRUIT_SKILL),
            ("sdlc-empathy", SDLC_EMPATHY_SKILL),
            ("sdlc-prepare", SDLC_PREPARE_SKILL),
            ("sdlc-run-wave", SDLC_RUN_WAVE_SKILL),
            ("sdlc-tool-run", SDLC_TOOL_RUN_SKILL),
            ("sdlc-tool-build", SDLC_TOOL_BUILD_SKILL),
            ("sdlc-tool-audit", SDLC_TOOL_AUDIT_SKILL),
            ("sdlc-tool-uat", SDLC_TOOL_UAT_SKILL),
            ("sdlc-quality-fix", SDLC_QUALITY_FIX_SKILL),
            ("sdlc-vision-adjustment", SDLC_VISION_ADJUSTMENT_SKILL),
            (
                "sdlc-architecture-adjustment",
                SDLC_ARCHITECTURE_ADJUSTMENT_SKILL,
            ),
        ],
    )
}

/// Remove legacy project-level sdlc scaffolding written by older versions of `sdlc init`.
pub fn migrate_legacy_project_scaffolding(root: &Path) -> anyhow::Result<()> {
    let sdlc_files = &[
        "sdlc-next.md",
        "sdlc-status.md",
        "sdlc-approve.md",
        "sdlc-specialize.md",
        "sdlc-run.md",
        "sdlc-plan.md",
        "sdlc-milestone-uat.md",
        "sdlc-pressure-test.md",
        "sdlc-enterprise-readiness.md",
        "sdlc-setup-quality-gates.md",
        "sdlc-cookbook.md",
        "sdlc-cookbook-run.md",
        "sdlc-ponder.md",
        "sdlc-ponder-commit.md",
        "sdlc-recruit.md",
        "sdlc-empathy.md",
        "sdlc-prepare.md",
        "sdlc-run-wave.md",
        "sdlc-tool-run.md",
        "sdlc-tool-build.md",
        "sdlc-tool-audit.md",
        "sdlc-tool-uat.md",
        "sdlc-quality-fix.md",
        "sdlc-vision-adjustment.md",
        "sdlc-architecture-adjustment.md",
    ];

    remove_if_exists(&paths::claude_commands_dir(root), sdlc_files)?;
    remove_if_exists(
        &paths::gemini_commands_dir(root),
        &[
            "sdlc-next.toml",
            "sdlc-status.toml",
            "sdlc-approve.toml",
            "sdlc-specialize.toml",
            "sdlc-run.toml",
            "sdlc-plan.toml",
            "sdlc-milestone-uat.toml",
            "sdlc-pressure-test.toml",
            "sdlc-enterprise-readiness.toml",
            "sdlc-setup-quality-gates.toml",
            "sdlc-cookbook.toml",
            "sdlc-cookbook-run.toml",
            "sdlc-ponder.toml",
            "sdlc-ponder-commit.toml",
            "sdlc-recruit.toml",
            "sdlc-empathy.toml",
            "sdlc-prepare.toml",
            "sdlc-run-wave.toml",
            "sdlc-tool-run.toml",
            "sdlc-tool-build.toml",
            "sdlc-tool-audit.toml",
            "sdlc-tool-uat.toml",
            "sdlc-quality-fix.toml",
            "sdlc-vision-adjustment.toml",
            "sdlc-architecture-adjustment.toml",
            "sdlc-next.md",
            "sdlc-status.md",
            "sdlc-approve.md",
            "sdlc-specialize.md",
            "sdlc-run.md",
            "sdlc-plan.md",
            "sdlc-milestone-uat.md",
            "sdlc-pressure-test.md",
            "sdlc-enterprise-readiness.md",
            "sdlc-setup-quality-gates.md",
            "sdlc-cookbook.md",
            "sdlc-cookbook-run.md",
            "sdlc-ponder.md",
            "sdlc-ponder-commit.md",
            "sdlc-recruit.md",
            "sdlc-empathy.md",
            "sdlc-prepare.md",
            "sdlc-run-wave.md",
            "sdlc-tool-run.md",
            "sdlc-tool-build.md",
            "sdlc-tool-audit.md",
            "sdlc-tool-uat.md",
            "sdlc-quality-fix.md",
            "sdlc-vision-adjustment.md",
            "sdlc-architecture-adjustment.md",
        ],
    )?;
    remove_if_exists(&paths::opencode_commands_dir(root), sdlc_files)?;
    remove_if_exists(&root.join(".opencode/commands"), sdlc_files)?;
    remove_if_exists(
        &paths::codex_skills_dir(root),
        &[
            "sdlc-next/SKILL.md",
            "sdlc-status/SKILL.md",
            "sdlc-approve/SKILL.md",
            "sdlc-specialize/SKILL.md",
            "sdlc-run/SKILL.md",
            "sdlc-plan/SKILL.md",
            "sdlc-milestone-uat/SKILL.md",
            "sdlc-pressure-test/SKILL.md",
            "sdlc-enterprise-readiness/SKILL.md",
            "sdlc-setup-quality-gates/SKILL.md",
            "sdlc-cookbook/SKILL.md",
            "sdlc-cookbook-run/SKILL.md",
            "sdlc-ponder/SKILL.md",
            "sdlc-ponder-commit/SKILL.md",
            "sdlc-recruit/SKILL.md",
            "sdlc-empathy/SKILL.md",
            "sdlc-prepare/SKILL.md",
            "sdlc-run-wave/SKILL.md",
            "sdlc-tool-run/SKILL.md",
            "sdlc-tool-build/SKILL.md",
            "sdlc-tool-audit/SKILL.md",
            "sdlc-tool-uat/SKILL.md",
            "sdlc-quality-fix/SKILL.md",
            "sdlc-vision-adjustment/SKILL.md",
            "sdlc-architecture-adjustment/SKILL.md",
        ],
    )?;
    remove_if_exists(&root.join(".codex/commands"), sdlc_files)?;

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

const GUIDANCE_MD_CONTENT: &str = r#"# Engineering Guidance

Read this before any implementation, bug fix, or test action.

## North Star: Vision & Architecture

Before writing a single line of code, read:

- **`VISION.md`** — *what* we are building and *why*. Every feature, every tradeoff, every design decision must serve this vision. If a proposed change works against it, surface it before proceeding.
- **`ARCHITECTURE.md`** — *how* the system works. Components, interfaces, data flows, and sequence diagrams showing how everything fits together. Code must conform to the architecture — never silently deviate.

These are the guiding light. When in doubt about any decision, return to them first.

## 1. Build It Right

Do it the proper way — not the quick way. The correct solution is one that
will still be correct in six months. Favor proven patterns, clear
abstractions, and designs that are easy to understand and extend. Never
trade long-term correctness for short-term convenience.

## 2. Understand Bugs Before Fixing Them

Before touching a bug, trace its root cause holistically — read surrounding
code, follow the data flow, understand why it broke. Fix the cause, not the
symptom. A patch that introduces a new bug in three months is worse than
no fix.

## 3. Enterprise Quality Bar

We build enterprise-grade software. The bar is Steve Jobs: relentless
attention to detail, nothing ships that embarrasses us, correctness and
reliability are non-negotiable. If something isn't right, make it right.

## 4. Philosophy of Software Design

Follow John Ousterhout's principles: deep modules, minimal exposed
complexity, interfaces that hide implementation detail, and code readable
in isolation. Complexity is the enemy — fight it at every level.

## 5. Meaningful, Reliable, Fast Tests

Tests must earn their place. When a test breaks, choose deliberately:
- **Remove** — if it adds little value or tests implementation detail
- **Rewrite** — if it was poorly structured for the scenario
- **Refactor** — if the interface it tests changed legitimately
- **Quick-fix** — only if the fix is obvious and the test is clearly valuable

Never keep a flaky or low-value test just to preserve coverage numbers.

## 6. Using sdlc

All state lives in `.sdlc/` YAML files. **Never edit them directly** — use the CLI.
Direct edits cause deserialization failures and corrupt state.

| Action | Command |
|---|---|
| Create feature | `sdlc feature create <slug> --title "…"` |
| Get next action | `sdlc next --for <slug> --json` |
| Write artifact | Write Markdown to `output_path` from the directive |
| Submit draft | `sdlc artifact draft <slug> <type>` |
| Approve artifact | `sdlc artifact approve <slug> <type>` |
| Reject artifact | `sdlc artifact reject <slug> <type>` |
| Merge (release feature) | `sdlc merge <slug>` |
| Add task | `sdlc task add <slug> "title"` |
| Start task | `sdlc task start <slug> <task-id>` |
| Complete task | `sdlc task complete <slug> <task-id>` |
| Block task | `sdlc task block <slug> <task-id> "reason"` |
| Add comment | `sdlc comment create <slug> "body"` |
| Show feature | `sdlc feature show <slug> --json` |
| List tasks | `sdlc task list <slug>` |
| Project state | `sdlc state` |
| Survey milestone waves | `sdlc project prepare [--milestone <slug>]` |
| Mark milestone prepared | `sdlc milestone mark-prepared <slug>` |
| Project phase | `sdlc project status` |
| Escalate to human | `sdlc escalate create --kind <kind> --title "…" --context "…" [--feature <slug>]` |
| List escalations | `sdlc escalate list` |
| Resolve escalation | `sdlc escalate resolve <id> "resolution note"` |

Phases advance automatically from artifact approvals — never call `sdlc feature transition`.
The only files you write directly are Markdown artifacts to `output_path`.

## 7. SDLC Tool Suite

Project-scoped TypeScript tools in `.sdlc/tools/` — callable by agents and humans during any lifecycle phase.
Read `.sdlc/tools/tools.md` for the full list, or each tool's `README.md` for detailed docs.

| Tool | Command | Purpose |
|---|---|---|
| ama | `sdlc tool run ama --setup` then `sdlc tool run ama --question "..."` | Search codebase for relevant file excerpts |

Build a custom tool: `sdlc tool scaffold <name> "<description>"`
Update the manifest after adding/changing tools: `sdlc tool sync`

## 8. Project Secrets

Encrypted secrets live in `.sdlc/secrets/`. The encrypted files (`.age`) and key
name sidecars (`.meta.yaml`) are **safe to commit**. Plain `.env.*` files must never
be committed — they are gitignored automatically.

| Action | Command |
|---|---|
| List environments | `sdlc secrets env list` |
| List key names (no decrypt) | `sdlc secrets env names <env>` |
| Load secrets into shell | `eval $(sdlc secrets env export <env>)` |
| Set a secret | `sdlc secrets env set <env> KEY=value` |
| List authorized keys | `sdlc secrets keys list` |
| Add a key | `sdlc secrets keys add --name <n> --key "$(cat ~/.ssh/id_ed25519.pub)"` |
| Rekey after key change | `sdlc secrets keys rekey` |

**For agents:** Check `sdlc secrets env names <env>` to see which variables are
available. Load the matching env before any task or build step that needs credentials:
- Feature/local work → `eval $(sdlc secrets env export development)`
- Deploy tasks → `eval $(sdlc secrets env export production)`

Never log or hardcode secret values. Reference by env var name only (e.g. `$ANTHROPIC_API_KEY`).

**In builds:** The vault is for local and agent use only. CI/CD platforms (GitHub Actions,
etc.) manage their own secrets separately — agents cannot inject into platform CI secrets.
If a build needs a credential that must live in CI, use `secret_request` escalation (§9).

## 9. Escalating to the Human

Escalations are for **actions only a human can take**. They are rare and deliberate — not a
general-purpose communication channel. Before escalating, ask: "Can I resolve this myself?"
If yes, do it. If not, escalate.

| Kind | When to escalate | Example |
|---|---|---|
| `secret_request` | Need a credential or env var that doesn't exist | "Add STRIPE_API_KEY to production env in Secrets page" |
| `question` | Strategic decision with no clear right answer | "Should checkout support crypto payments?" |
| `vision` | Product direction is undefined or contradictory | "No vision defined — what is the milestone goal?" |
| `manual_test` | Testing requires physical interaction | "Verify Google OAuth login in production browser" |

**Do NOT escalate:** code review findings, spec ambiguity you can resolve, implementation
decisions, anything an agent can handle autonomously.

**How to escalate:**

```bash
sdlc escalate create \
  --kind secret_request \
  --title "Need OPENAI_API_KEY in .env.production" \
  --context "AI summary feature calls OpenAI in prod. Dev works with a mock. Need the real key to test end-to-end." \
  --feature my-ai-feature   # omit if not feature-specific
```

**After creating:** stop the current run immediately. If `--feature` was specified, the feature
is now gated by an auto-added Blocker comment. The escalation appears in the Dashboard under
**"Needs Your Attention"**. The human must act before the feature can proceed.

**The difference from `comment --flag blocker`:**

- `comment --flag blocker` — an implementation concern the next agent cycle might fix
- `sdlc escalate create` — an action only a human can perform; stop until resolved

## 10. Frontend API Calls

Never hardcode `http://localhost:PORT` in frontend code — CORS blocks cross-origin
requests in development and the address is wrong in production.

**Pattern:**
- Use a relative base URL (`/api`) in all fetch/client code
- Configure the dev server proxy (Vite `server.proxy`, Next.js `rewrites`,
  webpack `devServer.proxy`) to forward `/api` → `http://localhost:<API_PORT>`
- In production, frontend and API share the same origin — relative paths resolve correctly

When fixing a CORS error or adding a new API client, apply this pattern instead of
adding CORS headers or introducing environment-specific URLs.
"#;

const SDLC_NEXT_COMMAND: &str = r#"---
description: Get the next directive for a feature and act on it
argument-hint: <feature-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-next

Read the next directive for a feature and act on it. This is the primary entry point for driving features forward.

## What is sdlc?

`sdlc` is a project management state machine. It tracks features through a lifecycle:

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```

Every phase requires specific Markdown artifacts to be written and approved before advancing.
`sdlc next --json` tells you exactly what to do next. You act on it, submit the artifact, and the phase advances.

## Steps

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

### 1. Resolve the slug

Get `<feature-slug>` from $ARGUMENTS. If none provided:
```bash
sdlc next
```
Show the output and ask the user which feature to drive.

### 2. Get the directive

```bash
sdlc next --for <slug> --json
```

Key fields: `action`, `message`, `output_path`, `current_phase`, `is_heavy`, `gates`.

### 3. Handle `done`

> "All SDLC phases complete for '[slug]'."

### 4. Execute the directive

For **artifact creation** (`create_spec`, `create_design`, `create_tasks`, `create_qa_plan`, `create_review`, `create_audit`):
1. Run `sdlc feature show <slug> --json` for context
2. Read existing artifacts in `.sdlc/features/<slug>/`
3. Write a thorough Markdown artifact to `output_path`

For **approval** (`approve_spec`, `approve_design`, `approve_tasks`, `approve_qa_plan`, `approve_review`, `approve_audit`, `approve_merge`):
1. Read the artifact at `output_path`, verify it is complete and correct
2. Run `sdlc artifact approve <slug> <artifact_type>` autonomously — no confirmation needed

For **implementation** (`implement_task`):
1. Run `sdlc task list <slug>` to find the next pending task
2. Read design and tasks artifacts for context
3. Implement the task, then run `sdlc task complete <slug> <task-id>`

For **merge** (`merge`):
```bash
sdlc merge <slug> --json
```
This transitions the feature to `released`. Execute immediately — no confirmation needed.

For **gates** (`wait_for_approval`, `unblock_dependency`):
Stop and report clearly. These require human intervention before the feature can advance.

### 5. Show updated state

```bash
sdlc next --for <slug>
```
"#;

const SDLC_STATUS_COMMAND: &str = r#"---
description: Show SDLC state for the project or a specific feature
argument-hint: [feature-slug]
allowed-tools: Bash
---

# sdlc-status

Show the current SDLC state — what features exist, what phase they're in, and what needs attention.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Usage

```
/sdlc-status              → project-wide overview
/sdlc-status <slug>       → detailed view of one feature
```

## Project-wide overview

```bash
sdlc state
sdlc next
sdlc query needs-approval
sdlc query blocked
sdlc query ready
sdlc ponder list
```

Show features grouped by: needs approval, blocked, in progress, ready. Include active ponder entries (exploring/converging ideas).

## Single-feature detail

```bash
sdlc feature show <slug>
sdlc next --for <slug>
sdlc task list <slug>
sdlc comment list <slug>
```

Show phase, artifact status, open tasks, comments, and the next action.

## Lifecycle

```
draft → specified → planned → ready → implementation → review → audit → qa → merge → released
```
"#;

const SDLC_APPROVE_COMMAND: &str = r#"---
description: Review and approve an sdlc artifact to advance the feature phase
argument-hint: <feature-slug> <artifact-type>
allowed-tools: Bash, Read
---

# sdlc-approve

Read an artifact, present it for review, and approve it to advance the feature to the next phase.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Usage

```
/sdlc-approve <feature-slug> <artifact-type>
```

Artifact types: `spec` `design` `tasks` `qa_plan` `review` `audit` `qa_results`

## Steps

1. **Resolve args** from $ARGUMENTS. If missing, run `sdlc query needs-approval`.

2. **Read the artifact:**
   ```
   .sdlc/features/<slug>/<type>.md
   ```

3. **Present to user.** Ask: "Approve this [type] for '[slug]'?"

4. **Do NOT approve without explicit user confirmation.**

5. **On approval:**
   ```bash
   sdlc artifact approve <slug> <type>
   ```

6. **On rejection:**
   ```bash
   sdlc artifact reject <slug> <type>
   sdlc comment create <slug> "<feedback>"
   ```

7. **Show what comes next:**
   ```bash
   sdlc next --for <slug>
   ```
"#;

const SDLC_SPECIALIZE_COMMAND: &str = r#"---
description: Survey this project and generate a tailored AI team (Claude agents + skills)
argument-hint: [project-description]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-specialize

Generate a project-specific AI team — Claude agent personas and blueprint-style skills —
tailored to this project's tech stack, domain, and team roles. Runs across 4 sessions with
explicit human checkpoints so you approve each stage before files are written.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Overview

This command produces:
- **`.claude/agents/<first-last>.md`** — persona agents with YAML frontmatter, background,
  Principles, This Codebase, ALWAYS/NEVER rules, and When Stuck section
- **`.claude/skills/<domain-role>/SKILL.md`** — blueprint skills with Quick Reference table,
  Phase 1–4 workflow, Step Back challenges, and Done Gate checklist

---

## Session 1: Survey the Project

Read the project to understand its domain, tech stack, and current state:

```bash
sdlc state
```

Then read (if present):
- `VISION.md` or `docs/vision.md`
- `docs/architecture.md` or `ARCHITECTURE.md`
- `AGENTS.md` or `CLAUDE.md`
- `README.md`
- Root config files (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`, etc.)
- Key source directories (list top-level dirs)

Summarize:
1. **Project purpose** — what does it do in one sentence?
2. **Tech stack** — languages, frameworks, key libraries
3. **Domain areas** — e.g., "CLI tooling", "API layer", "frontend", "data pipeline"
4. **Current SDLC phase** — active features, milestones, maturity

### ✋ Gate 1: Confirm Understanding

Present your summary to the user:

> "Here's what I found about [project]:
> - Purpose: ...
> - Stack: ...
> - Domain areas: ...
> - Current state: ...
>
> Does this look right before I design the team?"

**Wait for explicit user confirmation before proceeding.**

---

## Session 2: Design the Team Roster

Design 3–5 specialist roles that match the project's actual domain areas.

For each role, define:
- **Persona name** (first-last, e.g., `alex-chen`) — a real-sounding human name
- **Role title** — e.g., "API Engineer", "Frontend Builder", "Data Pipeline Architect"
- **Domain ownership** — which files/directories/subsystems they own
- **Model assignment** — `opus` for architects/heavy reasoners, `sonnet` for implementers
- **Color** — pick from: `orange`, `blue`, `green`, `purple`, `red`, `yellow`, `cyan`
- **Skill name** — kebab-case domain slug, e.g., `api-engineer`, `frontend-builder`

Present as a table:

| Name | Role | Domain | Model | Color | Skill |
|------|------|--------|-------|-------|-------|
| alex-chen | API Engineer | `src/api/`, `src/routes/` | sonnet | blue | api-engineer |
| ... | ... | ... | ... | ... | ... |

### ✋ Gate 2: Approve the Roster

> "Here's the proposed team roster for [project]. Does this look right?
> Any roles to add, remove, or rename before I generate the files?"

**Wait for explicit user approval. Do NOT write any files before this gate.**

---

## Session 3: Generate Agents and Skills

For each approved roster entry, generate two files.

### Agent format: `.claude/agents/<first-last>.md`

```markdown
---
name: <first-last>
description: Use when <specific domain triggers>. Examples — "<example 1>", "<example 2>", "<example 3>".
model: <opus|sonnet|haiku>
color: <color>
---

You are <Full Name>, <background paragraph — 3-4 sentences describing their career history at named companies/projects, their area of expertise, and their deeply held technical philosophy. Be specific and concrete, not generic>.

## Your Principles

- **<Principle name>.** <One sentence explanation of why this matters in this codebase>.
- **<Principle name>.** <One sentence explanation>.
- (3–5 principles total)

## This Codebase

**<Area 1>:**
- `path/to/file.ext` — brief description of what it does
- `path/to/dir/` — brief description

**<Area 2>:**
- `path/to/file.ext` — brief description
(cover 2–4 domain areas with the actual file paths from the project)

## ALWAYS

- <concrete, specific rule about this codebase — not generic advice>
- <specific rule>
- (3–6 rules)

## NEVER

- <concrete anti-pattern specific to this domain>
- <specific anti-pattern>
- (3–6 rules)

## When You're Stuck

1. **<Common failure mode>:** <Specific debugging approach with actual commands or file paths>.
2. **<Common failure mode>:** <Specific approach>.
3. (2–4 entries)
```

### Skill format: `.claude/skills/<domain-role>/SKILL.md`

```markdown
---
name: <domain-role>
description: Use when <triggers>. Delegate to **<Full Name>** for implementation.
---

# <Role Title>

You are a <domain> specialist. Delegate to **<Full Name>** for implementation.

## Principles

1. **<Principle>.** <Explanation>.
2. **<Principle>.** <Explanation>.
(3–5 principles)

## Quick Reference

| Area | Path | Notes |
|------|------|-------|
| <area> | `<actual/path>` | <note> |
(use real paths from the project)

## Phase 1: Understand the Change

Before writing any code, read:
1. <specific files relevant to this domain>
2. <related interface/type files>
3. <test patterns in use>

State: what is being added/changed and which layer it lives in.

## Phase 2: Design the Interface

<Domain-specific interface design guidance — types, APIs, contracts>

## Phase 3: Implement

Delegate to **<Full Name>** for the implementation. Work in this order:
1. <domain-specific implementation order>
2. ...

## Step Back: Challenge Before Committing

Before finalizing the implementation, ask:

### 1. <First challenge question for this domain>
> "<Challenge prompt>"
- <specific constraint to check>

### 2. <Second challenge question>
> "<Challenge prompt>"
- <specific constraint to check>

(2–4 challenges relevant to this domain)

## Phase 4: Verify

```bash
<actual quality commands for this project's stack>
```

## Done Gate

- [ ] <Specific completion criterion for this domain>
- [ ] <Specific criterion>
- [ ] All tests pass
- [ ] <Stack-specific quality check passes>
```

Write all agents to `.claude/agents/` and all skills to `.claude/skills/`.

---

## Session 4: Update AGENTS.md

Add a `## Team` section to `AGENTS.md` (or update if it exists) listing each agent and their domain:

```markdown
## Team

| Agent | Role | Domain | Invoke When |
|-------|------|--------|-------------|
| @<first-last> | <Role> | <Domain> | <When to use> |
```

### ✋ Gate 4: Final Confirmation

List all files created:
```
Created:
  .claude/agents/alex-chen.md
  .claude/agents/...
  .claude/skills/api-engineer/SKILL.md
  .claude/skills/.../SKILL.md
  AGENTS.md (updated Team section)
```

> "All done. Your project now has a tailored AI team. Use `/sdlc-next` to drive features forward
> with these specialists, or invoke agents directly by name in Claude Code."
"#;

const SDLC_NEXT_PLAYBOOK: &str = r#"# sdlc-next

Use this playbook to drive the next SDLC directive for a feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> **NEVER edit `.sdlc/` YAML files directly.** All state changes go through `sdlc` CLI commands. The only files you write are Markdown artifacts to the `output_path` from `sdlc next --json`.

## Steps

1. Resolve the slug.
   - If one is not provided, run `sdlc next` and pick a feature.
2. Run `sdlc next --for <slug> --json`.
3. Parse directive fields: `action`, `message`, `output_path`, `current_phase`, `is_heavy`, `gates`.
4. For creation actions:
   - Read feature context and existing artifacts.
   - Write the required artifact to `output_path`.
   - Mark it draft with `sdlc artifact draft <slug> <artifact_type>`.
5. For approval actions (`approve_spec`, `approve_design`, `approve_review`, `approve_merge`):
   - Read the artifact at `output_path`, verify it is complete and correct.
   - Run `sdlc artifact approve <slug> <artifact_type>` autonomously.
6. For implementation:
   - Run `sdlc task list <slug>`.
   - Implement the next task and run `sdlc task complete <slug> <task_id>`.
7. Run `sdlc next --for <slug>` to show updated state.
"#;

const SDLC_STATUS_PLAYBOOK: &str = r#"# sdlc-status

Use this playbook to report SDLC state for the whole project or one feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Project view

Run:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`
- `sdlc ponder list`

## Feature view

Run:
- `sdlc feature show <slug>`
- `sdlc next --for <slug>`
- `sdlc task list <slug>`
- `sdlc comment list <slug>`
"#;

const SDLC_APPROVE_PLAYBOOK: &str = r#"# sdlc-approve

Use this playbook to review and approve an SDLC artifact.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve `<slug>` and `<artifact_type>`.
2. Read the artifact file in `.sdlc/features/<slug>/`:
   - `spec` -> `spec.md`
   - `design` -> `design.md`
   - `tasks` -> `tasks.md`
   - `qa_plan` -> `qa-plan.md`
   - `review` -> `review.md`
   - `audit` -> `audit.md`
   - `qa_results` -> `qa-results.md`
3. Present the artifact for review.
4. Only proceed after explicit user approval.
5. On approval: `sdlc artifact approve <slug> <artifact_type>`.
6. On rejection: `sdlc artifact reject <slug> <artifact_type>` and add a comment.
7. Run `sdlc next --for <slug>` to show what is next.
"#;

const SDLC_NEXT_SKILL: &str = r#"---
name: sdlc-next
description: Get the next SDLC directive for a feature and act on it. Use when driving a feature forward one step at a time.
---

# SDLC Next Skill

Use this skill when a user asks for the next SDLC action for a feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Resolve the feature slug.
2. Run `sdlc next --for <slug> --json`.
3. Follow the directive fields (`action`, `message`, `output_path`, `gates`).
4. For approval or dependency gates, surface context and wait for explicit user approval.
5. For creation actions, write the requested artifact at `output_path`.
6. For implementation actions, complete the next pending task.
7. Run `sdlc next --for <slug>` to confirm what comes next.
"#;

const SDLC_STATUS_SKILL: &str = r#"---
name: sdlc-status
description: Show SDLC state for the project or a specific feature. Use when checking progress, blockers, or next actions.
---

# SDLC Status Skill

Use this skill when a user asks for SDLC status across the project or for one feature.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

Project view:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`
- `sdlc ponder list`

Feature view:
- `sdlc feature show <slug>`
- `sdlc next --for <slug>`
- `sdlc task list <slug>`
- `sdlc comment list <slug>`
"#;

const SDLC_APPROVE_SKILL: &str = r#"---
name: sdlc-approve
description: Review and approve an SDLC artifact to advance the feature phase. Use when verifying specs, designs, tasks, reviews, or audits.
---

# SDLC Approve Skill

Use this skill when a user wants to review and approve an SDLC artifact.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve `<slug>` and `<artifact_type>`.
2. Read the artifact under `.sdlc/features/<slug>/`.
3. Present key findings to the user for explicit approval.
4. On approval, run `sdlc artifact approve <slug> <artifact_type>`.
5. On rejection, run `sdlc artifact reject <slug> <artifact_type>` and capture feedback.
6. Run `sdlc next --for <slug>` to show the updated directive.
"#;

// ---------------------------------------------------------------------------
// sdlc-run — Claude command
// ---------------------------------------------------------------------------

const SDLC_RUN_COMMAND: &str = r#"---
description: Autonomously drive a feature to completion
argument-hint: <feature-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-run

Drive a feature forward autonomously — executing every action in the state machine loop until the feature is done.

Use `sdlc-next` when you want to execute one step at a time.
Use `sdlc-run` when you want the agent to drive the feature to completion.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

---

## Steps

### 1. Resolve the slug

Get `<feature-slug>` from $ARGUMENTS. If none:
```bash
sdlc next
```
Ask the user which feature to run.

### 2. Confirm scope (if `is_heavy`)

Before starting, show the current phase and what actions will be executed:
```bash
sdlc feature show <slug>
sdlc next --for <slug> --json
```

If any upcoming actions are `is_heavy` (implement_task, fix_review_issues, run_qa), tell the user:
> "This run includes heavy actions (implementation/QA). Proceeding autonomously."

### 3. Run the loop

Repeat until `done`:

```
directive = sdlc next --for <slug> --json

if action == done        → report completion, exit
otherwise                → execute the action (see sdlc-next for action handlers)
                         → loop
```

Execute each action exactly as documented in `sdlc-next`. Do not skip steps or compress multiple actions into one pass — each action advances the state machine and must complete before the next is evaluated.

> **Never call `sdlc feature transition` directly.** Phases advance automatically when artifacts are approved. If a transition isn't happening, an artifact is missing a `draft` or `approve` call — re-check with `sdlc next --for <slug> --json`.

### 4. On unexpected failure

If an action fails in a way that cannot be recovered automatically, stop and report:
1. What action failed
2. What was attempted
3. What the human needs to resolve

Do not loop indefinitely on a failing action.

### 5. On completion

```bash
sdlc feature show <slug>
```

Report the final phase and a summary of everything accomplished.

---

### 6. Next

Always end with a single `**Next:**` line:

| Outcome | Next |
|---|---|
| Feature `done`, milestone has more work | `**Next:** /sdlc-prepare <milestone-slug>` |
| Feature `done`, milestone all released | `**Next:** /sdlc-milestone-uat <milestone-slug>` |
| Feature `done`, no milestone | `**Next:** /sdlc-prepare` |
| Unexpected failure | `**Next:** /sdlc-run <slug>` _(after fixing the blocker)_ |
"#;

// ---------------------------------------------------------------------------
// sdlc-plan — Claude command
// ---------------------------------------------------------------------------

const SDLC_PLAN_COMMAND: &str = r#"---
description: Distribute a plan — week-by-week brief, task dump, or design doc — into sdlc milestones, features, and tasks. Idempotent: re-running refines what exists, never duplicates.
argument-hint: [--file <path>] or paste plan content inline
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-plan

Takes a body of work and distributes it into the sdlc structure. Creates milestones, features, and tasks where they don't exist. Refines them where they do. Running it again with a tweaked plan is safe and correct — the second run adjusts, not piles on.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Idempotency Contract

This is the most important property of this command. Every operation must be safe to repeat:

- **Milestones** — slug derived deterministically from the plan. If the slug already exists: update title, vision, and acceptance test. Never create a duplicate.
- **Features** — slug derived deterministically. If the slug already exists: update title and description. Never create a duplicate.
- **Milestone↔Feature links** — `sdlc milestone add-feature` is already idempotent. Run it unconditionally.
- **Tasks** — before adding, search for an existing task with a matching title in that feature. If found: skip. Never duplicate tasks.

Slug derivation must be deterministic: same plan text → same slugs every time. Lowercase, spaces → hyphens, strip punctuation, max 40 chars.

---

## Phase 1: Load Current State (parallel)

Run both simultaneously:

```bash
sdlc milestone list --json
sdlc feature list --json
```

Build a registry of existing milestones and features.

---

## Phase 2: Parse and Map

Read the plan. Produce a structured mapping before touching anything. Print it.

### What becomes a milestone
A milestone is a coherent unit of delivery with a user-observable goal, verifiable deliverables, and multiple related features.

### What becomes a feature
A feature is a semantically cohesive unit that ships together as one observable capability. Group related task-list items into one feature.

### What becomes a task
Individual implementation steps within a feature.

### Vision derivation
Synthesize the milestone goals into one sentence: what can a user do when this ships?

### Acceptance test derivation
Convert deliverables to a `- [ ]` checklist. Write it as `/tmp/<slug>_acceptance_test.md`.

---

## Phase 3: Execute (parallel agents per milestone)

Spawn one agent per milestone. Each agent:

### Step 1: Create or update the milestone
```bash
sdlc milestone create <slug> --title "<title>"
sdlc milestone update <slug> --vision "<derived vision>"
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md
```

### Step 2: For each feature (sequential within agent)
```bash
sdlc feature create <slug> --title "<title>" --description "<description>"
sdlc milestone add-feature <milestone-slug> <feature-slug>
```

### Step 3: For each task in the feature
Check for duplicates with `sdlc task search`, then:
```bash
sdlc task add <feature-slug> "<title>"
```

---

## Phase 4: Summary

Print results: milestones created/updated, features created/updated, tasks added/skipped.

**Next:** `/sdlc-focus`

---

## Notes

- Features that exist but aren't in any milestone are re-linked to the correct milestone.
- If the plan has no explicit structure, derive milestone boundaries from semantic groupings.
- Lean toward fewer, larger milestones. A milestone should ship something a user can experience.
"#;

// ---------------------------------------------------------------------------
// sdlc-milestone-uat — Claude command
// ---------------------------------------------------------------------------

const SDLC_MILESTONE_UAT_COMMAND: &str = r#"---
description: Run the acceptance test for a milestone — execute every step, sign the checklist, fix issues or create tasks, never pause
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-milestone-uat

Run a milestone's acceptance test end-to-end. Executes every checklist item, signs each one as it passes, and for failures either fixes immediately, creates a task and continues, or creates a task and halts. Writes a signed `uat_results.md` to the milestone directory.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Be the user.** UAT means running the product as a real user would — start the server, open the UI, run the CLI commands, follow the flow. Not reading code. Actually doing it.
- **Never pause.** Decide and act on every failure without asking.
- **Always forward.** Create tasks for issues; never revert state.
- **Everything in git.** `uat_results.md` is committed alongside the code it validates.

---

## Steps

### 1. Load the milestone

```bash
sdlc milestone info <slug> --json
```

Extract `title`, `vision`, and `acceptance_test` content. If no acceptance test, stop.

### 2. Parse checklist items

Collect every `- [ ]` line as an ordered list of steps.

### 3. Execute each step

For each checklist item:
- **Execute** the command or check described
- **Evaluate** actual output against expected outcome
- **Track** result: PASS → `[x]` with timestamp, FAIL → assess severity

### 4. Failure response

#### FIX immediately
Localized, low blast radius, completable in this session. Fix, re-run, record as `[x] (fixed: <what changed>)`.

#### TASK + CONTINUE
Real issue but doesn't block remaining steps. Create a task on the feature that owns the failing behavior, record as failed, continue:

```bash
sdlc task add <feature-slug> "<one-line description of the failure>"
```

Where `<feature-slug>` is the feature in the milestone responsible for the broken behavior. If multiple features could own it, pick the most relevant. If genuinely ambiguous, pick the first feature in the milestone.

#### TASK + HALT
Failure makes remaining steps meaningless. Create a task (same `sdlc task add <feature-slug> "..."` pattern), record as failed, stop execution.

### 5. Write `uat_results.md`

Write signed checklist to `.sdlc/milestones/<slug>/uat_results.md`:

```markdown
# UAT Run — <milestone-title>
**Date:** <ISO-8601 timestamp>
**Agent:** <model identifier>
**Verdict:** PASS | PASS WITH TASKS | FAILED

---

- [x] <step text> _(<timestamp>)_
- [x] <step text> _(fixed: <what changed> · <timestamp>)_
- [ ] ~~<step text>~~ _(✗ task <feature>#<id> — <one-line reason>)_

---

**Tasks created:** <feature>#<id>, ...
**<N>/<total> steps passed**
```

### 6. Flip milestone state

**On PASS or PASS WITH TASKS:** mark the milestone Released immediately after writing results:

```bash
sdlc milestone complete <slug>
```

This sets `released_at` and transitions the milestone from `Verifying` → `Released`. For PASS WITH TASKS the milestone is still Released — the outstanding tasks are tracked inside their features and addressed in a future cycle.

**On FAILED:** do NOT call `milestone complete`. The milestone stays in `Verifying`. The tasks created in Step 4 are already attached to the appropriate features. The agent will drive those features to fix the gaps, then re-run this UAT.

### 7. Final report

| Verdict | State after | Next |
|---|---|---|
| PASS | `Released` | Commit `uat_results.md` |
| PASS WITH TASKS | `Released` | Commit `uat_results.md` |
| FAILED | `Verifying` (unchanged) | `/sdlc-run <first-blocking-feature-slug>` — fix, then re-run `/sdlc-milestone-uat <slug>` |

Always end output with exactly one **Next:** line showing the command to run.
"#;

// ---------------------------------------------------------------------------
// sdlc-pressure-test — Claude command
// ---------------------------------------------------------------------------

const SDLC_PRESSURE_TEST_COMMAND: &str = r#"---
description: Pressure-test a milestone against user perspectives — are we building what users actually want? Autonomously edits vision, features, acceptance tests, and creates tasks for gaps.
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-pressure-test

Pressure-test a milestone's direction against real user perspectives. This is not a code review or quality gate — it's a "are we solving the right problem?" check. Runs empathy interviews, identifies gaps between what's planned and what users need, and autonomously edits project docs to align them.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## When to Use

- Before starting implementation on a milestone (ideal)
- When a milestone feels off but you can't articulate why
- After a UAT failure that suggests we built the wrong thing
- When the team is building features no one asked for

## Ethos

- **Users over builders.** What we want to build matters less than what users need.
- **Edit, don't report.** This command produces changes, not a report that sits unread.
- **Conflicts are gold.** When user perspectives disagree with what's planned, that's the most valuable signal.
- **Always forward.** We add tasks, sharpen descriptions, and adjust acceptance criteria. The state machine moves forward.

---

## Steps

### 1. Load the milestone

```bash
sdlc milestone info <slug> --json
```

Extract title, vision, features, acceptance_test. If vision is empty, note as critical gap.

### 2. Load all features in the milestone

For each feature slug:
```bash
sdlc feature show <feature-slug>
```

Build a map of titles, descriptions, phases, existing specs, and tasks.

### 3. Identify user perspectives

Identify 3-5 specific user personas. Not abstract "users" — specific people in specific situations.

**Always include:**
1. The primary user (hands on keyboard daily)
2. Someone affected indirectly (downstream, ops, support)
3. A skeptic or reluctant adopter
4. A new/first-time user encountering this for the first time

### 4. Run empathy interviews (parallel)

For each perspective, conduct a deep interview:
- **Context**: typical day interacting with what this milestone delivers
- **Needs**: what problem it solves, what success looks like
- **Friction**: what would cause frustration or abandonment
- **Gaps**: what's missing from the planned features
- **Acceptance**: how they would test whether it delivers value

### 5. Synthesize findings

Analyze: alignments, conflicts, gaps, overbuilding, acceptance gaps.

### 6. Autonomous edits

#### A. Sharpen milestone vision
```bash
sdlc milestone update <slug> --vision "<sharpened vision>"
```

#### B. Update feature descriptions
```bash
sdlc feature update <feature-slug> --description "<user-aligned description>"
```

#### C. Add missing features
```bash
sdlc feature create <new-slug> --title "<title>" --description "<description>"
sdlc milestone add-feature <milestone-slug> <new-slug>
```

#### D. Create tasks for gaps
```bash
sdlc task add <feature-slug> --title "[user-gap] <specific gap to address>"
```

#### E. Update acceptance test
```bash
sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md
```

### 7. Commit and report

Print the pressure test report with perspectives consulted, edits made, conflicts surfaced, and overbuilding warnings.

---

### 8. Next

| Outcome | Next |
|---|---|
| Edits made, features in draft | `**Next:** /sdlc-run <first-feature-slug>` |
| New features created | `**Next:** /sdlc-run <new-feature-slug>` |
| Major direction change needed | `**Next:** /sdlc-plan` with revised plan |
| Milestone well-aligned | `**Next:** /sdlc-milestone-uat <slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-enterprise-readiness — Claude command
// ---------------------------------------------------------------------------

const SDLC_ENTERPRISE_READINESS_COMMAND: &str = r#"---
description: Analyze a project for enterprise-grade production readiness and distribute findings into sdlc milestones, features, and tasks via sdlc-plan — or add to existing milestones and update active tasks
argument-hint: [--stage <mvp|production|scale|enterprise>] [--into <milestone-slug>]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-enterprise-readiness

Run an enterprise readiness analysis against the current project and translate findings into sdlc structure. The output is not a report — it's milestones, features, and tasks that enter the state machine and get built.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

Three modes of operation:

1. **Create new milestones** (default) — groups findings into `ops-*` milestones and feeds them through sdlc-plan
2. **Add to existing milestone** (`--into <slug>`) — adds findings as features/tasks to an existing milestone
3. **Update active tasks** — when findings overlap with in-progress features, adds `[ops-gap]` tasks

## Ethos

- **Gaps become milestones, not reports.** Every finding either becomes a feature or gets explicitly deferred with rationale.
- **Build for the next stage, not three ahead.** MVP projects don't need multi-region. Scope to what matters now.
- **Blast radius drives priority.** A missing timeout can crash the service (P0). A missing Grafana panel is annoying (P3).

---

## Steps

### 1. Determine current and target stage

Parse `$ARGUMENTS` for `--stage`. If not provided, infer from project signals.

| Stage | Signals | Next Stage |
|---|---|---|
| **MVP Pilot** | No CI/CD, no monitoring, manual deploys | Production |
| **Production** | CI/CD exists, basic health checks, some docs | Scale |
| **Scale** | Monitoring, automated ops, multi-customer | Enterprise |
| **Enterprise** | Compliance artifacts, DR runbooks, SLAs | Maintenance |

### 2. Load existing sdlc state

```bash
sdlc milestone list --json
sdlc feature list --json
```

### 3. Run enterprise readiness analysis

Launch expert agents in parallel:
- **A. Storage/Data** — backup, recovery, data integrity, persistence
- **B. Operations** — deployment, monitoring, alerting, runbooks
- **C. Security** — TLS, auth, rate limiting, secrets management

### 4. Route findings based on mode

#### Mode A: Add to existing milestone (`--into <slug>`)
Add `[ops-gap]` tasks to existing features or create new features within the milestone.

#### Mode B: Update active tasks (automatic)
Scan existing features for overlap and add `[ops-gap]` tasks where applicable.

#### Mode C: Create new milestones (default)
Group remaining gaps into `ops-*` milestones (ops-ship-blockers, ops-production-hardening, etc.).

### 5. Synthesize remaining gaps into a plan

Assign priority (P0-P3), group into milestones, decompose into features and tasks.

### 7. Produce the plan document (Mode C only)

Write to `/tmp/enterprise-readiness-plan.md` and feed through `/sdlc-plan`.

### 9. Report

Print enterprise readiness report with distributed gaps, absorbed tasks, deferred items, and expert consensus.

---

### 10. Next

| Outcome | Next |
|---|---|
| Ship blockers created (Mode C) | `**Next:** /sdlc-run <first-ops-feature-slug>` |
| Added to milestone (Mode A) | `**Next:** /sdlc-run <first-new-feature-slug>` |
| Tasks added to active features (Mode B) | `**Next:** /sdlc-status` |
| Already enterprise-ready | `**Next:** /sdlc-status` |
"#;

// ---------------------------------------------------------------------------
// Playbooks (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_RUN_PLAYBOOK: &str = r#"# sdlc-run

Use this playbook to autonomously drive a feature to completion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Steps

1. Resolve the feature slug. If not provided, run `sdlc next` and pick a feature.
2. Run `sdlc feature show <slug>` and `sdlc next --for <slug> --json` to assess scope.
3. Enter the loop:
   a. Run `sdlc next --for <slug> --json`.
   b. If `action == done` → report completion, exit.
   c. Otherwise → execute the action per sdlc-next protocol, then loop.
4. For each action, execute exactly as documented — write artifacts, implement tasks, run approvals.
5. Never call `sdlc feature transition` directly — phases advance from artifact approvals.
6. On unexpected failure, stop and report what failed and what needs resolving.
"#;

const SDLC_PLAN_PLAYBOOK: &str = r#"# sdlc-plan

Use this playbook to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Steps

1. Load current state: `sdlc milestone list --json` and `sdlc feature list --json`.
2. Parse the plan and produce a structured mapping (milestones → features → tasks).
3. For each milestone:
   a. Create or update: `sdlc milestone create <slug> --title "<title>"`.
   b. Set vision: `sdlc milestone update <slug> --vision "<vision>"`.
   c. Set acceptance test: `sdlc milestone set-acceptance-test <slug> --file /tmp/<slug>_acceptance_test.md`.
4. For each feature:
   a. Create or update: `sdlc feature create <slug> --title "<title>" --description "<desc>"`.
   b. Link: `sdlc milestone add-feature <milestone-slug> <feature-slug>`.
5. For each task: check for duplicates with `sdlc task search`, then `sdlc task add`.
6. Report: milestones created/updated, features created/updated, tasks added/skipped.

## Key Rules

- Idempotent: re-running refines, never duplicates.
- Slug derivation must be deterministic (lowercase, hyphens, max 40 chars).
- Group related items into cohesive features — don't make every line item a feature.
"#;

const SDLC_MILESTONE_UAT_PLAYBOOK: &str = r#"# sdlc-milestone-uat

Use this playbook to run a milestone's acceptance test end-to-end.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`. Extract acceptance_test.
2. Parse every `- [ ]` line as an ordered list of test steps.
3. Execute each step as a real user would (run commands, check output).
4. For each step:
   - PASS → record `[x]` with timestamp.
   - FAIL (fixable) → fix inline, re-run, record as `[x] (fixed: <what>)`.
   - FAIL (non-blocking) → create task with `sdlc task add`, continue.
   - FAIL (blocking) → create task, halt execution.
5. Write `uat_results.md` to `.sdlc/milestones/<slug>/uat_results.md`.
6. Report verdict: PASS, PASS WITH TASKS, or FAILED.

## Key Rules

- Be the user: run the product, don't read code.
- Never pause to ask — decide and act on every failure.
- Always forward: create tasks for issues, never revert state.
"#;

const SDLC_PRESSURE_TEST_PLAYBOOK: &str = r#"# sdlc-pressure-test

Use this playbook to pressure-test a milestone against user perspectives.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 3-5 specific user personas (primary, indirect, skeptic, first-time).
4. Run empathy interviews for each perspective (context, needs, friction, gaps, acceptance).
5. Synthesize findings: alignments, conflicts, gaps, overbuilding, acceptance gaps.
6. Make autonomous edits:
   a. Sharpen vision: `sdlc milestone update <slug> --vision "<vision>"`.
   b. Update descriptions: `sdlc feature update <slug> --description "<desc>"`.
   c. Add missing features: `sdlc feature create` + `sdlc milestone add-feature`.
   d. Create gap tasks: `sdlc task add <slug> --title "[user-gap] <gap>"`.
   e. Update acceptance test: `sdlc milestone set-acceptance-test`.
7. Report: perspectives consulted, edits made, conflicts surfaced.

## Key Rules

- Users over builders. What we want to build matters less than what users need.
- Edit, don't report. Findings become changes to vision, features, tasks.
- Conflicts are gold. Don't smooth over disagreements — surface them.
"#;

const SDLC_ENTERPRISE_READINESS_PLAYBOOK: &str = r#"# sdlc-enterprise-readiness

Use this playbook to analyze a project for enterprise readiness and distribute findings into sdlc.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Determine current and target stage (MVP → Production → Scale → Enterprise).
2. Load existing state: `sdlc milestone list --json` and `sdlc feature list --json`.
3. Run analysis across three domains: Storage/Data, Operations, Security.
4. Route findings by mode:
   a. `--into <slug>`: add gaps as features/tasks to existing milestone.
   b. Automatic: scan for overlap with in-progress features, add `[ops-gap]` tasks.
   c. Default: group gaps into new `ops-*` milestones (ops-ship-blockers, ops-production-hardening).
5. For new milestones (Mode C): write plan to `/tmp/enterprise-readiness-plan.md`, feed through sdlc-plan.
6. Report: current/target stage, distributed gaps, absorbed tasks, deferred items.

## Key Rules

- Gaps become milestones, not reports.
- Build for the next stage, not three ahead.
- Blast radius drives priority (P0-P3).
"#;

// ---------------------------------------------------------------------------
// Skills (Agents)
// ---------------------------------------------------------------------------

const SDLC_RUN_SKILL: &str = r#"---
name: sdlc-run
description: Autonomously drive a feature to completion. Use when a feature should run unattended through multiple phases.
---

# SDLC Run Skill

Use this skill to autonomously drive a feature through the sdlc state machine to completion.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Resolve the feature slug.
2. Run `sdlc next --for <slug> --json` to get the current directive.
3. Loop: execute action → re-read directive → repeat.
4. Stop only at `done` or unexpected failure.
5. All actions — including approvals and merge — execute autonomously.
6. Never call `sdlc feature transition` directly; phases advance from artifact approvals.
7. On completion, report what was accomplished and what comes next.
"#;

const SDLC_PLAN_SKILL: &str = r#"---
name: sdlc-plan
description: Distribute a plan into sdlc milestones, features, and tasks. Use when decomposing a roadmap or plan into trackable work.
---

# SDLC Plan Skill

Use this skill to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

> Never edit `.sdlc/` YAML directly — see §6 of `.sdlc/guidance.md`.

## Workflow

1. Load current state: `sdlc milestone list --json` and `sdlc feature list --json`.
2. Parse the plan into milestones → features → tasks.
3. Create/update milestones with vision and acceptance tests.
4. Create/update features and link to milestones.
5. Add tasks, checking for duplicates first.
6. Report: counts of created, updated, and skipped items.

## Key Rule

Idempotent — re-running refines what exists, never duplicates.
"#;

const SDLC_MILESTONE_UAT_SKILL: &str = r#"---
name: sdlc-milestone-uat
description: Run the acceptance test for a milestone end-to-end. Use when validating that a milestone meets its definition of done.
---

# SDLC Milestone UAT Skill

Use this skill to run a milestone's acceptance test end-to-end.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone: `sdlc milestone info <slug> --json`.
2. Parse `- [ ]` checklist items from the acceptance test.
3. Execute each step as a real user would.
4. On failure: fix inline, or create a task and continue/halt.
5. Write `uat_results.md` to `.sdlc/milestones/<slug>/`.
6. Report verdict: PASS, PASS WITH TASKS, or FAILED.
"#;

const SDLC_PRESSURE_TEST_SKILL: &str = r#"---
name: sdlc-pressure-test
description: Pressure-test a milestone against user perspectives. Use when validating that a milestone builds what users actually want.
---

# SDLC Pressure Test Skill

Use this skill to pressure-test a milestone against user perspectives.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone and its features from sdlc.
2. Identify 3-5 specific user personas (primary, indirect, skeptic, first-time).
3. Run empathy interviews for each perspective.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Make autonomous edits: sharpen vision, update descriptions, add features, create `[user-gap]` tasks, update acceptance test.
6. Report perspectives consulted, edits made, and conflicts surfaced.
"#;

const SDLC_ENTERPRISE_READINESS_SKILL: &str = r#"---
name: sdlc-enterprise-readiness
description: Analyze a project for enterprise readiness and distribute findings into sdlc. Use when assessing production hardening gaps.
---

# SDLC Enterprise Readiness Skill

Use this skill to analyze a project for enterprise readiness and distribute findings into sdlc structure.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Determine current and target stage (MVP/Production/Scale/Enterprise).
2. Load existing sdlc state.
3. Analyze three domains: Storage/Data, Operations, Security.
4. Route findings: add to existing milestone, update active tasks, or create new `ops-*` milestones.
5. For new milestones, write a plan and feed through sdlc-plan.
6. Report: stage assessment, distributed gaps, deferred items.
"#;

// ---------------------------------------------------------------------------
// sdlc-setup-quality-gates — Claude command
// ---------------------------------------------------------------------------

const SDLC_SETUP_QUALITY_GATES_COMMAND: &str = r#"---
description: Set up pre-commit hooks for this project — detect languages, install auto-fix and verification checks
argument-hint: <setup|update|check|fix>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-setup-quality-gates

Detect the project's languages and install pre-commit hooks with auto-fix and verification phases. Quality is enforced at commit time — the hook runs automatically on every `git commit`, ensuring no broken code ever reaches the repo.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Modes

| Mode | What it does |
|---|---|
| `setup` | Fresh install — detect languages, create `.git/hooks/pre-commit` |
| `update` | Read existing hook, identify gaps against the checklist, add missing checks |
| `check` | Audit existing hook — report what's configured, what's missing, what's slow |
| `fix` | Run all auto-fix tools on the entire codebase right now (not just staged files) |

---

## Tool Matrix

| Check | Go | TypeScript | Rust | Python |
|-------|-----|------------|------|--------|
| Format | gofmt | prettier | rustfmt | black/ruff |
| Imports | goimports | eslint-plugin-import | rustfmt | isort/ruff |
| Lint | golangci-lint | eslint | clippy | ruff |
| Types | compiler | tsc | compiler | mypy/pyright |
| Complexity | gocyclo | eslint complexity | clippy | radon/ruff |

## Threshold Defaults

| Metric | Default | Rationale |
|--------|---------|-----------|
| File length | 500 lines | Fits in head |
| Function length | 100 lines | Single responsibility |
| Cyclomatic complexity | 15-25 | Testable |
| Max pre-commit time | 10s | Won't get disabled |

---

## Steps

### 1. Resolve mode

Get mode from `$ARGUMENTS`. Default to `setup` if none provided.

### 2. Detect languages

```bash
ls go.mod Cargo.toml package.json pyproject.toml 2>/dev/null
```

Build a list of detected languages and their corresponding tools.

### 3. Check existing hooks

```bash
cat .git/hooks/pre-commit 2>/dev/null
```

### 4. Execute based on mode

#### setup (fresh install)

Create `.git/hooks/pre-commit` with two-phase approach:

**Phase 1: Auto-fix** — run formatters on staged files, run linters with `--fix`, re-stage fixed files.

**Phase 2: Verify** — check formatting (should pass after phase 1), run linting (unfixable issues), type check, file length check, complexity check.

```bash
#!/bin/bash
set -e

# Get staged files by type
staged_by_ext() { git diff --cached --name-only --diff-filter=ACM | grep -E "$1" || true; }

STAGED_GO=$(staged_by_ext '\.go$')
STAGED_TS=$(staged_by_ext '\.(ts|tsx)$')
STAGED_RS=$(staged_by_ext '\.rs$')
STAGED_PY=$(staged_by_ext '\.py$')

# Phase 1: Auto-fix
[[ -n "$STAGED_GO" ]] && gofmt -w $STAGED_GO && git add $STAGED_GO
[[ -n "$STAGED_TS" ]] && npx prettier --write $STAGED_TS && npx eslint --fix $STAGED_TS 2>/dev/null; git add $STAGED_TS
[[ -n "$STAGED_RS" ]] && rustfmt $STAGED_RS && git add $STAGED_RS
[[ -n "$STAGED_PY" ]] && ruff format $STAGED_PY && ruff check --fix $STAGED_PY && git add $STAGED_PY

# Phase 2: Verify
[[ -n "$STAGED_GO" ]] && golangci-lint run ./...
[[ -n "$STAGED_TS" ]] && npx tsc --noEmit && npx eslint --max-warnings 0 $STAGED_TS
[[ -n "$STAGED_RS" ]] && cargo clippy -- -D warnings
[[ -n "$STAGED_PY" ]] && ruff check $STAGED_PY && mypy $STAGED_PY

# File length check
for f in $STAGED_GO $STAGED_TS $STAGED_RS $STAGED_PY; do
  [[ -f "$f" ]] && lines=$(wc -l < "$f") && [[ $lines -gt 500 ]] && echo "ERROR: $f ($lines > 500 lines)" && exit 1
done
```

Only include sections for detected languages. Make executable:
```bash
chmod +x .git/hooks/pre-commit
```

#### update (modify existing)

1. Read `.git/hooks/pre-commit`
2. Compare against the tool matrix for detected languages
3. Add missing checks (formatters, linters, type checks, length checks)
4. Preserve any custom project-specific checks already in the hook

#### check (audit)

1. Read existing hook
2. Report what's configured vs what's missing
3. Time the hook execution on a sample commit to check if it's under 10s

#### fix (run fixes now)

Run auto-fix tools on all files (not just staged):

```bash
# Go
[[ -f go.mod ]] && gofmt -w . && goimports -w .

# TypeScript
[[ -f package.json ]] && npx prettier --write . && npx eslint --fix .

# Rust
[[ -f Cargo.toml ]] && cargo fmt

# Python
[[ -f pyproject.toml ]] && ruff format . && ruff check --fix .
```

### 5. Test the hook

Stage a file and run the hook manually:
```bash
git stash
echo "// test" >> <some-file>
git add <some-file>
.git/hooks/pre-commit
git checkout -- <some-file>
git stash pop
```

### 6. Report

```
## Quality Gates: [mode]

**Languages:** [detected]
**Hook:** [created|updated|exists|missing]

### Checks Configured
| Check | Tool | Auto-fix | Phase |
|-------|------|----------|-------|
| Formatting | [tool] | YES | 1 |
| Linting | [tool] | PARTIAL | 1+2 |
| Types | [tool] | NO | 2 |
| File length | wc -l | NO | 2 |

### Missing (if any)
- [check]: [tool needed]
```

**Next:** `/sdlc-status`

---

## Rules

- KEEP hook under 10 seconds — if it's slow, it gets disabled
- CHECK staged files only (not whole repo) in pre-commit
- AUTO-FIX first, verify second
- RE-STAGE fixed files after auto-fix
- FAIL with context — show file:line and how to fix
- Only include checks for detected languages — don't install Go checks in a Rust project
"#;

// ---------------------------------------------------------------------------
// sdlc-setup-quality-gates — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_SETUP_QUALITY_GATES_PLAYBOOK: &str = r#"# sdlc-setup-quality-gates

Use this playbook to set up pre-commit hooks for a project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Modes

- `setup` — detect languages, create `.git/hooks/pre-commit`
- `update` — read existing hook, add missing checks
- `check` — audit existing hook, report gaps
- `fix` — run all auto-fix tools on the entire codebase

## Steps

1. Detect languages: check for `go.mod`, `Cargo.toml`, `package.json`, `pyproject.toml`.
2. Check existing hooks: `cat .git/hooks/pre-commit`.
3. For `setup`: create a two-phase pre-commit hook:
   - Phase 1: Auto-fix (formatters, `--fix` linters, re-stage)
   - Phase 2: Verify (lint, type check, file length ≤500 lines)
4. For `update`: read existing hook, compare against tool matrix, add missing checks.
5. For `check`: audit hook, report configured vs missing checks.
6. For `fix`: run auto-fix tools on all files (not staged-only).
7. Test: stage a file, run hook manually, verify it passes.

## Tool Matrix

| Check | Go | TypeScript | Rust | Python |
|-------|-----|------------|------|--------|
| Format | gofmt | prettier | rustfmt | black/ruff |
| Lint | golangci-lint | eslint | clippy | ruff |
| Types | compiler | tsc | compiler | mypy |

## Key Rules

- Keep hook under 10 seconds
- Check staged files only in pre-commit
- Auto-fix first, verify second
- Only include checks for detected languages
"#;

// ---------------------------------------------------------------------------
// sdlc-setup-quality-gates — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_SETUP_QUALITY_GATES_SKILL: &str = r#"---
name: sdlc-setup-quality-gates
description: Set up pre-commit hooks and quality gates for a project. Use when configuring automated quality enforcement.
---

# SDLC Setup Quality Gates Skill

Use this skill to set up pre-commit hooks and quality gates for a project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Detect project languages (Go, TypeScript, Rust, Python).
2. Create or update `.git/hooks/pre-commit` with two-phase approach: auto-fix then verify.
3. Wire quality gates into `.sdlc/config.yaml` for agent enforcement.
4. Test the hook with a sample staged file.
5. Report: checks configured, missing gaps, sdlc config status.

## Key Rules

- Hook must run under 10 seconds (staged files only).
- Auto-fix phase runs formatters and `--fix` linters, then re-stages.
- Verify phase runs lint, type check, and file length check.
- Only include tools for detected languages.
"#;

// ---------------------------------------------------------------------------
// sdlc-specialize — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_SPECIALIZE_PLAYBOOK: &str = r#"# sdlc-specialize

Survey this project and generate a tailored AI team (agents + skills).

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Read project files: `VISION.md`, `AGENTS.md`, `CLAUDE.md`, root config files, key source dirs.
2. Run `sdlc state` to understand current SDLC phase and maturity.
3. Summarize: purpose, tech stack, domain areas, current state.
4. **Gate 1**: Present summary to user — wait for confirmation.
5. Design 3-5 specialist roles matching domain areas (persona name, role, domain ownership, model, skill name).
6. **Gate 2**: Present roster table — wait for user approval.
7. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each role.
8. Update `AGENTS.md` with a Team section listing all agents.
9. **Gate 3**: List all created files — confirm completion.

## Key Rules

- Always include user perspectives (not just engineers).
- Agents get concrete `This Codebase` sections with real file paths.
- Skills get 4-phase workflow (Understand, Design, Implement, Verify) + Done Gate.
- Wait for explicit user approval at each gate before proceeding.
"#;

// ---------------------------------------------------------------------------
// sdlc-specialize — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_SPECIALIZE_SKILL: &str = r#"---
name: sdlc-specialize
description: Survey this project and generate a tailored AI team (agents + skills). Use when setting up project-specific agent personas and domain skills.
---

# SDLC Specialize Skill

Use this skill to generate a project-specific AI team with agent personas and skills.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Survey: read project config, source dirs, VISION.md, AGENTS.md, `sdlc state`.
2. Summarize purpose, tech stack, domain areas, current state.
3. Gate 1: confirm understanding with user.
4. Design 3-5 specialist roles matching domain areas.
5. Gate 2: approve roster with user.
6. Generate `.claude/agents/<name>.md` and `.claude/skills/<role>/SKILL.md` for each.
7. Update AGENTS.md with Team section.
8. Gate 3: confirm all files created.

## Key Rules

- Agents have: frontmatter, background, Principles, This Codebase, ALWAYS/NEVER, When Stuck.
- Skills have: Quick Reference, Phase 1-4, Step Back, Done Gate.
- Always wait for user approval at each gate.
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook — Claude command
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_COMMAND: &str = r#"---
description: Create developer-scenario cookbook recipes that prove a milestone delivers real value — goals not features, promise not tickets
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-cookbook

Create developer-scenario cookbook recipes for a milestone. Cookbooks prove milestones deliver meaningful, usable capability — not just that features pass their tests. UAT asks "does the feature work?" while cookbooks ask "can a developer actually accomplish something?"

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Goals not features.** Recipes are named after what a developer accomplishes, not what system components are exercised. "Bootstrap a project with AI agents ready" not "test skeleton installer."
- **Promise validation.** The recipe set proves the milestone's stated promise. Read the vision, not the ticket backlog.
- **Replayable by strangers.** Every recipe runs from a clean state with exact commands. No assumed state.
- **Edit don't report.** This command produces recipe files, not a report that sits unread.

---

## Steps

### 1. Load the milestone

```bash
sdlc milestone info <slug> --json
```

Extract title, vision, features, acceptance_test. The vision is your north star — extract the promise in one sentence.

### 2. Load all features in the milestone

For each feature slug:
```bash
sdlc feature show <feature-slug>
```

Understand what was built — specs, designs, implementation status. Features are means; the promise is the end.

### 3. Identify developer personas

Identify 1-3 developer personas who would exercise this milestone:
- **Primary builder** — hands on keyboard daily, building with this tool
- **First-timer** — encountering this for the first time, following docs
- **Integration dev** — wiring this into an existing system or pipeline

Different personas reveal different recipes. A first-timer reveals setup friction. An integration dev reveals API assumptions.

### 4. Draft recipe titles

Generate 3-5 recipe candidates. Each title must be:
- A developer goal in plain language (action verb + object)
- Achievable using only what the milestone delivers
- Independently runnable from a clean state

**Reject** recipes that could be replaced by a unit test, have "verify" or "test" as the primary verb, require state from a previous recipe, or prove only that the system doesn't crash.

**Accept** recipes that would appear in a "getting started" guide, represent real workflows, and would make a skeptic say "okay, this actually works."

### 5. Write recipe files

Write each recipe to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`:

```markdown
# Recipe: [Developer Goal in Plain Language]

## Goal
One sentence: what a developer is trying to accomplish.

## What It Proves
Why this matters. Connect explicitly back to the milestone's promise.

## Personas
Which developer persona(s) this recipe serves.

## Prerequisites
What state the world needs to be in. Keep minimal. Create all fixtures inline.

## Steps
Exact commands a developer types, in order. Do not describe commands — write the commands.

## Expected
- Key output lines (exact text or pattern)
- Files that MUST exist after the recipe completes
- Files that MUST NOT exist

## Verdict Criteria
How to evaluate: PASS, PARTIAL (what worked/didn't), FAIL (what broke).
```

### 6. Write cookbook infrastructure

- Write `.cookbook/README.md` if missing (what it is, how to run, where results live, how to add recipes)
- Ensure `.cookbook/runs/` is in `.gitignore` (results are ephemeral, never committed)

### 7. Acid test

Before finishing, challenge the full recipe set:

1. **Goal check** — Is each recipe named after what a developer accomplishes, or what the system does?
2. **Promise check** — Does this recipe set prove the milestone's stated promise? Every part of the promise needs at least one recipe.
3. **Replayability check** — Can someone with a clean machine run every recipe without asking questions?
4. **Sufficiency check** — Would a skeptic, after running these, agree the milestone succeeded?

Remove or revise any recipe that fails. Three strong recipes beat five weak ones.

---

### 8. Next

**Next:** `/sdlc-cookbook-run <milestone-slug>`
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook-run — Claude command
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_RUN_COMMAND: &str = r#"---
description: Execute cookbook recipes for a milestone — run each scenario, record verdicts, create tasks for failures
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-cookbook-run

Execute cookbook recipes for a milestone and record the results. Be the developer. Run every step. Record honest verdicts. Failures become `[cookbook-gap]` tasks on the owning feature.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Ethos

- **Be the developer.** Run the exact commands in the recipe. Don't skip steps, don't improvise.
- **Never pause.** Execute all recipes in sequence. Stop only when all are done.
- **Always forward.** Failures create tasks. The state machine moves forward.
- **Honest verdicts.** A PARTIAL that documents what broke is worth more than a PASS that hides issues.

---

## Steps

### 1. Load recipes

```bash
ls .cookbook/recipes/<milestone-slug>/
```

Read all recipe files from `.cookbook/recipes/<milestone-slug>/`. If no recipes exist, stop and say: "No recipes found. Run `/sdlc-cookbook <milestone-slug>` first."

### 2. Load milestone context

```bash
sdlc milestone info <slug> --json
```

Map recipes to features — understand which feature each recipe exercises.

### 3. Create run directory

Create a timestamped run directory:
```
.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/
```

### 4. Execute each recipe

For each recipe file, in order:

1. **Read the recipe** — understand goal, prerequisites, steps, expected outcomes
2. **Run prerequisites** — execute setup commands, create fixtures
3. **Execute steps** — run each command exactly as written, capture output
4. **Evaluate expected** — check output against expected lines, verify files exist/don't exist
5. **Record verdict** — PASS, PARTIAL (what worked + what didn't), or FAIL (what broke)
6. **Write result file** — save as `<recipe-name>.result.md` in the run directory

### 5. Handle failures

On PARTIAL or FAIL:
```bash
sdlc task add <feature-slug> --title "[cookbook-gap] <recipe-name>: <failure summary>"
```

Create one task per failure, on the feature the recipe exercises.

### 6. Write summary

Write `summary.md` in the run directory:

```markdown
# Cookbook Run: <milestone-slug>

**Date:** <timestamp>
**Commit:** <git rev-parse HEAD>
**Environment:** <OS, arch>

## Results

| Recipe | Verdict | Notes |
|--------|---------|-------|
| recipe-001-... | PASS/PARTIAL/FAIL | ... |

## Overall: PASS / PARTIAL / FAIL

**What this confirms:**

**What is still open:**
```

### 7. Report

Print the summary with overall verdict, individual results, and tasks created for failures.

---

### 8. Next

| Outcome | Next |
|---|---|
| All PASS | `**Next:** /sdlc-milestone-verify <milestone-slug>` |
| Any FAIL/PARTIAL | `**Next:** /sdlc-run <failing-feature-slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_PLAYBOOK: &str = r#"# sdlc-cookbook

Create developer-scenario cookbook recipes that prove a milestone delivers real value.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load the milestone: `sdlc milestone info <slug> --json`. Extract the vision/promise.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 1-3 developer personas (primary builder, first-timer, integration dev).
4. Draft 3-5 recipe titles framed as developer goals (not feature names).
5. Write recipe files to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`.
   - Sections: Goal, What It Proves, Personas, Prerequisites, Steps, Expected, Verdict Criteria.
6. Write `.cookbook/README.md` if missing. Ensure `.cookbook/runs/` is in `.gitignore`.
7. Acid test: goal check, promise check, replayability check, sufficiency check.

## Key Rules

- Goals not features. Name recipes after what developers accomplish.
- Promise validation. Recipe set proves the milestone's stated promise.
- Replayable by strangers. Clean state, exact commands, inline fixtures.
- Three strong recipes beat five weak ones.
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook-run — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_RUN_PLAYBOOK: &str = r#"# sdlc-cookbook-run

Execute cookbook recipes for a milestone and record results.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load recipes from `.cookbook/recipes/<milestone-slug>/`.
2. Load milestone context: `sdlc milestone info <slug> --json`. Map recipes to features.
3. Create run dir: `.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/`.
4. For each recipe: run prerequisites, execute steps, evaluate expected, record verdict.
5. On PARTIAL/FAIL: `sdlc task add <feature-slug> --title "[cookbook-gap] <recipe>: <failure>"`.
6. Write `<recipe>.result.md` + `summary.md` in run directory.
7. Report overall verdict.

## Key Rules

- Be the developer. Run exact commands from the recipe.
- Honest verdicts. PARTIAL that documents what broke beats a PASS that hides issues.
- Failures create `[cookbook-gap]` tasks on the owning feature.
- All PASS → `/sdlc-milestone-verify <slug>`. Any FAIL → `/sdlc-run <failing-feature>`.
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_SKILL: &str = r#"---
name: sdlc-cookbook
description: Create developer-scenario cookbook recipes for a milestone. Use when proving a milestone delivers meaningful, usable capability — not just that features pass tests.
---

# SDLC Cookbook Skill

Use this skill to create developer-scenario cookbook recipes that prove a milestone delivers real value.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load milestone and extract the vision/promise: `sdlc milestone info <slug> --json`.
2. Load all features: `sdlc feature show <feature-slug>` for each.
3. Identify 1-3 developer personas (primary builder, first-timer, integration dev).
4. Draft 3-5 recipe titles as developer goals (not feature names).
5. Write recipes to `.cookbook/recipes/<milestone-slug>/recipe-NNN-<goal-slug>.md`.
6. Write `.cookbook/README.md` if missing. Add `.cookbook/runs/` to `.gitignore`.
7. Acid test: goal check, promise check, replayability check, sufficiency check.

## Key Rules

- Recipes named after developer goals, not system components.
- Recipe set proves the milestone's stated promise.
- Every recipe runnable from clean state with exact commands.
- Three strong recipes beat five weak ones.
"#;

// ---------------------------------------------------------------------------
// sdlc-cookbook-run — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_COOKBOOK_RUN_SKILL: &str = r#"---
name: sdlc-cookbook-run
description: Execute cookbook recipes for a milestone and record results. Use when validating that cookbook scenarios pass and creating tasks for failures.
---

# SDLC Cookbook Run Skill

Use this skill to execute cookbook recipes and record results for a milestone.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load recipes from `.cookbook/recipes/<milestone-slug>/`.
2. Load milestone context: `sdlc milestone info <slug> --json`. Map recipes to features.
3. Create run dir: `.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/`.
4. Execute each recipe: prerequisites, steps, evaluate expected, record verdict.
5. On failure: `sdlc task add <feature-slug> --title "[cookbook-gap] <recipe>: <failure>"`.
6. Write result files + summary in run directory.
7. Report overall verdict. All PASS → milestone-verify. Any FAIL → sdlc-run on failing feature.
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder — Claude command
// ---------------------------------------------------------------------------

const SDLC_PONDER_COMMAND: &str = r#"---
description: Open the ideation workspace — explore ideas with recruited thought partners, capture artifacts in the scrapbook, commit when ready. Embeds ideation, empathy, and recruitment protocols natively.
argument-hint: [slug or new idea description]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-ponder

Open the ponder workspace for creative exploration. This command sets the context —
from here, everything is conversation. You have access to all thinking tools. Artifacts
you produce land in the scrapbook and persist across sessions. Every session is logged
as a Markdown dialogue that accumulates over time.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Entering the workspace

### If $ARGUMENTS is a known slug

```bash
sdlc ponder show <slug>
sdlc ponder session list <slug>
```

Read the manifest, team, and all scrapbook artifacts. Load the team's agent definitions.
Read the session list — if sessions exist, read the most recent one to restore full context:

```bash
sdlc ponder session read <slug> <number>
```

Orient yourself from the **orientation strip** in the most recent session (or the manifest):

```
WHERE WE ARE   <current state of the thinking>
→ NEXT MOVE    <what the previous session said to do next>
COMMIT SIGNAL  <condition that unlocks commitment>
```

Summarize where the idea stands: what's been explored, who's on the team, open questions,
and what the orientation strip says to do next. Then dive in.

### If $ARGUMENTS looks like a new idea (not an existing slug)

1. Derive a slug from the idea text (lowercase, hyphens, max 40 chars).
2. Create the entry:
```bash
sdlc ponder create <slug> --title "<derived title>"
sdlc ponder capture <slug> --content "<verbatim user text>" --as brief.md
```
3. Read the brief. Identify domain signals. Recruit 2-3 initial thought partners —
   always include:
   - A domain expert (someone who's built something like this before)
   - An end-user advocate (who uses what this produces?)
   - A pragmatic skeptic (should this exist at all?)
4. Register them:
```bash
sdlc ponder team add <slug> --name "<name>" --role "<role>" \
  --context "<why this person>" --agent <agent-path>
```

### If no arguments

```bash
sdlc ponder list
```

Show all active ponder entries. Ask the user what they want to explore.

---

## During the session

You are a facilitator running a collaborative thinking session. The recruited team
members are your co-thinkers — channel their expertise and perspectives.

### What you do naturally

- **Interrogate the brief.** Push past the stated solution to find the real problem.
  "You said database — what problem does the database solve? Who reads these preferences?
  At what scale? What happens when cohort preferences conflict with individual ones?"
- **Channel thought partners.** Don't just think as yourself — voice the perspectives
  of recruited team members. "Kai would push back here — layered config inheritance is
  notoriously hard to debug. Have you thought about how a developer traces why a
  preference has a particular value?"
- **Suggest captures.** When a breakthrough happens — a reframing, a key decision, a
  constraint surfaced — offer to capture it: "That reframing is important. Should I
  capture it as problem.md in the scrapbook?"
- **Surface what's missing.** Track which dimensions of the idea have been explored.
  Problem framing? User perspectives? Technical landscape? Solution options? Decisions?
  Gently surface gaps: "We've been deep on the data model but haven't talked about who
  the users of this system actually are."

### Capturing artifacts

When something is worth persisting:

```bash
# Write inline content
sdlc ponder capture <slug> --content "<markdown content>" --as <filename>.md

# Or write to a temp file first for larger artifacts
# (write the file, then capture it)
sdlc ponder capture <slug> --file /tmp/exploration.md --as exploration.md
```

### Recruiting additional partners

If a new domain surfaces ("oh, this also needs a real-time sync layer"), recruit:

```bash
# Create the agent, then register them
sdlc ponder team add <slug> --name "<name>" --role "<role>" \
  --context "<context>" --agent .claude/agents/<name>.md
```

### Embedded capabilities

#### Ideation protocol

When exploring a problem:
1. **Understand** — capture the problem statement, your interpretation, scope, success criteria
2. **Gather context** — read relevant code, specs, adjacent systems
3. **Synthesize** — landscape, constraints, gaps, key files
4. **Consult thought partners** — channel each recruited expert's perspective
5. **Explore solutions** — 3-4 options including "do nothing", with trade-offs
6. **Step back** — assumption audit, fresh eyes test, skeptic's questions, reversal
7. **Think out loud** — share learnings, surprises, core tension, questions
8. **Collaborate** — listen, adjust, iterate with the user

#### Empathy protocol

When exploring user perspectives:
1. **Identify stakeholders** — direct users, indirect, blockers, forgotten
2. **Create perspective agents** — specific humans in specific situations
3. **Deep interview each** — context, needs, friction, delight, deal-breakers
4. **Synthesize** — alignments, conflicts, gaps, surprises
5. **Step back** — bias check, quiet voice, stress test, humility check
6. **Recommend** — evidence-based, tradeoffs acknowledged, unknowns flagged

Always include at least 3 perspectives. Always include a skeptic.

#### Recruitment protocol

When a domain signal emerges and you need a thought partner:
1. **Orient** — what expertise is needed and why
2. **Design the expert** — real name, career background at named companies, specific
   technical philosophy, strong opinions
3. **Create the agent** — write to `.claude/agents/<name>.md`
4. **Register** — `sdlc ponder team add <slug> --name ... --agent ...`

#### Feature shaping protocol

When an idea starts converging toward something buildable:
1. **Seed** — working name, one-liner, hypothesis, trigger
2. **User perspectives** — who uses this, who's affected, who's skeptical
3. **Expert consultation** — technical feasibility, architecture fit, constraints
4. **Shape** — core value prop, user stories, design decisions, trade-offs
5. **Define MVP** — minimum lovable, not minimum viable
6. **Step back** — do we need this? scope creep? quiet voices heard?

---

## Session Log Protocol

**Every session must be logged before ending.** The log is the persistent record
of the dialogue — it's how future sessions restore context without re-reading transcripts.

> ⚠️ **Sessions are not scrapbook artifacts — these are different things.**
>
> - ❌ Do NOT use the `Write` tool to create session files directly in the ponder directory
> - ❌ Do NOT use `sdlc ponder capture` to save sessions
> - ✓ ALWAYS use `sdlc ponder session log` — this is the only correct path
>
> Why it matters: `sdlc ponder session log` auto-numbers the file, places it in the
> correct `sessions/` subdirectory, increments the session counter in the manifest,
> and mirrors the orientation fields so future sessions and the web UI can read them.
> Skipping this command means the session is invisible — it becomes an artifact,
> not a session.

### Session file format

Session files are Markdown with a YAML frontmatter block. The frontmatter carries
metadata; the body is the free-form dialogue.

```markdown
---
session: <N>
timestamp: <ISO-8601 UTC>
orientation:
  current: "<one-liner: where the thinking is right now>"
  next: "<one-liner: concrete next action or focus>"
  commit: "<one-liner: condition that unlocks commitment>"
---

<session dialogue here — tool calls, partner voices, sketches, decisions, questions>
```

Inline markers to use consistently:
- `⚑  Decided:` — something resolved, with brief rationale
- `?  Open:` — unresolved question or tension still alive
- `Recruited: NAME · ROLE` — when a new partner joins mid-session
- `**NAME · ROLE**` — header for each partner voice block

### The only correct logging procedure

1. Write the session content to a temp file using the Write tool:
```bash
# Write tool → /tmp/ponder-session-<slug>.md
```
2. Register it:
```bash
sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>.md
```

The system auto-assigns the session number — do not try to number the file yourself.

---

## Ending the session

Before summarizing:

1. **Compose the session document.** Write a complete Markdown file to
   `/tmp/ponder-session-<slug>.md` using the Write tool. Include everything that
   happened — tool calls, partner voices, sketches, decisions (⚑), open questions (?),
   and recruitment events. Set the orientation fields to reflect where the thinking
   is right now, what should happen next, and what unlocks commitment.

2. **Log it:**
```bash
sdlc ponder session log <slug> --file /tmp/ponder-session-<slug>.md
```

3. **Summarize** what was explored, what was captured, and what remains unexplored.
   Include the orientation strip so the user sees it.

Always end with **Next:**

| State | Next |
|---|---|
| Early exploration, many gaps | `**Next:** /sdlc-ponder <slug>` (continue exploring) |
| Direction emerging, need depth | `**Next:** /sdlc-ponder <slug>` (continue with focus on <gap>) |
| Idea shaped, ready to commit | `**Next:** /sdlc-ponder-commit <slug>` |
| Idea explored and parked | `**Next:** /sdlc-ponder` (explore something else) |
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder-commit — Claude command
// ---------------------------------------------------------------------------

const SDLC_PONDER_COMMIT_COMMAND: &str = r#"---
description: Commit to a pondered idea — crystallize it into milestones and features via sdlc-plan
argument-hint: <ponder-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task
---

# sdlc-ponder-commit

Commit to a pondered idea. Reads the entire scrapbook, synthesizes with the recruited
thought partners using the feature-shaping protocol, and produces milestones, features,
and tasks that enter the state machine. The bridge from sketchbook to blueprint.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Prerequisites

A ponder entry should have enough substance to commit. Not a rigid checklist, but assess:

- Is the problem understood? (not just the solution)
- Have user perspectives been considered?
- Are the key technical decisions made?
- Is the scope defined?

If substance is thin, say so and suggest `/sdlc-ponder <slug>` to continue exploring.

---

## Steps

### 1. Load the scrapbook

```bash
sdlc ponder show <slug> --json
```

Read every artifact in the scrapbook. Read the team definitions. Build full context.

### 2. Load existing sdlc state

```bash
sdlc milestone list --json
sdlc feature list --json
```

Understand what already exists — avoid duplicating milestones or features.

### 3. Synthesize

With the full scrapbook and team context, determine the right structure:

**Small idea** (single capability, fits in one feature) →
- One feature, possibly added to an existing milestone
- Tasks decomposed from the exploration/decisions artifacts

**Medium idea** (multiple related capabilities) →
- One milestone with 2-5 features
- Vision synthesized from the problem framing and user perspectives

**Large idea** (significant initiative, multiple delivery phases) →
- Multiple milestones with clear ordering
- Each milestone has a user-observable goal

Present the proposed structure to the user.

### 4. Produce the plan

Write a structured plan to the scrapbook:

```bash
sdlc ponder capture <slug> --file /tmp/<slug>-plan.md --as plan.md
```

### 5. Distribute via sdlc-plan

Feed the plan into the state machine using the `/sdlc-plan` flow.

### 6. Update the ponder entry

```bash
sdlc ponder update <slug> --status committed
```

Record which milestones were created (update `committed_to` in manifest).

### 7. Report

Show what was created: milestones, features, tasks. Link back to the scrapbook.

---

### 8. Next

| Outcome | Next |
|---|---|
| Single feature created | `**Next:** /sdlc-run <feature-slug>` |
| Milestone created | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Multiple milestones | `**Next:** /sdlc-pressure-test <first-milestone-slug>` |
| Plan needs refinement | `**Next:** /sdlc-ponder <slug>` (back to exploring) |
"#;

// ---------------------------------------------------------------------------
// sdlc-recruit — Claude command
// ---------------------------------------------------------------------------

const SDLC_RECRUIT_COMMAND: &str = r#"---
description: Recruit an expert thought partner — creates an agent with real background, strong opinions, and domain expertise
argument-hint: <role description or domain context>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
---

# sdlc-recruit

Identify and recruit the ideal expert for a specific need. Produces a fully realized
agent definition — not a generic role, but a specific person with career history,
technical philosophy, and strong opinions.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Steps

### 1. Orient

Read the project to understand what expertise is needed:
- `VISION.md` or `docs/vision.md`
- `CLAUDE.md` or `AGENTS.md`
- Root config files for tech stack signals
- `sdlc state` for current project context

Parse $ARGUMENTS for the domain/role being recruited for.

### 2. Design the expert

Create a specific person, not a generic role:
- **Real name** (first-last, e.g., `kai-tanaka`)
- **Career background** — 3-4 sentences at named companies/projects with concrete
  technical contributions
- **Technical philosophy** — deeply held beliefs that create productive tension
- **Strong opinions** — specific to this domain, not generic best practices
- **Blind spots** — what this expert might miss (so other partners compensate)

### 3. Create the agent

Write to `.claude/agents/<name>.md`:

```markdown
---
name: <first-last>
description: Use when <specific triggers>. Examples — "<example 1>", "<example 2>".
model: opus
---

You are <Full Name>, <career background paragraph>.

## Your Principles
- **<Principle>.** <Why this matters>.
(3-5 principles)

## This Codebase
**<Area>:**
- `path/to/file` — relevance
(actual paths from the project)

## ALWAYS
- <concrete rule about this codebase>
(3-6 rules)

## NEVER
- <concrete anti-pattern for this domain>
(3-6 rules)

## When You're Stuck
1. **<Failure mode>:** <Specific approach with actual commands/paths>.
(2-4 entries)
```

### 4. Optionally register with a ponder entry

If recruiting for a ponder session:
```bash
sdlc ponder team add <ponder-slug> --name "<name>" --role "<role>" \
  --context "<why this person>" --agent .claude/agents/<name>.md
```

---

### 5. Next

| Context | Next |
|---|---|
| Within a ponder session | `**Next:** /sdlc-ponder <slug>` (continue with new partner) |
| For a pressure test | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Standalone | `**Next:** Use @<name> in conversation to invoke the agent` |
"#;

// ---------------------------------------------------------------------------
// sdlc-empathy — Claude command
// ---------------------------------------------------------------------------

const SDLC_EMPATHY_COMMAND: &str = r#"---
description: Interview user perspectives deeply — surface needs, friction, deal-breakers, and conflicts before making decisions
argument-hint: <feature, system, or decision to evaluate>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Task, AskUserQuestion
---

# sdlc-empathy

Run deep empathy interviews against a feature, system, or decision. Identifies specific
user personas, interviews each with probing questions, synthesizes findings, and surfaces
conflicts that reveal design tensions.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## Ethos

- **Users over builders.** What we want to build matters less than what users need.
- **Absence is information.** If we can't find a perspective, that's a gap to acknowledge.
- **Conflicts are gold.** Disagreement between personas reveals tensions to resolve.
- **Empathy requires effort.** Quick assumptions aren't empathy. Deep interviews are.

---

## Steps

### 1. Identify stakeholders

For the subject in question, identify 3-5 specific personas:
1. **Primary user** — hands on keyboard daily
2. **Indirect stakeholder** — affected downstream (ops, support, consumers)
3. **Adoption blocker** — skeptic or reluctant user
4. **Forgotten voice** — new user, edge case, accessibility need

Be specific: not "developer" but "developer debugging a production issue at 2am."

### 2. Find or create perspective agents

For each persona, check if an agent exists. If missing, recruit one using the
recruitment protocol — write a perspective agent to `.claude/agents/<persona>-perspective.md`.

**PAUSE if a critical perspective is missing.** Surface the gap to the user before
proceeding blind.

### 3. Deep interview each perspective

For each persona, ask across five dimensions:

**Context:** "Walk me through your typical day when you'd interact with this."
**Needs:** "What problem are you solving? What does success look like?"
**Friction:** "What would make you sigh? Give up? Try something else?"
**Delight:** "What would make you think 'they get it'?"
**Deal-breakers:** "What would make you refuse to use this? Actively complain?"

### 4. Synthesize

| Analysis | What to surface |
|---|---|
| Alignments | Needs shared across 3+ personas |
| Conflicts | Where personas disagree — these are the most valuable |
| Gaps | Needs we didn't anticipate |
| Overbuilding | Things we planned that no persona actually wants |

### 5. Step back

- **Bias check** — did we hear uncomfortable truths, or only what we wanted?
- **Quiet voice** — whose perspective was easiest to ignore?
- **Stress test** — what if each persona is right and we're wrong?
- **Humility** — what don't we know that we don't know?

### 6. Recommend

Evidence-based recommendations tied to specific interview findings.
Acknowledge tradeoffs — who loses and why.
Flag what still needs real user validation.

---

### 7. Capture (if in a ponder session)

```bash
sdlc ponder capture <slug> --file /tmp/perspectives.md --as perspectives.md
```

### 8. Next

| Context | Next |
|---|---|
| Within a ponder session | `**Next:** /sdlc-ponder <slug>` |
| Pre-pressure-test | `**Next:** /sdlc-pressure-test <milestone-slug>` |
| Standalone for a feature | `**Next:** /sdlc-run <feature-slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_PONDER_PLAYBOOK: &str = r#"# sdlc-ponder

Open the ponder workspace for creative exploration and ideation.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. If a slug is provided, load the entry:
   - `sdlc ponder show <slug>`
   - `sdlc ponder session list <slug>` — if sessions exist, read the most recent one:
     `sdlc ponder session read <slug> <N>`
   - Orient from the orientation strip (WHERE WE ARE / NEXT MOVE / COMMIT SIGNAL).
   - Summarize what's been explored, open questions, and what to do next.
2. If a new idea is provided, create the entry:
   - `sdlc ponder create <slug> --title "<title>"`
   - `sdlc ponder capture <slug> --content "<brief>" --as brief.md`
   - Recruit 2-3 thought partners: domain expert, end-user advocate, pragmatic skeptic.
3. If no arguments: `sdlc ponder list` and ask which to explore.
4. Facilitate: interrogate the brief, channel thought partners, suggest captures.
5. When artifacts are ready: `sdlc ponder capture <slug> --content "..." --as <name>.md`.
6. Before ending: write and log the session file:
   - Compose a Markdown session with YAML frontmatter (session, timestamp, orientation).
   - `sdlc ponder session log <slug> --file /tmp/session-<N>.md`
7. End with **Next:** — continue exploring, commit, or park.
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder-commit — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_PONDER_COMMIT_PLAYBOOK: &str = r#"# sdlc-ponder-commit

Crystallize a pondered idea into milestones and features.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Load scrapbook: `sdlc ponder show <slug> --json`. Read all artifacts.
2. Load existing state: `sdlc milestone list --json`, `sdlc feature list --json`.
3. Assess readiness: problem understood? users considered? scope defined?
4. Synthesize: small → feature, medium → milestone + features, large → multiple milestones.
5. Write plan: `sdlc ponder capture <slug> --file /tmp/plan.md --as plan.md`.
6. Feed into state machine via `/sdlc-plan`.
7. Update: `sdlc ponder update <slug> --status committed`.
8. Report what was created. **Next:** pressure-test or run.
"#;

// ---------------------------------------------------------------------------
// sdlc-recruit — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_RECRUIT_PLAYBOOK: &str = r#"# sdlc-recruit

Recruit an expert thought partner as a persistent agent.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Orient: read project context (CLAUDE.md, AGENTS.md, sdlc state).
2. Design the expert: real name, career background at named companies, strong opinions, blind spots.
3. Write agent to `.claude/agents/<name>.md` with principles, codebase context, ALWAYS/NEVER rules.
4. Optionally register with ponder entry:
   `sdlc ponder team add <slug> --name "<name>" --role "<role>" --context "<why>" --agent .claude/agents/<name>.md`
5. **Next:** use @<name> in conversation, or continue ponder session.
"#;

// ---------------------------------------------------------------------------
// sdlc-empathy — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_EMPATHY_PLAYBOOK: &str = r#"# sdlc-empathy

Run deep empathy interviews to surface user needs and conflicts.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Identify 3-5 specific personas: primary user, indirect stakeholder, adoption blocker, forgotten voice.
2. For each, create a perspective agent if missing.
3. Deep interview across: context, needs, friction, delight, deal-breakers.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Step back: bias check, quiet voice, stress test, humility.
6. Recommend with evidence. Acknowledge tradeoffs.
7. If in ponder session: `sdlc ponder capture <slug> --file /tmp/perspectives.md --as perspectives.md`.
8. **Next:** continue ponder, pressure-test, or run.
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_PONDER_SKILL: &str = r#"---
name: sdlc-ponder
description: Open the ideation workspace for creative exploration. Use when exploring, interrogating, or developing ideas before they become features.
---

# SDLC Ponder Skill

Use this skill to open a ponder workspace for exploring ideas.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. If slug given: `sdlc ponder show <slug>`. Then `sdlc ponder session list <slug>` —
   read the most recent session to restore context. Orient from the orientation strip.
2. If new idea: create entry, capture brief, recruit 2-3 thought partners.
3. If no args: `sdlc ponder list`. Ask which to explore.
4. Facilitate: interrogate, channel partners, capture artifacts.
5. Capture with `sdlc ponder capture <slug> --content "..." --as <name>.md`.
6. Before ending: compose session Markdown with YAML frontmatter (session, timestamp,
   orientation) and log it: `sdlc ponder session log <slug> --file /tmp/session.md`.
7. End with **Next:** — continue, commit, or park.
"#;

// ---------------------------------------------------------------------------
// sdlc-ponder-commit — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_PONDER_COMMIT_SKILL: &str = r#"---
name: sdlc-ponder-commit
description: Crystallize a pondered idea into milestones and features. Use when an idea is ready to enter the state machine.
---

# SDLC Ponder Commit Skill

Use this skill to commit a pondered idea into the state machine.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Load scrapbook: `sdlc ponder show <slug> --json`.
2. Load existing state: `sdlc milestone list --json`, `sdlc feature list --json`.
3. Assess readiness. If thin, suggest `/sdlc-ponder <slug>` instead.
4. Synthesize into milestones/features/tasks.
5. Write plan and feed via `/sdlc-plan`.
6. `sdlc ponder update <slug> --status committed`.
7. Report. **Next:** pressure-test or run.
"#;

// ---------------------------------------------------------------------------
// sdlc-recruit — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_RECRUIT_SKILL: &str = r#"---
name: sdlc-recruit
description: Recruit an expert thought partner as a persistent agent. Use when you need domain expertise, user perspectives, or productive skepticism.
---

# SDLC Recruit Skill

Use this skill to recruit an expert agent for the project.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Orient: read project context (CLAUDE.md, AGENTS.md, stack signals).
2. Design expert: real name, career at named companies, strong opinions, blind spots.
3. Write agent to `.claude/agents/<name>.md`.
4. Optionally register: `sdlc ponder team add <slug> --name ... --agent ...`.
5. **Next:** invoke with @<name> or continue session.
"#;

// ---------------------------------------------------------------------------
// sdlc-empathy — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_EMPATHY_SKILL: &str = r#"---
name: sdlc-empathy
description: Interview user perspectives deeply to surface needs, friction, and conflicts. Use before making design decisions or when pressure-testing scope.
---

# SDLC Empathy Skill

Use this skill to run deep empathy interviews.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Identify 3-5 specific personas for the subject.
2. Create perspective agents if missing.
3. Deep interview: context, needs, friction, delight, deal-breakers.
4. Synthesize: alignments, conflicts, gaps, overbuilding.
5. Step back: bias, quiet voice, stress test, humility.
6. Recommend with evidence. If in ponder, capture as perspectives.md.
"#;

// ---------------------------------------------------------------------------
// sdlc-prepare — Claude command
// ---------------------------------------------------------------------------

const SDLC_PREPARE_COMMAND: &str = r#"---
description: Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark ready to execute
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent
---

# sdlc-prepare

Pre-flight a milestone end-to-end: read the vision, audit every feature for alignment, fix structural gaps, write a wave plan, and mark the milestone prepared. This command makes real changes — it is not read-only.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Resolve the milestone slug

Use `$ARGUMENTS` as the slug. If none provided, run:
```bash
sdlc project prepare --json
```
and pick the active milestone from `milestone_slug` in the output.

### 2. Read the product vision

In order:
1. `docs/vision.md` — if it exists, read it
2. `CLAUDE.md` — read the `## Project` section
3. `README.md` — first two sections

Synthesize a one-paragraph vision statement to use as the alignment anchor.

### 3. Read milestone state

```bash
sdlc milestone info <slug> --json
```

Note: `vision`, `features` list, `prepared_at`.

### 4. Audit each feature for alignment

For each feature slug in the milestone:

```bash
sdlc feature show <slug> --json
```

Then read any existing artifacts:
- `.sdlc/features/<slug>/spec.md`
- `.sdlc/features/<slug>/design.md`
- `.sdlc/features/<slug>/tasks.md`

Check:
- Does the description exist and clearly connect to the milestone vision?
- Are tasks concrete and actionable (not vague placeholders)?
- Do dependency references point to real feature slugs?

### 5. Fix structural gaps

For each feature needing repair:

**Missing or weak description:**
```bash
sdlc feature update <slug> --description "<clear one-liner tied to the vision>"
```

**Broken dependency reference** (dep slug doesn't exist):
```bash
sdlc feature update <slug> --depends-on <correct-slug>
```

**Vague tasks** — rewrite with specific action verbs:
```bash
sdlc task update <slug> <task-id> --title "<specific action>"
```

**Features that don't belong** (contradict vision, wrong milestone) — archive them:
```bash
sdlc feature archive <slug>
```

### 6. Run prepare and build wave plan

```bash
sdlc project prepare --milestone <slug> --json
```

Parse the `waves` array. Write a wave plan file:

```bash
# Build wave_plan.yaml content from prepare output and write to the milestone dir
```

Wave plan format at `.sdlc/milestones/<slug>/wave_plan.yaml`:
```yaml
milestone: <slug>
waves:
  - number: 1
    label: Planning
    slugs: [feat-a, feat-b]
  - number: 2
    label: Implementation
    slugs: [feat-c]
```

Use wave labels from the prepare output if present; otherwise label Wave 1 `Planning`, Wave 2 `Implementation`, remaining waves `Wave N`.

### 7. Mark milestone prepared

```bash
sdlc milestone mark-prepared <slug>
```

### 8. Report

Print a summary:
1. **Vision** — the one-paragraph anchor used
2. **Fixes applied** — what was changed and why
3. **Wave Plan** — wave number, label, feature slugs, feature count
4. **Blocked features** — any features that couldn't be fixed (explain why)

### 9. Next

Always end with exactly one `**Next:**` line:

| Outcome | Next |
|---|---|
| Wave plan written, milestone prepared | `**Next:** /sdlc-run-wave <slug>` |
| Blockers remain after fixes | `**Next:** Resolve the blockers above, then re-run /sdlc-prepare <slug>` |
| All features already done (verifying) | `**Next:** /sdlc-milestone-uat <slug>` |
| Project idle (no active milestone) | `**Next:** /sdlc-ponder to start exploring ideas` |
"#;

// ---------------------------------------------------------------------------
// sdlc-prepare — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_PREPARE_PLAYBOOK: &str = r#"# sdlc-prepare

Pre-flight a milestone: align features with vision, fix gaps, write wave plan, mark prepared.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve slug from arguments; if missing, run `sdlc project prepare --json` and read `milestone_slug`.
2. Read product vision from `docs/vision.md`, `CLAUDE.md` §Project, and `README.md`.
3. Run `sdlc milestone info <slug> --json`. For each feature: `sdlc feature show <slug> --json` and read spec/design/tasks.md if present.
4. Fix structural gaps: missing descriptions (`sdlc feature update`), broken deps, vague tasks (`sdlc task update`), out-of-scope features (`sdlc feature archive`).
5. Run `sdlc project prepare --milestone <slug> --json`. Write `.sdlc/milestones/<slug>/wave_plan.yaml` from the `waves` array.
6. Run `sdlc milestone mark-prepared <slug>`.
7. Report: vision anchor, fixes applied, wave plan summary, any remaining blockers.
8. End: `**Next:** /sdlc-run-wave <slug>` (or `fix blockers then /sdlc-prepare` if blockers remain).
"#;

// ---------------------------------------------------------------------------
// sdlc-prepare — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_PREPARE_SKILL: &str = r#"---
name: sdlc-prepare
description: Pre-flight a milestone — align features with vision, fix gaps, write wave plan, mark prepared. Use before executing a milestone.
---

# SDLC Prepare Skill

Pre-flight a milestone end-to-end and mark it ready for parallel execution.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve slug; if missing run `sdlc project prepare --json` to find active milestone.
2. Read vision from `docs/vision.md`, `CLAUDE.md`, `README.md`.
3. Audit each feature: `sdlc feature show <slug> --json` + read artifacts. Fix descriptions, tasks, deps.
4. Run `sdlc project prepare --milestone <slug> --json`. Write `.sdlc/milestones/<slug>/wave_plan.yaml`.
5. Run `sdlc milestone mark-prepared <slug>`.
6. End: `**Next:** /sdlc-run-wave <slug>`.
"#;

// ---------------------------------------------------------------------------
// sdlc-run-wave — Claude command
// ---------------------------------------------------------------------------

const SDLC_RUN_WAVE_COMMAND: &str = r#"---
description: Execute Wave 1 features in parallel, then advance to the next wave
argument-hint: <milestone-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, Agent
---

# sdlc-run-wave

Execute the current wave of a milestone in parallel, then re-run prepare to advance to the next wave.

> **Before acting:** read `.sdlc/guidance.md` — especially §6 "Using sdlc". Never edit `.sdlc/` YAML directly. <!-- sdlc:guidance -->

## Steps

### 1. Resolve the milestone slug

Use `$ARGUMENTS` as the slug. If none provided, run:
```bash
sdlc project prepare --json
```
and read `milestone_slug` from the output.

### 2. Check for wave plan

Read `.sdlc/milestones/<slug>/wave_plan.yaml`. If missing, stop and tell the user:

> Wave plan not found. Run `/sdlc-prepare <slug>` first to generate it.

### 3. Get the authoritative current wave

Re-run prepare to get live state — this is always authoritative:
```bash
sdlc project prepare --milestone <slug> --json
```

Wave 1 of the prepare output is the current wave (features not yet done). The wave_plan.yaml is the structural record; prepare output is the live state.

### 4. Summarize the wave

Print:
- Wave number and label
- Feature count
- For each feature: slug, phase, next action
- Whether any features need worktrees (`needs_worktrees` flag)

### 5. Handle worktree features

If any Wave 1 features have `needs_worktrees: true`, print a notice for each:

> **Manual step required:** Feature `<slug>` needs a dedicated worktree.
> Run in a separate terminal: `/sdlc-run <slug>`

Skip these features from the parallel batch.

### 6. Execute remaining Wave 1 features in parallel

For each remaining feature in Wave 1, spawn a parallel Agent call running `/sdlc-run <feature-slug>`.

Use the Agent tool with multiple concurrent calls — one per feature. Do not run them sequentially.

Wait for all agents to complete.

### 7. Advance to next wave

After all Wave 1 agents complete, re-run:
```bash
sdlc project prepare --milestone <slug> --json
```

Check the result:
- **Waves remain** — loop back to step 3 and execute the next wave.
- **No waves remain, milestone is `Verifying`** — proceed to step 8.
- **Blockers surfaced** — stop and report them.

### 8. Run UAT automatically when all waves are done

When prepare returns no remaining waves and the milestone is `Verifying`, invoke the acceptance test immediately — do not stop and print a Next suggestion:

```
/sdlc-milestone-uat <slug>
```

The UAT command will write results, call `sdlc milestone complete` on pass, and end with its own `**Next:**` line.

### 9. Next

Only print a `**Next:**` line if execution stopped before UAT:

| Outcome | Next |
|---|---|
| Blockers surfaced | `**Next:** Resolve blockers listed above, then /sdlc-run-wave <slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-run-wave — Playbook (Gemini / OpenCode)
// ---------------------------------------------------------------------------

const SDLC_RUN_WAVE_PLAYBOOK: &str = r#"# sdlc-run-wave

Execute the current wave of a milestone in parallel, then advance to the next wave.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Steps

1. Resolve slug from arguments; if missing, run `sdlc project prepare --json` and read `milestone_slug`.
2. Read `.sdlc/milestones/<slug>/wave_plan.yaml` — if missing, tell user to run `/sdlc-prepare <slug>` first.
3. Run `sdlc project prepare --milestone <slug> --json` — Wave 1 of this output is the authoritative current wave.
4. For features with `needs_worktrees: true`: print manual step instructions; skip from parallel batch.
5. Execute remaining Wave 1 features in parallel (spawn concurrent `/sdlc-run <slug>` calls).
6. After all complete, re-run `sdlc project prepare --milestone <slug> --json`.
7. If waves remain, loop back to step 3. If no waves remain (milestone `Verifying`), invoke `/sdlc-milestone-uat <slug>` immediately — do not stop and print a Next suggestion. If blockers surfaced, end: `**Next:** Resolve blockers, then /sdlc-run-wave <slug>`.
"#;

// ---------------------------------------------------------------------------
// sdlc-run-wave — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_RUN_WAVE_SKILL: &str = r#"---
name: sdlc-run-wave
description: Execute Wave 1 features of a milestone in parallel, then advance to the next wave. Use after /sdlc-prepare.
---

# SDLC Run-Wave Skill

Execute the current milestone wave in parallel and advance.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Resolve slug; if missing run `sdlc project prepare --json` to find active milestone.
2. Check `.sdlc/milestones/<slug>/wave_plan.yaml` exists — if not, tell user to run `/sdlc-prepare <slug>`.
3. Run `sdlc project prepare --milestone <slug> --json`. Wave 1 is the current wave.
4. Skip features needing worktrees (print manual instructions). Execute the rest in parallel via `/sdlc-run <slug>`.
5. After all complete, re-run prepare. If waves remain, loop to step 3. If no waves (milestone `Verifying`), invoke `/sdlc-milestone-uat <slug>` directly. If blockers: `**Next:** Resolve blockers, then /sdlc-run-wave <slug>`.
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-run — Claude command
// ---------------------------------------------------------------------------

const SDLC_TOOL_RUN_COMMAND: &str = r#"---
description: Run an SDLC tool and act on the JSON result
argument-hint: <tool-name> [args]
allowed-tools: Bash, Read
---

# sdlc-tool-run

Run an installed SDLC tool and act on the JSON result.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

## Steps

### 1. Check available tools

```bash
cat .sdlc/tools/tools.md
```

### 2. Run the tool

Use `$ARGUMENTS` as `<tool-name>` (and any extra args):

```bash
# Simple question
sdlc tool run <name> --question "..."

# Scoped run
sdlc tool run <name> --scope "..."

# Complex JSON input
sdlc tool run <name> --json '{"key": "val"}'
```

### 3. Parse and act on the result

The tool outputs `{ ok, data, error?, duration_ms }`.

- If `ok: false` — explain the error and suggest a fix
- If `ok: true` — describe the findings and recommend next steps based on the data

**Next:** `/sdlc-tool-audit <name>` if the tool output reveals quality issues
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-build — Claude command
// ---------------------------------------------------------------------------

const SDLC_TOOL_BUILD_COMMAND: &str = r#"---
description: Scaffold and implement a new SDLC tool
argument-hint: <name> "<description>"
allowed-tools: Bash, Read, Write, Edit
---

# sdlc-tool-build

Scaffold and implement a new SDLC tool end-to-end.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

## Steps

### 1. Read an existing tool for reference

```bash
cat .sdlc/tools/ama/tool.ts
```

### 2. Scaffold the new tool

Use `$ARGUMENTS` as `<name> "<description>"`:

```bash
sdlc tool scaffold <name> "<description>"
```

This creates `.sdlc/tools/<name>/tool.ts`, `config.yaml`, and `README.md`.

### 3. Fill in the metadata

Open `tool.ts` and update the `meta` object:
- `name` — matches directory name exactly
- `display_name` — human-readable title
- `description` — one sentence, no trailing period
- `version` — semver (start at `"0.1.0"`)
- `input_schema` — JSON Schema for `--run` input
- `output_schema` — JSON Schema for `data` field in result

### 4. Implement `run()`

Implement the `run()` async function:
- Accept typed input matching `input_schema`
- Return `ToolResult<YourOutputType>`
- Return `{ ok: false, error: "..." }` on any error (never throw)
- Log progress to stderr via `makeLogger`

### 5. Add `--setup` mode (if needed)

Only add setup if the tool requires one-time initialization (e.g., building an index).
Skip this step for tools that are always ready.

### 6. Write README.md

Update `.sdlc/tools/<name>/README.md` with:
- One-sentence description
- Setup instructions (if applicable)
- Usage examples with exact commands
- How it works (1–3 sentences)

### 7. Test `--meta` mode

```bash
bun run .sdlc/tools/<name>/tool.ts --meta | jq .
```

Verify all fields are present and correct.

### 8. Test `--run` mode

```bash
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
echo '{"key": "val"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .
```

### 9. Test via CLI wrapper

```bash
sdlc tool run <name>
sdlc tool run <name> --json '{"key": "val"}'
```

### 10. Sync tools.md

```bash
sdlc tool sync
```

### 11. Commit

Stage and commit the new tool files.

**Next:** `/sdlc-tool-audit <name>`
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-audit — Claude command
// ---------------------------------------------------------------------------

const SDLC_TOOL_AUDIT_COMMAND: &str = r#"---
description: Audit an SDLC tool against the full quality contract
argument-hint: <tool-name>
allowed-tools: Bash, Read
---

# sdlc-tool-audit

Audit an SDLC tool against the full quality contract.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

Use `$ARGUMENTS` as `<tool-name>`.

## Checklist

Read `.sdlc/tools/<name>/tool.ts` and verify each item. Mark ✓ or ✗.

### Metadata (5 checks)

- [ ] `name` matches the directory name exactly (e.g. `"quality-check"` in `quality-check/`)
- [ ] `display_name` is human-readable and title-cased
- [ ] `description` is one sentence, present tense, no trailing period
- [ ] `version` is valid semver (e.g. `"0.1.0"`)
- [ ] `input_schema` and `output_schema` are defined

### Protocol (4 checks)

- [ ] `--meta` mode: exits 0 and outputs valid `ToolMeta` JSON
- [ ] `--run` mode: reads JSON from stdin before doing any work
- [ ] `--run` mode: exits 1 when `ok: false`
- [ ] `--setup` mode: handled gracefully (or explicitly absent with a comment)

### Error handling (4 checks)

- [ ] Errors return `{ ok: false, error: "..." }` — never throw unhandled exceptions
- [ ] All `catch` branches log the error and return an error result
- [ ] No bare `process.exit()` calls in library functions (only in the CLI entrypoint)
- [ ] All log output goes to stderr, not stdout

### Logging (2 checks)

- [ ] Uses `makeLogger` from `_shared/log.ts`
- [ ] No `console.log()` calls for logs (only `console.error()` via logger)

### Documentation (3 checks)

- [ ] `README.md` exists and has Usage section
- [ ] `README.md` has Setup section (or "Setup required: No" note)
- [ ] Instruction header in `tool.ts` has WHAT IT DOES, WHAT IT READS, WHAT IT WRITES, EXTENDING

## Commands

```bash
# Read the tool
cat .sdlc/tools/<name>/tool.ts

# Test --meta mode
bun run .sdlc/tools/<name>/tool.ts --meta | jq .

# Verify exit code
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run; echo "exit: $?"
```

**Next:** `/sdlc-tool-uat <name>` after all checks pass
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-uat — Claude command
// ---------------------------------------------------------------------------

const SDLC_TOOL_UAT_COMMAND: &str = r#"---
description: Run UAT scenarios for an SDLC tool
argument-hint: <tool-name>
allowed-tools: Bash
---

# sdlc-tool-uat

Run UAT scenarios for an SDLC tool.

> **Before acting:** read `.sdlc/guidance.md` — especially §7 "SDLC Tool Suite". <!-- sdlc:guidance -->

Use `$ARGUMENTS` as `<tool-name>`.

## Scenarios

Run each scenario and record the verdict (PASS / FAIL / SKIP).

### 1. Metadata

```bash
bun run .sdlc/tools/<name>/tool.ts --meta | jq .
```

Verify: `name`, `display_name`, `description`, `version`, `input_schema`, `output_schema` all present.

### 2. Happy path

```bash
echo '{"question":"test"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
```

Expected: `true`

### 3. Empty input (optional fields)

```bash
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok
```

Expected: `true` — tools must handle missing optional inputs gracefully.

### 4. CLI wrapper

```bash
sdlc tool run <name> --question "test"
```

Expected: JSON output on stdout, exit 0.

### 5. Error path

Supply invalid or missing required input and verify the tool returns an error result:

```bash
echo '{"invalid_key": true}' | bun run .sdlc/tools/<name>/tool.ts --run | jq '{ok, error}'
```

Expected: `{ "ok": false, "error": "..." }` (not a crash).

### 6. Logging format

```bash
echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run 2>&1 >/dev/null | head -5
```

Expected: lines match `[sdlc-tool:<name>] (INFO|WARN|ERROR|DEBUG):`.

### 7. Discovery

```bash
sdlc tool list
```

Expected: `<name>` appears in the output.

**Next:** `sdlc tool sync` to regenerate `tools.md`
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-run — Gemini playbook + Agents skill
// ---------------------------------------------------------------------------

const SDLC_TOOL_RUN_PLAYBOOK: &str = r#"# sdlc-tool-run

Run an installed SDLC tool and act on its JSON result.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Steps

1. Check available tools: `cat .sdlc/tools/tools.md`
2. Run the tool using `$ARGUMENTS` as `<tool-name>` (plus any extra args):
   - Simple question: `sdlc tool run <name> --question "..."`
   - Scoped run: `sdlc tool run <name> --scope "..."`
   - Complex input: `sdlc tool run <name> --input '{"key":"val"}'`
3. Parse the JSON result `{ ok, data, error?, duration_ms }`.
4. If `ok: false` — explain the error and suggest a fix.
   If `ok: true` — describe the findings and recommend next steps.

**Next:** `/sdlc-tool-audit <name>` if the tool output reveals quality issues
"#;

const SDLC_TOOL_RUN_SKILL: &str = r#"---
name: sdlc-tool-run
description: Run an installed SDLC tool and act on its JSON result. Use when an agent needs to invoke a tool and interpret the output.
---

# SDLC Tool-Run Skill

Run an SDLC tool and act on the result.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Check available tools: `cat .sdlc/tools/tools.md`
2. Run: `sdlc tool run <name> --question "..."` (or `--scope` / `--input` for complex input).
3. Parse `{ ok, data, error?, duration_ms }`.
4. `ok: false` → explain error. `ok: true` → act on findings.
5. End: `**Next:** /sdlc-tool-audit <name>` if issues found.
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-build — Gemini playbook + Agents skill
// ---------------------------------------------------------------------------

const SDLC_TOOL_BUILD_PLAYBOOK: &str = r#"# sdlc-tool-build

Scaffold and implement a new SDLC tool end-to-end.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Steps

1. Read `cat .sdlc/tools/ama/tool.ts` as the reference implementation.
2. Scaffold: `sdlc tool scaffold <name> "<description>"`
3. Open `.sdlc/tools/<name>/tool.ts` and fill the `--meta` mode (ToolMeta object).
4. Implement the `--run` mode: read JSON from stdin, do work, write `ToolResult` to stdout.
5. Handle `--setup` mode if the tool needs one-time setup; otherwise add a comment skipping it.
6. Write `README.md` with Usage and Setup sections.
7. Test `--meta`: `bun run .sdlc/tools/<name>/tool.ts --meta | jq .`
8. Test `--run` happy path: `echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok`
9. Run `/sdlc-tool-audit <name>` to check the full quality contract.
10. Run `/sdlc-tool-uat <name>` to verify all 7 scenarios.
11. Sync discovery: `sdlc tool sync`
12. Commit all changes.

**Next:** `/sdlc-tool-audit <name>`
"#;

const SDLC_TOOL_BUILD_SKILL: &str = r#"---
name: sdlc-tool-build
description: Scaffold and implement a new SDLC tool end-to-end. Use when building a new tool from scratch.
---

# SDLC Tool-Build Skill

Scaffold, implement, and ship a new SDLC tool.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Read `ama/tool.ts` as a reference implementation.
2. `sdlc tool scaffold <name> "<description>"` to create the skeleton.
3. Fill `--meta` mode (ToolMeta), `--run` mode (stdin → ToolResult), and `--setup` if needed.
4. Write `README.md` with Usage + Setup sections.
5. Test: `--meta` then `--run` happy path.
6. Audit: `/sdlc-tool-audit <name>` then UAT: `/sdlc-tool-uat <name>`.
7. `sdlc tool sync` and commit.
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-audit — Gemini playbook + Agents skill
// ---------------------------------------------------------------------------

const SDLC_TOOL_AUDIT_PLAYBOOK: &str = r#"# sdlc-tool-audit

Audit an SDLC tool against the full quality contract (18-item checklist).

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Checklist

Read `.sdlc/tools/<name>/tool.ts` and mark ✓ or ✗ for each item.

### Metadata (5)
- [ ] `name` matches the directory name exactly
- [ ] `display_name` is human-readable and title-cased
- [ ] `description` is one sentence, present tense, no trailing period
- [ ] `version` is valid semver (e.g. `"0.1.0"`)
- [ ] `input_schema` and `output_schema` are defined

### Protocol (4)
- [ ] `--meta` mode: exits 0 and outputs valid ToolMeta JSON
- [ ] `--run` mode: reads JSON from stdin before doing any work
- [ ] `--run` mode: exits 1 when `ok: false`
- [ ] `--setup` mode: handled gracefully (or explicitly absent with a comment)

### Error handling (4)
- [ ] Errors return `{ ok: false, error: "..." }` — never throw unhandled exceptions
- [ ] All `catch` branches log the error and return an error result
- [ ] No bare `process.exit()` calls in library functions (only in CLI entrypoint)
- [ ] All log output goes to stderr, not stdout

### Logging (2)
- [ ] Uses `makeLogger` from `_shared/log.ts`
- [ ] No `console.log()` calls for logs (only `console.error()` via logger)

### Documentation (3)
- [ ] `README.md` exists and has Usage section
- [ ] `README.md` has Setup section (or "Setup required: No" note)
- [ ] Instruction header in `tool.ts` has WHAT IT DOES, WHAT IT READS, WHAT IT WRITES, EXTENDING

**Next:** `/sdlc-tool-uat <name>` after all 18 checks pass
"#;

const SDLC_TOOL_AUDIT_SKILL: &str = r#"---
name: sdlc-tool-audit
description: Audit an SDLC tool against the full quality contract (18-item checklist). Use when verifying tool correctness before shipping.
---

# SDLC Tool-Audit Skill

Audit an SDLC tool against 18 quality checks in 5 categories.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

1. Read `.sdlc/tools/<name>/tool.ts`.
2. Check all 18 items: Metadata (5), Protocol (4), Error handling (4), Logging (2), Documentation (3).
3. Mark ✓/✗ for each. Report failing items with suggested fixes.
4. End: `**Next:** /sdlc-tool-uat <name>` when all pass.
"#;

// ---------------------------------------------------------------------------
// sdlc-tool-uat — Gemini playbook + Agents skill
// ---------------------------------------------------------------------------

const SDLC_TOOL_UAT_PLAYBOOK: &str = r#"# sdlc-tool-uat

Run UAT scenarios for an SDLC tool. Record PASS / FAIL / SKIP for each.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Scenarios

Use `$ARGUMENTS` as `<name>`.

### 1. Metadata
`bun run .sdlc/tools/<name>/tool.ts --meta | jq .`
Verify: `name`, `display_name`, `description`, `version`, `input_schema`, `output_schema` all present.

### 2. Happy path
`echo '{"question":"test"}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok`
Expected: `true`

### 3. Empty input (optional fields)
`echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run | jq .ok`
Expected: `true` — tools must handle missing optional inputs gracefully.

### 4. CLI wrapper
`sdlc tool run <name> --question "test"`
Expected: JSON output on stdout, exit 0.

### 5. Error path
`echo '{"invalid_key": true}' | bun run .sdlc/tools/<name>/tool.ts --run | jq '{ok, error}'`
Expected: `{ "ok": false, "error": "..." }` (not a crash).

### 6. Logging format
`echo '{}' | bun run .sdlc/tools/<name>/tool.ts --run 2>&1 >/dev/null | head -5`
Expected: lines match `[sdlc-tool:<name>] (INFO|WARN|ERROR|DEBUG):`.

### 7. Discovery
`sdlc tool list`
Expected: `<name>` appears in the output.

**Next:** `sdlc tool sync` to regenerate `tools.md`
"#;

const SDLC_TOOL_UAT_SKILL: &str = r#"---
name: sdlc-tool-uat
description: Run 7 UAT scenarios for an SDLC tool and record PASS/FAIL/SKIP. Use when validating a tool before shipping.
---

# SDLC Tool-UAT Skill

Run 7 UAT scenarios for an SDLC tool.

> Read `.sdlc/guidance.md` (§7 "SDLC Tool Suite"). <!-- sdlc:guidance -->

## Workflow

Record PASS / FAIL / SKIP for each scenario:
1. `--meta` — all required fields present
2. Happy path `--run` — `ok: true`
3. Empty input — `ok: true` (optional fields handled)
4. CLI wrapper `sdlc tool run` — JSON out, exit 0
5. Error path — `ok: false` with error message (no crash)
6. Logging format — lines match `[sdlc-tool:<name>] LEVEL:`
7. Discovery — `sdlc tool list` shows the tool

End: `**Next:** sdlc tool sync` if all pass.
"#;

// ---------------------------------------------------------------------------
// sdlc-quality-fix — Claude command
// ---------------------------------------------------------------------------

const SDLC_QUALITY_FIX_COMMAND: &str = r#"---
description: Fix failing quality-check results — reads /tmp/quality-check-result.json and applies the right fix strategy
argument-hint: [tool-name]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-quality-fix

Fix failing quality-check results. Reads `/tmp/quality-check-result.json` (written automatically by the quality-check tool when checks fail), selects the right fix strategy by failure count, and applies it.

> **Before acting:** read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Steps

### 1. Load failure data

```bash
cat /tmp/quality-check-result.json | jq '{ok, "failed": .data.failed, checks: [.data.checks[] | select(.status=="failed") | {name, output}]}'
```

If the file doesn't exist, run the quality-check tool first:
```bash
sdlc tool run quality-check
```

### 2. Triage by failure count

| Failures | Strategy | Rationale |
|----------|----------|-----------|
| 1 | fix-forward | Single targeted fix — patch, then confirm it's really fixed |
| 2–5 | fix-all | Multi-dimension review + fix across all seven code health axes |
| 6+ | remediate | Systemic problem — root-cause, enforce, document, verify |

### 3. Apply strategy

**1 failure → fix-forward:**
- Read the failing check name and its output from the JSON
- Diagnose: is this a one-line fix or a structural problem?
- If fixable: apply the minimal correct fix, re-run `sdlc tool run quality-check` to verify
- If structural: invoke `/fix-forward` with the check name as context

**2–5 failures → fix-all:**
- Extract all failing check names and their output
- Invoke `/fix-all` scoped to the files the failing checks touched
- Re-run `sdlc tool run quality-check` after fixes

**6+ failures → remediate:**
- The check suite is revealing a systemic issue
- Invoke `/remediate` with context: "quality-check found <N> failures: <check names>"
- The remediate skill will root-cause, fix, enforce, document, and verify

### 4. Verify

```bash
sdlc tool run quality-check
```

Expected: all previously failing checks now pass. If new failures appear, re-triage from Step 2.

**Next:** `/sdlc-setup-quality-gates update` if hook coverage is incomplete
"#;

const SDLC_QUALITY_FIX_PLAYBOOK: &str = r#"# sdlc-quality-fix

Fix failing quality-check results. Reads `/tmp/quality-check-result.json` (written by the quality-check tool when checks fail), selects the right fix strategy, and applies it.

> Read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Steps

### 1. Load failure data
`cat /tmp/quality-check-result.json | jq '{ok, "failed": .data.failed, checks: [.data.checks[] | select(.status=="failed") | {name, output}]}'`

If the file doesn't exist, run quality-check first:
`sdlc tool run quality-check`

### 2. Triage by failure count

| Failures | Strategy |
|----------|----------|
| 1 | Targeted patch — diagnose, fix, verify |
| 2–5 | Multi-dimension fix across all affected code |
| 6+ | Root-cause investigation, enforce, document |

### 3. Apply strategy

Extract failing check names and outputs. For each:
- Read the check output to understand the root cause
- Apply the minimal correct fix
- Avoid patching symptoms — fix the underlying issue

### 4. Verify
`sdlc tool run quality-check`
Expected: all previously failing checks now pass.

**Next:** `sdlc tool run quality-check` to verify
"#;

const SDLC_QUALITY_FIX_SKILL: &str = r#"---
name: sdlc-quality-fix
description: Fix failing quality-check results — load /tmp/quality-check-result.json, triage by failure count, apply the right fix strategy, and verify. Use when quality-check reports failures.
---

# SDLC Quality-Fix Skill

Fix failing quality-check results.

> Read `.sdlc/guidance.md`. <!-- sdlc:guidance -->

## Workflow

1. `cat /tmp/quality-check-result.json | jq '{ok, failed: .data.failed}'` — load failure data
2. Triage: 1 failure → targeted patch; 2–5 → multi-fix; 6+ → root-cause + remediate
3. Fix each failing check by reading its `output` field and applying the correct change
4. `sdlc tool run quality-check` — verify all checks now pass

End: `**Next:** sdlc tool run quality-check` to confirm clean.
"#;

// ---------------------------------------------------------------------------
// sdlc-vision-adjustment — Claude command
// ---------------------------------------------------------------------------

const SDLC_VISION_ADJUSTMENT_COMMAND: &str = r#"---
description: Systematically align all project docs, sdlc state, and code to a vision change — produces a graded drift report and change proposal, never applies changes without human approval
argument-hint: [describe the vision change]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-vision-adjustment

You are a technical program manager and architect who treats vision changes the way a surgeon treats incisions: methodical, complete, and with zero blind spots. When the vision shifts, you find every artifact that embeds the old direction — documentation, roadmap, code, guides, agent skills — and produce a complete drift report with specific proposed changes. You do not make changes during this skill. You map the gap, grade its severity, and present a change proposal for human approval before anything is touched.

## Principles

1. **Full Surface Area** — A vision change has consequences in places no one expects. Read everything: docs, sdlc milestones and features, code comments, guides, agent prompts, skills. Partial audits create false confidence.
2. **Drift Is Graded, Not Binary** — Not every inconsistency is equal. A locked architecture decision that contradicts the new direction is CRITICAL. A single sentence in a guide that uses old terminology is LOW. Grade each finding by its impact on implementation decisions.
3. **Propose, Don't Apply** — This skill produces a change proposal, not a change. The human approves the proposal before anything is touched. Unilateral application of vision changes is dangerous.
4. **Code Is Documentation** — Drift doesn't stop at markdown. Check: do any existing code structures, interfaces, constants, or data models embed the old direction?
5. **sdlc Is the Truth** — The milestone and feature list is the ground truth of what gets built. If the sdlc doesn't reflect the new vision, the team will build the wrong thing regardless of what the docs say.

---

## Phase 1: Capture the Vision Change

Before touching any file, document the change precisely.

Produce:

```markdown
## Vision Change Statement

**What changed:** [1-3 sentences. Specific, not vague.]

**What it replaces:** [What the old direction said. Quote the key phrase from vision.md if it exists.]

**Primary implication:** [The one thing that changes most as a result]

**Secondary implications:**
- [Implication 1]
- [Implication 2]
- [Implication 3]

**What does NOT change:** [Explicit non-changes. Prevents scope creep.]

**Success criteria for this adjustment:** [How will we know the adjustment is complete?]
```

**Gate 1a ✋** — Present the Vision Change Statement to the human. Ask:
- "Does this capture the change correctly?"
- "Are there implications I've missed?"
- "Are there things you explicitly want to NOT change?"

Do not proceed until the statement is approved. Everything downstream depends on getting this right.

---

## Phase 2: Document Audit

Read every markdown file in the project. Do not skim.

### 2a: Locate All Documents

```bash
find . -name "*.md" \
  -not -path "*/node_modules/*" \
  -not -path "*/.git/*" \
  -not -path "*/vendor/*" \
  | sort
```

Categorize by type:
- **Strategy docs** — vision.md, architecture.md, roadmap.md, CLAUDE.md
- **Agent configs** — .claude/agents/*.md, .claude/skills/**/SKILL.md
- **Guides** — .claude/guides/**/*.md, docs/**/*.md
- **Knowledge** — .ai/**, .blueprint/knowledge/**
- **Meta** — README.md, AGENTS.md

### 2b: Read and Tag Each Document

For each document, produce a finding entry (only for documents with drift):

```markdown
### `path/to/file.md`
**Type:** [strategy | agent | guide | knowledge | meta]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:**
- [Specific statement that contradicts the new vision]
**Proposed change:** [What needs to change — be specific]
```

### 2c: Strategy Docs First

Read strategy docs with extra care — they cascade into every downstream document that cites them. For `vision.md` and `architecture.md`, read every section, flag any claim that embeds the old direction, and flag any omission of a key aspect of the new direction.

---

## Phase 3: sdlc Audit

The roadmap is what gets built. Check every item.

```bash
sdlc milestone list
sdlc feature list
sdlc milestone info <slug>
```

For each milestone: Does the title still make sense? Are there features now wrong-headed or missing?

For each feature: Does it implement something that contradicts the new vision? Does it need scope changes?

Produce a roadmap drift table:

```markdown
## sdlc Drift

### Milestones
| Slug | Current Title | Status | Proposed Change |
|------|--------------|--------|-----------------|

### Features
| Slug | Current Title | Status | Proposed Change |
|------|--------------|--------|-----------------|

### Missing Features
| Proposed Slug | Title | Milestone | Reason Needed |
|--------------|-------|-----------|---------------|
```

---

## Phase 4: Code Audit

Check whether any existing code structures embed the old direction. Look for: type names, struct fields, constants, enums, interface names, and comments that reflect old concepts.

```bash
# Search for key terms from the old vision (replace with actual terms)
grep -rn "OLD_TERM" --include="*.rs" --include="*.ts" --include="*.tsx" . | grep -v "_test\."
```

Read the source files most likely to embed the old direction: domain types, interfaces, core business logic. For each file with potential drift:

```markdown
### `path/to/file.rs`
**Drift:** HIGH | MEDIUM | LOW
**What's wrong:** [Specific type/field/comment]
**Proposed change:** [Exact change needed]
```

---

## Phase 5: Drift Report and Change Proposal

Consolidate all findings into a single report:

```markdown
# Vision Adjustment Report

## Change Summary
[The Vision Change Statement from Phase 1]

---

## Drift Severity Overview

| Surface | CRITICAL | HIGH | MEDIUM | LOW |
|---------|----------|------|--------|-----|
| Strategy docs | N | N | N | N |
| Agent configs | N | N | N | N |
| Guides | N | N | N | N |
| sdlc roadmap | N | N | N | N |
| Code | N | N | N | N |
| **Total** | **N** | **N** | **N** | **N** |

---

## CRITICAL Findings
## HIGH Findings
## MEDIUM Findings
## LOW Findings

---

## Proposed sdlc Changes
### Milestones to Update
### Features to Update
### Features to Add
### Features to Remove or Cancel

---

## Proposed Code Changes

---

## Implementation Order

1. Update `vision.md` (source of truth)
2. Update `architecture.md` (cascades into agent skills and guides)
3. Update sdlc milestones and features
4. Update agent configs and skills
5. Update guides and knowledge docs
6. Apply code changes

---

## What Stays the Same
[Explicit list of things that do NOT change]
```

**Gate 5 ✋** — Present the complete drift report to the human. Ask:
- "Are there findings I missed?"
- "Do you agree with the severity ratings?"
- "Is the proposed implementation order right?"
- "Are there proposed changes you want to remove or modify?"

Wait for explicit approval before proceeding. After approval, apply changes in the sequence specified.

---

## Constraints

- NEVER modify any file during the audit phases — this skill ends at an approved proposal
- NEVER skip the code surface audit
- NEVER present a partial drift report — all surfaces before Gate 5
- ALWAYS get Vision Change Statement approval before Phase 2
- ALWAYS list "what stays the same" in the final report
- ALWAYS propose implementation order (vision.md → architecture → sdlc → agents → code)
- ALWAYS grade severity by implementation impact, not aesthetic distance

| Outcome | Next |
|---|---|
| Vision change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major direction change | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-vision-adjustment — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_VISION_ADJUSTMENT_PLAYBOOK: &str = r#"# sdlc-vision-adjustment

Align all project docs, sdlc state, and code to a vision change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture the Vision Change Statement — what changed, what it replaces, what does NOT change. **Gate 1a:** get human approval before reading any files.
2. Document audit — `find . -name "*.md" | sort`. Read every file. Tag findings: CRITICAL / HIGH / MEDIUM / LOW. Strategy docs first (they cascade).
3. sdlc audit — `sdlc milestone list`, `sdlc feature list`. For each: does it still make sense? Create a roadmap drift table.
4. Code audit — grep for old terms, read domain types and interfaces. Tag code drift findings.
5. Produce the Vision Adjustment Report: severity overview table, findings by severity, proposed sdlc changes (milestones/features to update/add/remove), proposed code changes, implementation order, what stays the same.
6. **Gate 5:** present the full report. Wait for human approval. Only then apply changes in order: vision.md → architecture → sdlc → agents → code.
"#;

// ---------------------------------------------------------------------------
// sdlc-vision-adjustment — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_VISION_ADJUSTMENT_SKILL: &str = r#"---
name: sdlc-vision-adjustment
description: Systematically align all project docs, sdlc state, and code to a vision change. Use when a strategic decision shifts the product direction and you need to find every place the old direction lives.
---

# SDLC Vision-Adjustment Skill

Audit and align the project to a vision change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture Vision Change Statement (what changed, what it replaces, what does NOT change). Gate 1a: get human approval before reading files.
2. Document audit — read every `.md` file, tag drift CRITICAL/HIGH/MEDIUM/LOW. Strategy docs first.
3. sdlc audit — `sdlc milestone list` + `sdlc feature list`. Produce roadmap drift table.
4. Code audit — grep for old terms, read domain types and interfaces.
5. Produce Vision Adjustment Report: severity overview, findings by severity, sdlc changes, code changes, implementation order, what stays the same.
6. Gate 5: get human approval. Then apply in order: vision.md → architecture → sdlc → agents → code.

NEVER modify any file before Gate 5 approval.
"#;

// ---------------------------------------------------------------------------
// sdlc-architecture-adjustment — Full Claude command
// ---------------------------------------------------------------------------

const SDLC_ARCHITECTURE_ADJUSTMENT_COMMAND: &str = r#"---
description: Systematically align all project docs, code, and sdlc state to an architecture change — produces a graded drift report and change proposal, never applies changes without human approval
argument-hint: [describe the architecture change]
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-architecture-adjustment

You are a systems architect and technical program manager who treats architecture changes the way a structural engineer treats load-bearing changes: measure twice, cut once, and document every consequence before touching anything. When the architecture shifts — components reorganized, interfaces redesigned, data flows rerouted, sequence diagrams invalidated — you find every artifact that embeds the old architecture and produce a complete drift report with specific proposed changes. You do not make changes during this skill. You map the gap, grade its severity, and present a change proposal for human approval before anything is touched.

## Principles

1. **Full Surface Area** — Architecture changes cascade everywhere. Read documentation, diagrams, code interfaces, data models, agent configs, sdlc features, and sequence flows. Partial audits create false confidence.
2. **Drift Is Graded, Not Binary** — A core interface contract that breaks the old component boundary is CRITICAL. A comment referencing an old component name is LOW. Grade each finding by its implementation impact.
3. **Propose, Don't Apply** — This skill produces a change proposal, not a change. Human approval required before anything is touched.
4. **Interfaces Are the Architecture** — The architecture lives in the interfaces, data models, and sequence flows — not just in documentation. Code that implements old component contracts is architectural drift.
5. **sdlc Is the Build Plan** — Features that assume old component boundaries or interfaces will build the wrong thing. If sdlc specs reference the old architecture, they must change before implementation begins.

---

## Phase 1: Capture the Architecture Change

Before touching any file, document the change precisely.

Produce:

```markdown
## Architecture Change Statement

**What changed:** [1-3 sentences. Specific: which component, boundary, interface, or flow changed.]

**What it replaces:** [What the old architecture said. Quote the key description if ARCHITECTURE.md exists.]

**Primary implication:** [The one thing that changes most as a result]

**Secondary implications:**
- [Implication 1 — component boundary change]
- [Implication 2 — interface contract change]
- [Implication 3 — data flow change]
- [Implication 4 — sequence diagram change]

**What does NOT change:** [Explicit non-changes. Prevents scope creep.]

**Success criteria for this adjustment:** [How will we know the adjustment is complete?]
```

**Gate 1a ✋** — Present the Architecture Change Statement to the human. Ask:
- "Does this capture the change correctly?"
- "Are there components or interfaces I've missed?"
- "Are there things you explicitly want to NOT change?"

Do not proceed until the statement is approved.

---

## Phase 2: Document Audit

Read every markdown file in the project. Do not skim.

### 2a: Locate All Documents

```bash
find . -name "*.md" \
  -not -path "*/node_modules/*" \
  -not -path "*/.git/*" \
  -not -path "*/vendor/*" \
  | sort
```

Categorize by type:
- **Architecture docs** — ARCHITECTURE.md, CLAUDE.md, any diagram files
- **Vision docs** — VISION.md, roadmap.md
- **Agent configs** — .claude/agents/*.md, .claude/skills/**/SKILL.md
- **Guides** — .claude/guides/**/*.md, docs/**/*.md
- **Knowledge** — .ai/**, .blueprint/knowledge/**
- **Meta** — README.md, AGENTS.md

### 2b: Read and Tag Each Document

For each document with drift, produce a finding entry:

```markdown
### `path/to/file.md`
**Type:** [architecture | vision | agent | guide | knowledge | meta]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:**
- [Specific statement that describes the old architecture]
**Proposed change:** [What needs to change — be specific]
```

### 2c: Architecture Docs First

Read ARCHITECTURE.md and CLAUDE.md with extra care — they cascade into every downstream document that cites them. For each, read every section, flag claims that embed the old component structure, and flag omissions of key aspects of the new architecture.

---

## Phase 3: Code Audit

Find code that implements old component boundaries, interface contracts, data models, or dependency patterns.

```bash
# Search for key terms from the old architecture (replace with actual terms)
grep -rn "OLD_COMPONENT" --include="*.rs" --include="*.ts" --include="*.tsx" . | grep -v "_test\."
```

Look specifically for:
- Type names, struct fields, enums, and constants that reflect old component names
- Interface definitions that embed old contracts
- Import paths that reference old module boundaries
- Comments that describe old data flows

For each file with potential drift:

```markdown
### `path/to/file.rs`
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What's wrong:** [Specific type/field/interface/comment]
**Proposed change:** [Exact change needed]
```

---

## Phase 4: Sequence / Flow Audit

Identify flows that are now incorrect under the new architecture.

For each major user-facing flow or agent workflow, trace the path:
- Which components are involved?
- Which interfaces are called?
- Which data models are passed?

Flag any flow where the old sequence is no longer valid:

```markdown
### Flow: [Name]
**Old sequence:** [brief description]
**New sequence:** [brief description under the new architecture]
**Drift:** CRITICAL | HIGH | MEDIUM | LOW
**What breaks:** [What will fail if not updated]
```

---

## Phase 5: sdlc Audit

Features that assume the old architecture in their spec, design, or tasks will build the wrong thing.

```bash
sdlc milestone list
sdlc feature list
```

For each feature in draft/specified/planned phases: Does its spec or design describe old component boundaries, old interfaces, or old data flows?

Produce an sdlc drift table:

```markdown
## sdlc Drift

### Features with Architectural Assumptions
| Slug | Current Phase | Artifact | What's Wrong | Proposed Change |
|------|--------------|----------|--------------|-----------------|

### Missing Features (new work created by the architecture change)
| Proposed Slug | Title | Milestone | Reason Needed |
|--------------|-------|-----------|---------------|
```

---

## Phase 6: Drift Report and Change Proposal

Consolidate all findings into a single report:

```markdown
# Architecture Adjustment Report

## Change Summary
[The Architecture Change Statement from Phase 1]

---

## Drift Severity Overview

| Surface | CRITICAL | HIGH | MEDIUM | LOW |
|---------|----------|------|--------|-----|
| Architecture docs | N | N | N | N |
| Code interfaces | N | N | N | N |
| Sequence flows | N | N | N | N |
| Agent configs | N | N | N | N |
| sdlc roadmap | N | N | N | N |
| **Total** | **N** | **N** | **N** | **N** |

---

## CRITICAL Findings
## HIGH Findings
## MEDIUM Findings
## LOW Findings

---

## Proposed sdlc Changes
### Features to Update
### Features to Add
### Features to Remove or Cancel

---

## Proposed Code Changes

---

## Implementation Order

1. Update `ARCHITECTURE.md` (source of truth for the system)
2. Update other docs (CLAUDE.md, guides, agent configs)
3. Update code (interfaces, types, module boundaries)
4. Update sdlc features (specs and designs that assume old architecture)

---

## What Stays the Same
[Explicit list of things that do NOT change]
```

**Gate 6 ✋** — Present the complete drift report to the human. Ask:
- "Are there findings I missed?"
- "Do you agree with the severity ratings?"
- "Is the proposed implementation order right?"
- "Are there proposed changes you want to remove or modify?"

Wait for explicit approval before proceeding. After approval, apply changes in the sequence specified.

---

## Constraints

- NEVER modify any file during the audit phases — this skill ends at an approved proposal
- NEVER skip the code surface audit
- NEVER skip the sequence/flow audit
- NEVER present a partial drift report — all surfaces before Gate 6
- ALWAYS get Architecture Change Statement approval before Phase 2
- ALWAYS list "what stays the same" in the final report
- ALWAYS propose implementation order (ARCHITECTURE.md → docs → code → sdlc)
- ALWAYS grade severity by implementation impact, not aesthetic distance

| Outcome | Next |
|---|---|
| Architecture change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major restructuring | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

// ---------------------------------------------------------------------------
// sdlc-architecture-adjustment — Playbook (Gemini/OpenCode)
// ---------------------------------------------------------------------------

const SDLC_ARCHITECTURE_ADJUSTMENT_PLAYBOOK: &str = r#"# sdlc-architecture-adjustment

Align all project docs, code, and sdlc state to an architecture change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture Architecture Change Statement — what component/boundary/interface/flow changed, what it replaces, what does NOT change. **Gate 1a:** get human approval before reading any files.
2. Document audit — `find . -name "*.md" | sort`. Read every file. Tag findings: CRITICAL / HIGH / MEDIUM / LOW. Architecture docs first (they cascade).
3. Code audit — grep for old component/interface terms, read domain types and interface definitions. Tag code drift findings.
4. Sequence/flow audit — trace major flows through the new architecture. Flag flows that are now incorrect.
5. sdlc audit — `sdlc feature list`. Read specs/designs for features in early phases. Find features that assume the old architecture.
6. Produce the Architecture Adjustment Report: severity overview table, findings by severity, proposed sdlc changes, proposed code changes, implementation order, what stays the same.
7. **Gate 6:** present the full report. Wait for human approval. Only then apply changes in order: ARCHITECTURE.md → docs → code → sdlc.
"#;

// ---------------------------------------------------------------------------
// sdlc-architecture-adjustment — Skill (Agents)
// ---------------------------------------------------------------------------

const SDLC_ARCHITECTURE_ADJUSTMENT_SKILL: &str = r#"---
name: sdlc-architecture-adjustment
description: Systematically align all project docs, code, and sdlc state to an architecture change. Use when a component boundary, interface contract, data flow, or system structure changes and you need to find every place the old architecture lives.
---

# SDLC Architecture-Adjustment Skill

Audit and align the project to an architecture change.

> Read `.sdlc/guidance.md` (§6: never edit YAML directly). <!-- sdlc:guidance -->

## Workflow

1. Capture Architecture Change Statement (what component/boundary/interface changed, what it replaces, what does NOT change). Gate 1a: get human approval before reading files.
2. Document audit — read every `.md` file, tag drift CRITICAL/HIGH/MEDIUM/LOW. Architecture docs first.
3. Code audit — grep for old component/interface terms, read domain types and interface definitions.
4. Sequence/flow audit — trace major flows through the new architecture. Flag invalidated flows.
5. sdlc audit — `sdlc feature list`. Read specs/designs that reference old architecture.
6. Produce Architecture Adjustment Report: severity overview, findings by severity, sdlc changes, code changes, implementation order, what stays the same.
7. Gate 6: get human approval. Then apply in order: ARCHITECTURE.md → docs → code → sdlc.

NEVER modify any file before Gate 6 approval.

| Outcome | Next |
|---|---|
| Architecture change aligned | `**Next:** /sdlc-run <feature-slug>` (if features were created) |
| Major restructuring | `**Next:** /sdlc-plan` with revised plan |
| Audit only, no changes needed | `**Next:** /sdlc-pressure-test <milestone-slug>` |
"#;

// ---------------------------------------------------------------------------
// Tool Suite TypeScript content
//
// These are the TypeScript files installed into `.sdlc/tools/` by `sdlc init`
// and `sdlc update`. Shared files (_shared/) are always overwritten (managed
// content). Per-tool config.yaml and README.md are written-if-missing
// (user-editable). Per-tool tool.ts is always overwritten (managed content).
// ---------------------------------------------------------------------------

/// Shared TypeScript contract — every tool imports from this file.
/// STDOUT is reserved for JSON; all logs go to STDERR.
const TOOL_SHARED_TYPES_TS: &str = r#"/**
 * SDLC Tool Shared Interface
 *
 * Every SDLC tool imports from this file. It defines the full type contract
 * that tools must satisfy. Do not change the shape of these types without
 * updating all core tools and regenerating tools.md.
 *
 * Tool protocol (stdin/stdout):
 * - --meta   No stdin. Writes ToolMeta JSON to stdout.
 * - --run    Reads JSON from stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 * - --setup  No stdin. Writes ToolResult JSON to stdout. Exit 0 ok, 1 error.
 *
 * All log output goes to STDERR. STDOUT is reserved for JSON only.
 */

/** Metadata describing a tool — returned by --meta mode. */
export interface ToolMeta {
  /** Matches the directory name exactly (e.g. "ama", "quality-check") */
  name: string
  /** Human-readable title shown in the tools list */
  display_name: string
  /** One sentence, present tense, no trailing period */
  description: string
  /** Semver, mirrors sdlc binary version at install time */
  version: string
  /** JSON Schema describing valid input for --run */
  input_schema: JsonSchema
  /** JSON Schema describing the data field in ToolResult */
  output_schema: JsonSchema
  /** True if --setup must run before first --run */
  requires_setup: boolean
  /** One sentence describing what setup does (required if requires_setup = true) */
  setup_description?: string
}

/** The result envelope returned by --run and --setup modes. */
export interface ToolResult<T = unknown> {
  ok: boolean
  data?: T
  /** Present only when ok = false */
  error?: string
  /** Wall-clock milliseconds for the operation */
  duration_ms?: number
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export type JsonSchema = Record<string, any>
"#;

/// Standard SDLC Tool Logger — writes structured lines to STDERR.
const TOOL_SHARED_LOG_TS: &str = r#"/**
 * Standard SDLC Tool Logger
 *
 * Writes structured log lines to STDERR (never stdout — stdout is reserved
 * for JSON output). Use this in every tool to produce consistent, parseable logs.
 *
 * Format: [sdlc-tool:<name>] LEVEL: message
 * Example: [sdlc-tool:ama] INFO:  Indexed 312 files in 842ms
 *
 * Set SDLC_TOOL_DEBUG=1 to enable debug-level output.
 */

export function makeLogger(toolName: string) {
  const prefix = `[sdlc-tool:${toolName}]`
  return {
    info:  (msg: string) => console.error(`${prefix} INFO:  ${msg}`),
    warn:  (msg: string) => console.error(`${prefix} WARN:  ${msg}`),
    error: (msg: string) => console.error(`${prefix} ERROR: ${msg}`),
    debug: (msg: string) => {
      if (process.env.SDLC_TOOL_DEBUG) console.error(`${prefix} DEBUG: ${msg}`)
    },
  }
}

export type Logger = ReturnType<typeof makeLogger>
"#;

/// Config loader — reads .sdlc/tools/<name>/config.yaml with defaults fallback.
const TOOL_SHARED_CONFIG_TS: &str = r#"/**
 * SDLC Tool Config Loader
 *
 * Reads .sdlc/tools/<name>/config.yaml. If the file is missing or unparseable,
 * returns the provided defaults — tools should never hard-fail on missing config.
 *
 * Supports flat key: value YAML only. Arrays and nested objects are intentionally
 * not supported — keep tool configs simple scalars.
 */
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

export function loadToolConfig<T extends Record<string, unknown>>(
  root: string,
  toolName: string,
  defaults: T,
): T {
  const configPath = join(root, '.sdlc', 'tools', toolName, 'config.yaml')
  try {
    const raw = readFileSync(configPath, 'utf8')
    const parsed = parseSimpleYaml(raw)
    return { ...defaults, ...parsed } as T
  } catch {
    return defaults
  }
}

/** Parse a flat key: value YAML file. Skips blank lines, comments, and array items. */
function parseSimpleYaml(content: string): Record<string, unknown> {
  const result: Record<string, unknown> = {}
  for (const line of content.split('\n')) {
    const trimmed = line.trim()
    if (!trimmed || trimmed.startsWith('#') || trimmed.startsWith('-')) continue
    const colonIdx = trimmed.indexOf(':')
    if (colonIdx === -1) continue
    const key = trimmed.slice(0, colonIdx).trim()
    const rawValue = trimmed.slice(colonIdx + 1).trim()
    if (!key || !rawValue) continue
    const value = rawValue.replace(/^["'](.*)["']$/, '$1')
    const num = Number(value)
    result[key] = Number.isNaN(num) ? value : num
  }
  return result
}
"#;

/// Cross-runtime helpers — normalizes argv, stdin, env, and exit for Bun/Deno/Node.
const TOOL_SHARED_RUNTIME_TS: &str = r#"/**
 * Cross-runtime helpers for Bun, Deno, and Node.
 *
 * Normalizes: argv access, stdin reading, env access, and process exit
 * across the three supported runtimes.
 *
 * Detection: checks for globalThis.Deno to identify Deno; falls back
 * to process (Node.js / Bun).
 */

/* eslint-disable @typescript-eslint/no-explicit-any */

/** Returns command-line arguments after the script name (process.argv[2+]). */
export function getArgs(): string[] {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return [...(globalThis as any).Deno.args]
  }
  return process.argv.slice(2)
}

/** Read all of stdin as a UTF-8 string. Returns empty string if stdin is a TTY or closed. */
export async function readStdin(): Promise<string> {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    const chunks: Uint8Array[] = []
    const reader = (globalThis as any).Deno.stdin.readable.getReader()
    try {
      while (true) {
        const { done, value } = await reader.read()
        if (done) break
        chunks.push(value)
      }
    } finally {
      reader.releaseLock()
    }
    const total = chunks.reduce((sum: number, c: Uint8Array) => sum + c.length, 0)
    const merged = new Uint8Array(total)
    let offset = 0
    for (const chunk of chunks) {
      merged.set(chunk, offset)
      offset += chunk.length
    }
    return new TextDecoder().decode(merged)
  }
  // Node.js / Bun
  if ((process.stdin as any).isTTY) return ''
  const chunks: Buffer[] = []
  for await (const chunk of process.stdin) {
    chunks.push(Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk))
  }
  return Buffer.concat(chunks).toString('utf8')
}

/** Get a process environment variable. Works across Bun, Deno, and Node. */
export function getEnv(key: string): string | undefined {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    return (globalThis as any).Deno.env.get(key)
  }
  return process.env[key]
}

/** Exit the process with the given code. */
export function exit(code: number): never {
  if (typeof (globalThis as any).Deno !== 'undefined') {
    ;(globalThis as any).Deno.exit(code)
  }
  process.exit(code)
  throw new Error('unreachable')
}
"#;

/// AMA tool implementation — keyword-indexed codebase search.
const TOOL_AMA_TS: &str = r#"/**
 * AMA — Ask Me Anything
 * =====================
 * Answers questions about the codebase by searching a pre-built keyword index.
 *
 * WHAT IT DOES
 * ------------
 * --setup:  Walks all source files matching configured extensions. On first run,
 *           indexes every file. On subsequent runs, skips unchanged files (mtime
 *           check), re-indexes changed/new files, and prunes deleted files.
 *           Writes chunks.json (TF-IDF index) and last_indexed.json (mtime map).
 *           Re-running --setup is always safe (incremental or full).
 *
 * --run:    Reads JSON from stdin: { "question": "string" }
 *           Loads the TF-IDF index, scores chunks by IDF-weighted keyword overlap,
 *           returns top results as source excerpts with relevance scores.
 *           Sources from files changed since last indexing are flagged stale.
 *
 * --meta:   Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/tools/ama/config.yaml                (extensions, chunk settings)
 * - .sdlc/tools/ama/index/chunks.json          (built by --setup)
 * - .sdlc/tools/ama/index/last_indexed.json    (mtime map; built by --setup)
 * - Source files matching config.extensions    (during --setup only)
 *
 * WHAT IT WRITES
 * --------------
 * - .sdlc/tools/ama/index/chunks.json          (during --setup; TF-IDF index)
 * - .sdlc/tools/ama/index/last_indexed.json    (during --setup; mtime map for incremental re-runs)
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 *
 * EXTENDING
 * ---------
 * Replace scoreChunks() with embedding-based cosine similarity to improve answer
 * quality. The rest of the pipeline (chunking, index format, protocol) stays the same.
 *
 * For LLM synthesis: call the Claude API in run() with the top excerpts as context.
 * Add "synthesis_model" to config.yaml to control which model is used.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { loadToolConfig } from '../_shared/config.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import {
  readdirSync, readFileSync, writeFileSync, mkdirSync, statSync, existsSync,
} from 'node:fs'
import { join, extname, relative } from 'node:path'

const log = makeLogger('ama')

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

interface AmaConfig {
  chunk_lines: number
  chunk_overlap: number
  max_results: number
  max_file_kb: number
  extensions: string
}

const DEFAULT_CONFIG: AmaConfig = {
  chunk_lines: 40,
  chunk_overlap: 5,
  max_results: 5,
  max_file_kb: 500,
  extensions: '.ts,.js,.tsx,.jsx,.rs,.go,.py,.rb,.java,.md,.txt,.yaml,.yml,.toml',
}

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'ama',
  display_name: 'AMA — Ask Me Anything',
  description: 'Answers questions about the codebase using a pre-built TF-IDF keyword index',
  version: '0.2.1',
  requires_setup: true,
  setup_description: 'Indexes source files for keyword search (first run is full index; subsequent runs are incremental)',
  input_schema: {
    type: 'object',
    required: ['question'],
    properties: {
      question: { type: 'string', description: 'The question to answer about the codebase' },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      sources: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            path: { type: 'string' },
            lines: { type: 'array', items: { type: 'number' }, minItems: 2, maxItems: 2 },
            excerpt: { type: 'string' },
            score: { type: 'number', description: 'TF-IDF relevance score (0.0–1.0)' },
            stale: { type: 'boolean', description: 'True if the source file changed since last index run' },
          },
        },
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Index types
// ---------------------------------------------------------------------------

interface Chunk {
  path: string
  start: number
  end: number
  tokens: string[]
}

interface Index {
  version: number
  generated: string
  chunks: Chunk[]
  idf: Record<string, number>
}

interface MtimeMap {
  version: number
  indexed_at: string
  files: Record<string, number>
}

interface AmaSource {
  path: string
  lines: [number, number]
  excerpt: string
  score: number
  stale?: boolean
}

interface AmaOutput {
  sources: AmaSource[]
}

// ---------------------------------------------------------------------------
// Setup — build the keyword index
// ---------------------------------------------------------------------------

export async function setup(root: string): Promise<ToolResult<{
  files_indexed: number
  files_skipped: number
  files_pruned: number
  chunks_written: number
  total_chunks: number
  duration_ms: number
  index_size_kb: number
}>> {
  const start = Date.now()
  const config = loadToolConfig(root, 'ama', DEFAULT_CONFIG)
  const extensions = new Set(
    String(config.extensions).split(',').map(e => e.trim()).filter(Boolean),
  )

  const indexDir = join(root, '.sdlc', 'tools', 'ama', 'index')
  mkdirSync(indexDir, { recursive: true })

  const chunksPath = join(indexDir, 'chunks.json')
  const mtimePath = join(indexDir, 'last_indexed.json')

  // Load previous index and mtime map for incremental re-indexing
  let prevChunks: Chunk[] = []
  let prevMtimes: Record<string, number> = {}
  const isIncremental = existsSync(chunksPath) && existsSync(mtimePath)
  if (isIncremental) {
    try {
      const prevIndex = JSON.parse(readFileSync(chunksPath, 'utf8')) as Index
      prevChunks = prevIndex.chunks ?? []
      const mtimeData = JSON.parse(readFileSync(mtimePath, 'utf8')) as MtimeMap
      prevMtimes = mtimeData.files ?? {}
      log.info(`incremental mode: ${prevChunks.length} existing chunks, ${Object.keys(prevMtimes).length} tracked files`)
    } catch {
      log.warn('could not load previous index — falling back to full re-index')
      prevChunks = []
      prevMtimes = {}
    }
  } else {
    log.info('full index mode (no previous index found)')
  }

  log.info(`indexing with extensions: ${[...extensions].join(', ')}`)

  const allFiles = walkFiles(root, extensions, Number(config.max_file_kb))
  log.info(`found ${allFiles.length} files to consider`)

  // Group previous chunks by file for efficient lookup
  const prevChunksByFile = new Map<string, Chunk[]>()
  for (const chunk of prevChunks) {
    const arr = prevChunksByFile.get(chunk.path) ?? []
    arr.push(chunk)
    prevChunksByFile.set(chunk.path, arr)
  }

  const newMtimes: Record<string, number> = {}
  const unchangedChunks: Chunk[] = []
  const freshChunks: Chunk[] = []
  let filesSkipped = 0
  let filesIndexed = 0

  for (const filePath of allFiles) {
    const relPath = relative(root, filePath)
    const mtime = statSync(filePath).mtimeMs
    if (isIncremental && prevMtimes[relPath] === mtime) {
      unchangedChunks.push(...(prevChunksByFile.get(relPath) ?? []))
      newMtimes[relPath] = mtime
      filesSkipped++
    } else {
      try {
        const content = readFileSync(filePath, 'utf8')
        const fileChunks = chunkFile(relPath, content, Number(config.chunk_lines), Number(config.chunk_overlap))
        freshChunks.push(...fileChunks)
        newMtimes[relPath] = mtime
        filesIndexed++
      } catch (e) {
        log.warn(`skipping ${relPath}: ${e}`)
      }
    }
  }

  // Count pruned files (tracked before but no longer on disk)
  const currentPaths = new Set(allFiles.map(f => relative(root, f)))
  const filesPruned = Object.keys(prevMtimes).filter(p => !currentPaths.has(p)).length
  if (filesPruned > 0) log.info(`pruned ${filesPruned} deleted/moved file(s) from index`)

  const allChunks = [...unchangedChunks, ...freshChunks]
  log.info(`${filesIndexed} indexed, ${filesSkipped} skipped, ${filesPruned} pruned — ${allChunks.length} total chunks`)

  // Compute smoothed IDF: log((N+1)/(df+1)) + 1 for each term
  const N = allChunks.length
  const df: Record<string, number> = {}
  for (const chunk of allChunks) {
    for (const token of chunk.tokens) {
      df[token] = (df[token] ?? 0) + 1
    }
  }
  const idf: Record<string, number> = {}
  for (const [term, freq] of Object.entries(df)) {
    idf[term] = Math.log((N + 1) / (freq + 1)) + 1
  }

  // Write index and mtime map
  const index: Index = { version: 2, generated: new Date().toISOString(), chunks: allChunks, idf }
  const indexJson = JSON.stringify(index)
  writeFileSync(chunksPath, indexJson)

  const mtimeMap: MtimeMap = { version: 1, indexed_at: new Date().toISOString(), files: newMtimes }
  writeFileSync(mtimePath, JSON.stringify(mtimeMap))

  const duration_ms = Date.now() - start
  const index_size_kb = Math.round(indexJson.length / 1024)
  log.info(`done in ${duration_ms}ms — index size: ${index_size_kb}KB`)

  return {
    ok: true,
    data: {
      files_indexed: filesIndexed,
      files_skipped: filesSkipped,
      files_pruned: filesPruned,
      chunks_written: freshChunks.length,
      total_chunks: allChunks.length,
      duration_ms,
      index_size_kb,
    },
    duration_ms,
  }
}

// ---------------------------------------------------------------------------
// Run — answer a question using the index
// ---------------------------------------------------------------------------

export async function run(
  input: { question?: string },
  root: string,
): Promise<ToolResult<AmaOutput>> {
  const start = Date.now()
  const config = loadToolConfig(root, 'ama', DEFAULT_CONFIG)

  const question = input.question?.trim()
  if (!question) {
    return { ok: false, error: 'input.question is required' }
  }

  const indexPath = join(root, '.sdlc', 'tools', 'ama', 'index', 'chunks.json')
  if (!existsSync(indexPath)) {
    return {
      ok: false,
      error: 'Index not built. Run setup first: sdlc tool run ama --setup',
    }
  }

  let index: Index
  try {
    index = JSON.parse(readFileSync(indexPath, 'utf8')) as Index
  } catch (e) {
    return { ok: false, error: `Failed to load index: ${e}. Re-run: sdlc tool run ama --setup` }
  }

  // Load mtime map for stale source detection (non-fatal if absent)
  let mtimes: Record<string, number> = {}
  try {
    const mtimePath = join(root, '.sdlc', 'tools', 'ama', 'index', 'last_indexed.json')
    if (existsSync(mtimePath)) {
      mtimes = (JSON.parse(readFileSync(mtimePath, 'utf8')) as MtimeMap).files ?? {}
    }
  } catch { /* stale detection skipped */ }

  log.info(`scoring ${index.chunks.length} chunks for: "${question}"`)

  // idf falls back gracefully to 1.0 weights for v1 indexes without IDF
  const idf = index.idf ?? {}
  const topChunks = scoreChunks(question, index.chunks, idf).slice(0, Number(config.max_results))

  const sources: AmaSource[] = []
  for (const { chunk, score } of topChunks) {
    const fullPath = join(root, chunk.path)
    try {
      const lines = readFileSync(fullPath, 'utf8').split('\n')
      const excerpt = lines.slice(chunk.start - 1, chunk.end).join('\n')

      // Stale detection: flag if file changed since last index run
      let stale = false
      try {
        if (mtimes[chunk.path] !== undefined && statSync(fullPath).mtimeMs !== mtimes[chunk.path]) {
          stale = true
          log.warn(`stale source: ${chunk.path} changed since last index run`)
        }
      } catch { /* file may not exist — handled above */ }

      const source: AmaSource = { path: chunk.path, lines: [chunk.start, chunk.end], excerpt, score }
      if (stale) source.stale = true
      sources.push(source)
    } catch {
      log.warn(`skipping deleted/moved file: ${chunk.path}`)
    }
  }

  const duration_ms = Date.now() - start
  log.info(`returned ${sources.length} sources in ${duration_ms}ms`)

  return { ok: true, data: { sources }, duration_ms }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

const SKIP_DIRS = new Set([
  'node_modules', '.git', 'target', 'dist', 'build', '.sdlc',
  '.next', '__pycache__', '.cache', 'coverage',
])

function walkFiles(root: string, extensions: Set<string>, maxFileKb: number): string[] {
  const results: string[] = []

  function walk(dir: string) {
    let entries: ReturnType<typeof readdirSync>
    try {
      entries = readdirSync(dir, { withFileTypes: true })
    } catch {
      return
    }
    for (const entry of entries) {
      if (entry.name.startsWith('.')) continue
      const full = join(dir, entry.name)
      if (entry.isDirectory()) {
        if (!SKIP_DIRS.has(entry.name)) walk(full)
      } else if (entry.isFile()) {
        if (!extensions.has(extname(entry.name))) continue
        try {
          if (statSync(full).size > maxFileKb * 1024) {
            log.warn(`skipping large file (${Math.round(statSync(full).size / 1024)}KB): ${relative(root, full)}`)
            continue
          }
        } catch {
          continue
        }
        results.push(full)
      }
    }
  }

  walk(root)
  return results
}

function chunkFile(
  relPath: string,
  content: string,
  chunkLines: number,
  overlap: number,
): Chunk[] {
  const lines = content.split('\n')
  const chunks: Chunk[] = []
  const step = Math.max(1, chunkLines - overlap)

  for (let i = 0; i < lines.length; i += step) {
    const start = i + 1 // 1-based line numbers
    const end = Math.min(i + chunkLines, lines.length)
    const tokens = extractTokens(lines.slice(i, end).join(' '))
    if (tokens.length > 0) {
      chunks.push({ path: relPath, start, end, tokens })
    }
    if (end >= lines.length) break
  }

  return chunks
}

/**
 * Extract lowercase tokens from text, splitting on camelCase and snake_case
 * boundaries to enable code-aware search. Words < 4 chars are omitted as noise.
 *
 * Examples:
 *   featureTransition → ['feature', 'transition']
 *   SdlcError         → ['sdlc', 'error']
 *   auth_token        → ['auth', 'token']
 *   authenticate      → ['authenticate']
 */
function extractTokens(text: string): string[] {
  // Split on camelCase and acronym boundaries before lowercasing
  const expanded = text
    .replace(/([a-z])([A-Z])/g, '$1 $2')        // camelCase → camel Case
    .replace(/([A-Z]+)([A-Z][a-z])/g, '$1 $2')  // XMLParser → XML Parser
  const seen = new Set<string>()
  const tokens: string[] = []
  for (const word of expanded.toLowerCase().split(/[^a-z0-9]+/)) {
    if (word.length >= 3 && !seen.has(word)) {
      seen.add(word)
      tokens.push(word)
    }
  }
  return tokens
}

/**
 * Score chunks using TF-IDF weighted overlap.
 * IDF is precomputed at index time (stored in chunks.json v2+).
 * Falls back to uniform weights (raw overlap) for v1 indexes without IDF.
 */
function scoreChunks(
  question: string,
  chunks: Chunk[],
  idf: Record<string, number>,
): { chunk: Chunk; score: number }[] {
  const queryTokens = extractTokens(question)
  if (queryTokens.length === 0) return []

  const hasIdf = Object.keys(idf).length > 0
  const results: { chunk: Chunk; score: number }[] = []

  for (const chunk of chunks) {
    const chunkSet = new Set(chunk.tokens)
    let score = 0
    let totalWeight = 0

    for (const token of queryTokens) {
      const weight = hasIdf ? (idf[token] ?? 1.0) : 1.0
      totalWeight += weight
      if (chunkSet.has(token)) score += weight
    }

    if (score > 0) {
      results.push({ chunk, score: totalWeight > 0 ? score / totalWeight : 0 })
    }
  }

  return results.sort((a, b) => b.score - a.score)
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'
const root = process.env.SDLC_ROOT ?? process.cwd()

if (mode === '--meta') {
  console.log(JSON.stringify(meta))
  exit(0)
} else if (mode === '--setup') {
  setup(root)
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as { question?: string }, root))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta, --setup, or --run.`)
  exit(1)
}
"#;

const TOOL_AMA_CONFIG_YAML: &str = r#"name: ama
version: 0.1.0
description: Answers questions about the codebase using a pre-built keyword index

# File extensions to include in the index (comma-separated)
extensions: .ts,.js,.tsx,.jsx,.rs,.go,.py,.rb,.java,.md,.txt,.yaml,.yml,.toml

# Number of lines per chunk
chunk_lines: 40

# Lines of overlap between consecutive chunks (reduces missed context at boundaries)
chunk_overlap: 5

# Maximum results to return per query
max_results: 5

# Skip files larger than this size (kilobytes)
max_file_kb: 500
"#;

const TOOL_AMA_README_MD: &str = r#"# AMA — Ask Me Anything

Answers questions about the codebase by searching a pre-built keyword index.

## Setup (run once)

```bash
sdlc tool run ama --setup
```

## Usage

```bash
sdlc tool run ama --question "where is JWT validation?"
sdlc tool run ama --question "how does feature transition work?"
```

## How it works

1. `--setup` walks source files, chunks them into 40-line windows, extracts keyword tokens,
   and writes `.sdlc/tools/ama/index/chunks.json`
2. `--run` scores chunks by keyword overlap with your question, returns top file excerpts
3. Your AI assistant reads the excerpts and synthesizes an answer

## Configuration

Edit `.sdlc/tools/ama/config.yaml` to change which file extensions are indexed
or to adjust chunk size, overlap, and result count.

## Index location

`.sdlc/tools/ama/index/chunks.json` — gitignored, regenerate with `--setup`

## Re-index when needed

Re-run `--setup` after significant file changes. It's fast and safe to run any time.
"#;

/// Quality-Check tool implementation — runs checks from .sdlc/tools/quality-check/config.yaml.
const TOOL_QUALITY_CHECK_TS: &str = r#"/**
 * Quality Check
 * =============
 * Runs checks defined in .sdlc/tools/quality-check/config.yaml and reports pass/fail.
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: { "scope"?: "string" }
 *          Loads checks from .sdlc/tools/quality-check/config.yaml.
 *          Runs each check's script as a shell command, records pass/fail + output.
 *          If scope is provided, only runs checks whose name matches the filter string.
 *          Returns ToolResult<{ passed, failed, checks[] }>.
 *
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 *
 * WHAT IT READS
 * -------------
 * - .sdlc/tools/quality-check/config.yaml
 *   → checks[]: { name, description, script }
 *
 * WHAT IT WRITES
 * --------------
 * - STDERR: structured log lines via _shared/log.ts
 * - STDOUT: JSON only (ToolResult shape from _shared/types.ts)
 *
 * EXTENDING
 * ---------
 * Add or edit checks in .sdlc/tools/quality-check/config.yaml:
 *   checks:
 *     - name: test
 *       description: Run unit tests
 *       script: cargo test --all
 * The quality-check tool picks them up automatically — no code changes needed.
 */

import type { ToolMeta, ToolResult } from '../_shared/types.ts'
import { makeLogger } from '../_shared/log.ts'
import { getArgs, readStdin, exit } from '../_shared/runtime.ts'
import { execSync } from 'node:child_process'
import { readFileSync } from 'node:fs'
import { join } from 'node:path'

const log = makeLogger('quality-check')

// ---------------------------------------------------------------------------
// Tool metadata
// ---------------------------------------------------------------------------

export const meta: ToolMeta = {
  name: 'quality-check',
  display_name: 'Quality Check',
  description: 'Runs checks from .sdlc/tools/quality-check/config.yaml and reports pass/fail',
  version: '0.3.0',
  requires_setup: false,
  input_schema: {
    type: 'object',
    properties: {
      scope: {
        type: 'string',
        description: 'Optional filter — only run checks whose name matches this string',
      },
    },
  },
  output_schema: {
    type: 'object',
    properties: {
      passed: { type: 'number' },
      failed: { type: 'number' },
      checks: {
        type: 'array',
        items: {
          type: 'object',
          properties: {
            name: { type: 'string' },
            description: { type: 'string' },
            command: { type: 'string' },
            status: { type: 'string', enum: ['passed', 'failed'] },
            output: { type: 'string' },
            duration_ms: { type: 'number' },
          },
        },
      },
    },
  },
}

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface PlatformCommand {
  name: string
  description?: string
  script: string
}

interface CheckResult {
  name: string
  description: string
  command: string
  status: 'passed' | 'failed'
  output: string
  duration_ms: number
}

interface QualityCheckOutput {
  passed: number
  failed: number
  checks: CheckResult[]
}

// ---------------------------------------------------------------------------
// Config YAML parser — reads checks[] from tool-local config.yaml
// ---------------------------------------------------------------------------

/**
 * Parse the `checks:` array from the tool's config.yaml.
 * Handles the specific YAML shape used by quality-check:
 *   checks:
 *     - name: <string>
 *       description: <string>
 *       script: <single-quoted or bare string>
 */
function parseChecksFromYaml(content: string): PlatformCommand[] {
  const checks: PlatformCommand[] = []
  const lines = content.split('\n')

  let inChecks = false
  let current: Partial<PlatformCommand> | null = null

  for (const line of lines) {
    // Top-level `checks:` section header
    if (/^checks:/.test(line)) {
      inChecks = true
      continue
    }
    // Any other top-level key ends the checks section
    if (/^\S/.test(line) && !/^checks:/.test(line)) {
      inChecks = false
    }

    if (!inChecks) continue

    // New item: `  - name: <value>`
    const itemMatch = line.match(/^\s{2}-\s+name:\s*(.*)$/)
    if (itemMatch) {
      if (current?.name && current?.script) {
        checks.push(current as PlatformCommand)
      }
      current = { name: unquoteYaml(itemMatch[1].trim()), description: '', script: '' }
      continue
    }

    if (!current) continue

    const descMatch = line.match(/^\s+description:\s*(.*)$/)
    if (descMatch) {
      current.description = unquoteYaml(descMatch[1].trim())
      continue
    }

    const scriptMatch = line.match(/^\s+script:\s*(.*)$/)
    if (scriptMatch) {
      current.script = unquoteYaml(scriptMatch[1].trim())
      continue
    }
  }

  if (current?.name && current?.script) {
    checks.push(current as PlatformCommand)
  }

  return checks
}

/** Strip surrounding single or double quotes from a YAML scalar value. */
function unquoteYaml(s: string): string {
  return s.replace(/^'([\s\S]*)'$/, '$1').replace(/^"([\s\S]*)"$/, '$1')
}

/** Load checks from the tool's own config.yaml. Returns [] on any error. */
function loadChecks(root: string): PlatformCommand[] {
  const configPath = join(root, '.sdlc', 'tools', 'quality-check', 'config.yaml')
  try {
    const raw = readFileSync(configPath, 'utf8')
    return parseChecksFromYaml(raw)
  } catch (e) {
    log.warn(`Could not read tool config at ${configPath}: ${e}`)
    return []
  }
}

// ---------------------------------------------------------------------------
// Run — execute platform checks
// ---------------------------------------------------------------------------

export async function run(
  input: { scope?: string },
  root: string,
): Promise<ToolResult<QualityCheckOutput>> {
  const start = Date.now()

  const commands = loadChecks(root)

  if (commands.length === 0) {
    log.warn('No checks configured in .sdlc/tools/quality-check/config.yaml — nothing to run')
    const duration_ms = Date.now() - start
    return {
      ok: true,
      data: { passed: 0, failed: 0, checks: [] },
      duration_ms,
    }
  }

  // Apply scope filter
  const scope = input.scope?.trim()
  const filtered = scope
    ? commands.filter(c => c.name.includes(scope))
    : commands

  log.info(`running ${filtered.length} check(s)${scope ? ` (scope: "${scope}")` : ''}`)

  const checks: CheckResult[] = []

  for (const cmd of filtered) {
    const checkStart = Date.now()
    log.info(`running check: ${cmd.name}`)

    let status: 'passed' | 'failed' = 'passed'
    let output = ''

    try {
      const result = execSync(cmd.script, {
        cwd: root,
        encoding: 'utf8',
        stdio: ['pipe', 'pipe', 'pipe'],
      })
      output = result.slice(-500) // last 500 chars
    } catch (e: unknown) {
      status = 'failed'
      if (e && typeof e === 'object' && 'stdout' in e && 'stderr' in e) {
        const err = e as { stdout?: string; stderr?: string }
        const combined = `${err.stdout ?? ''}${err.stderr ?? ''}`
        output = combined.slice(-500)
      } else {
        output = String(e).slice(-500)
      }
    }

    const duration_ms = Date.now() - checkStart
    log.info(`  ${cmd.name}: ${status} (${duration_ms}ms)`)

    checks.push({
      name: cmd.name,
      description: cmd.description ?? '',
      command: cmd.script,
      status,
      output,
      duration_ms,
    })
  }

  const passed = checks.filter(c => c.status === 'passed').length
  const failed = checks.filter(c => c.status === 'failed').length
  const duration_ms = Date.now() - start

  log.info(`done: ${passed} passed, ${failed} failed in ${duration_ms}ms`)

  return {
    ok: failed === 0,
    data: { passed, failed, checks },
    duration_ms,
  }
}

// ---------------------------------------------------------------------------
// CLI entrypoint
// ---------------------------------------------------------------------------

const mode = getArgs()[0] ?? '--run'
const root = process.env.SDLC_ROOT ?? process.cwd()

if (mode === '--meta') {
  console.log(JSON.stringify(meta))
  exit(0)
} else if (mode === '--run') {
  readStdin()
    .then(raw => run(JSON.parse(raw || '{}') as { scope?: string }, root))
    .then(result => { console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) })
    .catch(e => { console.log(JSON.stringify({ ok: false, error: String(e) })); exit(1) })
} else {
  console.error(`Unknown mode: ${mode}. Use --meta or --run.`)
  exit(1)
}
"#;

const TOOL_QUALITY_CHECK_CONFIG_YAML: &str = r#"# quality-check tool configuration
# Add your project's quality checks below.
# Each check runs its `script` as a shell command in the project root.
#
# Example:
#   checks:
#     - name: test
#       description: Run unit tests
#       script: cargo test --all
name: quality-check
version: "0.3.0"
checks:
"#;

const TOOL_QUALITY_CHECK_README_MD: &str = r#"# Quality Check

Runs checks defined in `.sdlc/tools/quality-check/config.yaml` and reports pass/fail.

## Usage

```bash
# Run all configured checks
sdlc tool run quality-check

# Filter to checks whose name matches a string
sdlc tool run quality-check --scope test
```

## How it works

Reads `checks` from `.sdlc/tools/quality-check/config.yaml`, runs each script as a shell
command in the project root, and reports pass/fail with the last 500 characters of output.

## Adding checks

Edit `.sdlc/tools/quality-check/config.yaml`:

```yaml
checks:
  - name: test
    description: Run unit tests
    script: cargo test --all
  - name: lint
    description: Run linter
    script: cargo clippy --all -- -D warnings
```

The quality-check tool picks them up automatically — no code changes needed.
"#;

/// Static tools.md manifest written at init time (before any TS runtime is available).
const TOOL_STATIC_TOOLS_MD: &str = r#"# SDLC Tools

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

## Adding a Custom Tool

Run `sdlc tool scaffold <name> "<description>"` to create a new tool skeleton.
Then implement the `run()` function in `.sdlc/tools/<name>/tool.ts` and run `sdlc tool sync`.
"#;

// ---------------------------------------------------------------------------
// write_core_tools — install TypeScript tool suite into .sdlc/tools/
// ---------------------------------------------------------------------------

/// Install (or refresh) the core SDLC tool suite into `.sdlc/tools/`.
///
/// - Creates `.sdlc/tools/` and `.sdlc/tools/_shared/` if missing.
/// - Shared files (`_shared/*.ts`) are always overwritten — managed content.
/// - `ama/tool.ts` is always overwritten — managed content.
/// - `ama/config.yaml` and `ama/README.md` are written-if-missing — user-editable.
/// - Writes a static `tools.md` manifest (overwritten; re-generated by `sdlc tool sync`).
/// - Appends `.sdlc/tools/*/index/` to `.gitignore` if not already present.
///
/// Called by both `sdlc init` and `sdlc update`.
pub fn write_core_tools(root: &Path) -> anyhow::Result<()> {
    let tools_dir = paths::tools_dir(root);
    let shared_dir = paths::tools_shared_dir(root);
    let ama_dir = paths::tool_dir(root, "ama");

    io::ensure_dir(&tools_dir)?;
    io::ensure_dir(&shared_dir)?;
    io::ensure_dir(&ama_dir)?;

    // Shared files — always overwrite (managed content agents must not edit)
    let shared_files: &[(&str, &str)] = &[
        ("types.ts", TOOL_SHARED_TYPES_TS),
        ("log.ts", TOOL_SHARED_LOG_TS),
        ("config.ts", TOOL_SHARED_CONFIG_TS),
        ("runtime.ts", TOOL_SHARED_RUNTIME_TS),
    ];
    for (filename, content) in shared_files {
        let path = shared_dir.join(filename);
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())
            .with_context(|| format!("failed to write _shared/{filename}"))?;
        if existed {
            println!("  updated: .sdlc/tools/_shared/{filename}");
        } else {
            println!("  created: .sdlc/tools/_shared/{filename}");
        }
    }

    // AMA tool.ts — always overwrite
    let ama_script = paths::tool_script(root, "ama");
    let existed = ama_script.exists();
    io::atomic_write(&ama_script, TOOL_AMA_TS.as_bytes()).context("failed to write ama/tool.ts")?;
    if existed {
        println!("  updated: .sdlc/tools/ama/tool.ts");
    } else {
        println!("  created: .sdlc/tools/ama/tool.ts");
    }

    // AMA config.yaml — write-if-missing (user may customize)
    let ama_config = paths::tool_config(root, "ama");
    let created = io::write_if_missing(&ama_config, TOOL_AMA_CONFIG_YAML.as_bytes())
        .context("failed to write ama/config.yaml")?;
    if created {
        println!("  created: .sdlc/tools/ama/config.yaml");
    } else {
        println!("  exists:  .sdlc/tools/ama/config.yaml");
    }

    // AMA README.md — write-if-missing
    let ama_readme = paths::tool_readme(root, "ama");
    let created = io::write_if_missing(&ama_readme, TOOL_AMA_README_MD.as_bytes())
        .context("failed to write ama/README.md")?;
    if created {
        println!("  created: .sdlc/tools/ama/README.md");
    } else {
        println!("  exists:  .sdlc/tools/ama/README.md");
    }

    // Quality-check tool
    let qc_dir = paths::tool_dir(root, "quality-check");
    io::ensure_dir(&qc_dir)?;

    // quality-check tool.ts — always overwrite (managed content)
    let qc_script = paths::tool_script(root, "quality-check");
    let existed = qc_script.exists();
    io::atomic_write(&qc_script, TOOL_QUALITY_CHECK_TS.as_bytes())
        .context("failed to write quality-check/tool.ts")?;
    println!(
        "  {}: .sdlc/tools/quality-check/tool.ts",
        if existed { "updated" } else { "created" }
    );

    // quality-check config.yaml — write-if-missing (user may customize)
    let qc_config = paths::tool_config(root, "quality-check");
    let created = io::write_if_missing(&qc_config, TOOL_QUALITY_CHECK_CONFIG_YAML.as_bytes())
        .context("failed to write quality-check/config.yaml")?;
    println!(
        "  {}: .sdlc/tools/quality-check/config.yaml",
        if created { "created" } else { "exists " }
    );

    // quality-check README.md — write-if-missing
    let qc_readme = paths::tool_readme(root, "quality-check");
    let created = io::write_if_missing(&qc_readme, TOOL_QUALITY_CHECK_README_MD.as_bytes())
        .context("failed to write quality-check/README.md")?;
    println!(
        "  {}: .sdlc/tools/quality-check/README.md",
        if created { "created" } else { "exists " }
    );

    // Static tools.md — overwrite (sdlc tool sync will regenerate from live metadata)
    let manifest_path = paths::tools_manifest_path(root);
    io::atomic_write(&manifest_path, TOOL_STATIC_TOOLS_MD.as_bytes())
        .context("failed to write tools/tools.md")?;

    // .gitignore — ensure index dirs are excluded
    append_gitignore_entry(root, ".sdlc/tools/*/index/")?;

    // .gitignore — ensure plain env files are never committed
    // (.sdlc/secrets/envs/*.age and *.meta.yaml are safe to commit)
    append_gitignore_entry(root, ".env")?;
    append_gitignore_entry(root, ".env.*")?;
    append_gitignore_entry(root, "!.env.example")?;

    Ok(())
}

/// Append `entry` to `.gitignore` if it is not already present.
/// Creates `.gitignore` if it does not exist. Idempotent.
fn append_gitignore_entry(root: &Path, entry: &str) -> anyhow::Result<()> {
    let gitignore = root.join(".gitignore");
    if gitignore.exists() {
        let current = std::fs::read_to_string(&gitignore).context("failed to read .gitignore")?;
        if current.lines().any(|l| l.trim() == entry) {
            return Ok(()); // already present
        }
        let suffix = if current.ends_with('\n') { "" } else { "\n" };
        let appended = format!("{current}{suffix}{entry}\n");
        io::atomic_write(&gitignore, appended.as_bytes()).context("failed to update .gitignore")?;
    } else {
        io::atomic_write(&gitignore, format!("{entry}\n").as_bytes())
            .context("failed to create .gitignore")?;
    }
    Ok(())
}
