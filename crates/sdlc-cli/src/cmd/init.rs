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
        ### Directive Interface\n\n\
        Use `sdlc next --for <slug> --json` to get the next directive. The JSON output tells the \
        consumer what to do next (action, message, output_path, is_heavy, gates).\n\n\
        ### Consumer Commands\n\n\
        - `/sdlc-next <slug>` — execute one step, then stop (human controls cadence)\n\
        - `/sdlc-run <slug>` — run autonomously until a HITL gate or completion\n\
        - `/sdlc-status [<slug>]` — show current state\n\
        - `/sdlc-plan` — distribute a plan into milestones, features, and tasks\n\
        - `/sdlc-milestone-uat <milestone-slug>` — run the acceptance test for a milestone\n\
        - `/sdlc-pressure-test <milestone-slug>` — pressure-test a milestone against user perspectives\n\
        - `/sdlc-enterprise-readiness [--stage <stage>]` — analyze production readiness\n\
        - `/sdlc-setup-quality-gates` — set up pre-commit hooks and quality gates\n\
        - `/sdlc-cookbook <milestone-slug>` — create developer-scenario cookbook recipes\n\
        - `/sdlc-cookbook-run <milestone-slug>` — execute cookbook recipes and record results\n\n\
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

fn remove_if_exists(dir: &Path, display_prefix: &str, filenames: &[&str]) -> anyhow::Result<()> {
    if !dir.exists() {
        return Ok(());
    }

    for filename in filenames {
        let path = dir.join(filename);
        if path.exists() {
            std::fs::remove_file(&path)?;
            println!("  removed: {display_prefix}/{filename} (deprecated)");
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
                "Autonomously drive a feature to completion or the next human gate",
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
                "Autonomously drive a feature to completion or the next human gate",
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
    ];

    remove_if_exists(
        &paths::claude_commands_dir(root),
        ".claude/commands",
        sdlc_files,
    )?;
    remove_if_exists(
        &paths::gemini_commands_dir(root),
        ".gemini/commands",
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
        ],
    )?;
    remove_if_exists(
        &paths::opencode_commands_dir(root),
        ".opencode/command",
        sdlc_files,
    )?;
    remove_if_exists(
        &root.join(".opencode/commands"),
        ".opencode/commands",
        sdlc_files,
    )?;
    remove_if_exists(
        &paths::codex_skills_dir(root),
        ".agents/skills",
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
        ],
    )?;
    remove_if_exists(&root.join(".codex/commands"), ".codex/commands", sdlc_files)?;

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

### 3. Handle human gates — STOP and surface to user

If `action` is any approval gate (`approve_spec`, `approve_design`, `approve_review`,
`approve_merge`, `wait_for_approval`, `unblock_dependency`):

Read the artifact at `output_path` and present it to the user. Say:
> "This phase requires your approval. Review the [type] above. Run `/sdlc-approve <slug> <type>` to approve."

**Do NOT call `sdlc artifact approve` without explicit user confirmation.**

### 4. Handle `done`

> "All SDLC phases complete for '[slug]'."

### 5. If `is_heavy` — confirm first

Ask the user to confirm before proceeding with long-running actions (implementation, QA).

### 6. Execute the directive

For **artifact creation** (`create_spec`, `create_design`, `create_tasks`, `create_qa_plan`, `create_review`, `create_audit`):
1. Run `sdlc feature show <slug> --json` for context
2. Read existing artifacts in `.sdlc/features/<slug>/`
3. Write a thorough Markdown artifact to `output_path`

For **implementation** (`implement_task`):
1. Run `sdlc task list <slug>` to find the next pending task
2. Read design and tasks artifacts for context
3. Implement the task, then run `sdlc task complete <slug> <task-id>`

### 7. Show updated state

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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
```

Show features grouped by: needs approval, blocked, in progress, ready.

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Steps

1. Resolve the slug.
   - If one is not provided, run `sdlc next` and pick a feature.
2. Run `sdlc next --for <slug> --json`.
3. Parse directive fields: `action`, `message`, `output_path`, `current_phase`, `is_heavy`, `gates`.
4. For human-gate actions (`approve_spec`, `approve_design`, `approve_review`, `approve_merge`, `wait_for_approval`, `unblock_dependency`):
   - Surface the artifact + message to the user.
   - Wait for explicit user approval before running `sdlc artifact approve`.
5. For creation actions:
   - Read feature context and existing artifacts.
   - Write the required artifact to `output_path`.
   - Mark it draft with `sdlc artifact draft <slug> <artifact_type>`.
6. For implementation:
   - Run `sdlc task list <slug>`.
   - Implement the next task and run `sdlc task complete <slug> <task_id>`.
7. Run `sdlc next --for <slug>` to show updated state.
"#;

const SDLC_STATUS_PLAYBOOK: &str = r#"# sdlc-status

Use this playbook to report SDLC state for the whole project or one feature.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Project view

Run:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`

## Feature view

Run:
- `sdlc feature show <slug>`
- `sdlc next --for <slug>`
- `sdlc task list <slug>`
- `sdlc comment list <slug>`
"#;

const SDLC_APPROVE_PLAYBOOK: &str = r#"# sdlc-approve

Use this playbook to review and approve an SDLC artifact.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Workflow

Project view:
- `sdlc state`
- `sdlc next`
- `sdlc query needs-approval`
- `sdlc query blocked`
- `sdlc query ready`

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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
description: Autonomously drive a feature to completion or the next human gate
argument-hint: <feature-slug>
allowed-tools: Bash, Read, Write, Edit, Glob, Grep
---

# sdlc-run

Drive a feature forward autonomously — executing every action in the state machine loop until a true human gate is reached or the feature is done.

Use `sdlc-next` when you want to execute one step at a time with human control between steps.
Use `sdlc-run` when you want the agent to drive as far as it can autonomously.

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

## True HITL gates (where sdlc-run stops)

| Gate | Why |
|---|---|
| `wait_for_approval` | Blocker/question comment exists — human resolves before proceeding |
| `unblock_dependency` | External blocker only a human can resolve |

Everything else — including `approve_merge` — runs autonomously.

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
> "This run includes heavy actions (implementation/QA). Proceeding autonomously — I'll stop at human gates."

### 3. Run the loop

Repeat until `done` or a HITL gate:

```
directive = sdlc next --for <slug> --json

if action == done        → report completion, exit
if action == HITL gate   → surface to user, exit
otherwise                → execute the action (see sdlc-next for action handlers)
                         → loop
```

Execute each action exactly as documented in `sdlc-next`. Do not skip steps or compress multiple actions into one pass — each action advances the state machine and must complete before the next is evaluated.

> **Never call `sdlc feature transition` directly.** Phases advance automatically when artifacts are approved. If a transition isn't happening, an artifact is missing a `draft` or `approve` call — re-check with `sdlc next --for <slug> --json`.

### 4. On HITL gate — surface clearly

When a human gate is reached, show:
1. What was accomplished in this run (phases traversed, artifacts written, tasks completed)
2. What the gate is and why it requires human action
3. Exactly what the human must do to unblock it

### 5. On unexpected failure

If an action fails in a way that cannot be recovered automatically, stop and report:
1. What action failed
2. What was attempted
3. What the human needs to resolve

Do not loop indefinitely on a failing action.

### 6. On completion

```bash
sdlc feature show <slug>
```

Report the final phase and a summary of everything accomplished.

---

### 7. Next

Always end with a single `**Next:**` line:

| Outcome | Next |
|---|---|
| Feature `done`, in a milestone | `**Next:** /sdlc-milestone-uat <milestone-slug>` |
| Feature `done`, no milestone | `**Next:** /sdlc-focus` |
| HITL gate reached | `**Next:** /sdlc-run <slug>` _(after resolving the gate)_ |
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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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
Real issue but doesn't block remaining steps. Create task with `sdlc task add`, record as failed, continue.

#### TASK + HALT
Failure makes remaining steps meaningless. Create task, record as failed, stop execution.

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

### 6. Final report

| Verdict | Next |
|---|---|
| PASS / PASS WITH TASKS | `**Next:** /sdlc-milestone-verify <slug>` |
| FAILED | `**Next:** /sdlc-run <blocking-feature-slug>` |
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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

Use this playbook to autonomously drive a feature to completion or the next human gate.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Steps

1. Resolve the feature slug. If not provided, run `sdlc next` and pick a feature.
2. Run `sdlc feature show <slug>` and `sdlc next --for <slug> --json` to assess scope.
3. Enter the loop:
   a. Run `sdlc next --for <slug> --json`.
   b. If `action == done` → report completion, exit.
   c. If `action` is a HITL gate (`wait_for_approval`, `unblock_dependency`) → surface to user, exit.
   d. Otherwise → execute the action per sdlc-next protocol, then loop.
4. For each action, execute exactly as documented — write artifacts, implement tasks, run approvals.
5. Never call `sdlc feature transition` directly — phases advance from artifact approvals.
6. On HITL gate, report what was accomplished and what the human must do to unblock.
7. On unexpected failure, stop and report what failed and what needs resolving.
"#;

const SDLC_PLAN_PLAYBOOK: &str = r#"# sdlc-plan

Use this playbook to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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
description: Autonomously drive a feature to completion or the next human gate. Use when a feature should run unattended through multiple phases.
---

# SDLC Run Skill

Use this skill to autonomously drive a feature through the sdlc state machine until a human gate or completion.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Workflow

1. Resolve the feature slug.
2. Run `sdlc next --for <slug> --json` to get the current directive.
3. Loop: execute action → re-read directive → repeat.
4. Stop at HITL gates (`wait_for_approval`, `unblock_dependency`) or `done`.
5. All other actions — including approvals and merge — execute autonomously.
6. Never call `sdlc feature transition` directly; phases advance from artifact approvals.
7. On gate or completion, report what was accomplished and what comes next.
"#;

const SDLC_PLAN_SKILL: &str = r#"---
name: sdlc-plan
description: Distribute a plan into sdlc milestones, features, and tasks. Use when decomposing a roadmap or plan into trackable work.
---

# SDLC Plan Skill

Use this skill to distribute a plan into sdlc milestones, features, and tasks.

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> **Before acting:** read `.sdlc/guidance.md` for engineering principles. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

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

> Read `.sdlc/guidance.md` before acting. <!-- sdlc:guidance -->

## Workflow

1. Load recipes from `.cookbook/recipes/<milestone-slug>/`.
2. Load milestone context: `sdlc milestone info <slug> --json`. Map recipes to features.
3. Create run dir: `.cookbook/runs/<milestone-slug>/<YYYY-MM-DDTHH-MM-SS>/`.
4. Execute each recipe: prerequisites, steps, evaluate expected, record verdict.
5. On failure: `sdlc task add <feature-slug> --title "[cookbook-gap] <recipe>: <failure>"`.
6. Write result files + summary in run directory.
7. Report overall verdict. All PASS → milestone-verify. Any FAIL → sdlc-run on failing feature.
"#;
