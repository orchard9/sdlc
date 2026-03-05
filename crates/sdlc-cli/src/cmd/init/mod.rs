use anyhow::Context;
use sdlc_core::{
    config::{Config, PlatformArg, PlatformCommand, PlatformConfig},
    io, paths,
    state::State,
};
use std::collections::HashMap;
use std::path::Path;

pub mod commands;
pub mod registry;
pub mod templates;

use templates::{AI_LOOKUP_INDEX_CONTENT, GUIDANCE_MD_CONTENT};
use templates::{
    MASQ_DEPLOY_SCRIPT, MASQ_DEV_MIGRATE_SCRIPT, MASQ_DEV_QUALITY_SCRIPT, MASQ_DEV_START_SCRIPT,
    MASQ_DEV_STOP_SCRIPT, MASQ_LOGS_SCRIPT,
};
use templates::{
    TOOL_AMA_CONFIG_YAML, TOOL_AMA_README_MD, TOOL_AMA_TS, TOOL_DEV_DRIVER_README_MD,
    TOOL_DEV_DRIVER_TS, TOOL_QUALITY_CHECK_CONFIG_YAML, TOOL_QUALITY_CHECK_README_MD,
    TOOL_QUALITY_CHECK_TS, TOOL_SHARED_AGENT_TS, TOOL_SHARED_CONFIG_TS, TOOL_SHARED_LOG_TS,
    TOOL_SHARED_RUNTIME_TS, TOOL_SHARED_TYPES_TS, TOOL_STATIC_TOOLS_MD,
    TOOL_TELEGRAM_RECAP_CONFIG_YAML, TOOL_TELEGRAM_RECAP_README_MD, TOOL_TELEGRAM_RECAP_TS,
};

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
        cfg.save(root)
            .with_context(|| format!("failed to write {}", config_path.display()))?;
        println!("  created: .sdlc/config.yaml");
    } else {
        println!("  exists:  .sdlc/config.yaml");
    }

    // 3. Write state.yaml if missing
    let state_path = paths::state_path(root);
    if !state_path.exists() {
        let state = State::new(&project_name);
        state
            .save(root)
            .with_context(|| format!("failed to write {}", state_path.display()))?;
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
        io::ensure_dir(&p).with_context(|| format!("failed to create {}", p.display()))?;
    }

    let index_path = root.join(paths::AI_LOOKUP_INDEX);
    io::write_if_missing(&index_path, AI_LOOKUP_INDEX_CONTENT.as_bytes())
        .with_context(|| format!("failed to write {}", index_path.display()))?;

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

    // 11. Bootstrap knowledge base (non-fatal — runs silently)
    if let Err(e) = sdlc_core::knowledge::librarian_init(root) {
        tracing::warn!(error = %e, "Knowledge librarian init failed (non-fatal)");
    }

    println!("\nSDLC initialized successfully.");
    println!("Next: sdlc ui    # then visit /setup to define Vision and Architecture");

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
    io::atomic_write(&path, GUIDANCE_MD_CONTENT.as_bytes())
        .with_context(|| format!("cannot write {}", path.display()))?;
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
        io::atomic_write(&agents_path, content.as_bytes()).with_context(|| {
            format!(
                "cannot create AGENTS.md at {} — check file permissions (try: chmod u+w AGENTS.md)",
                agents_path.display()
            )
        })?;
        println!("  created: AGENTS.md");
        return Ok(());
    }

    let existing = std::fs::read_to_string(&agents_path)
        .with_context(|| format!("cannot read AGENTS.md at {}", agents_path.display()))?;

    if existing.contains(SDLC_SECTION_START) {
        // Markers present — replace in-place
        if io::replace_between_markers(
            &agents_path,
            SDLC_SECTION_START,
            SDLC_SECTION_END,
            &marked_section,
        )
        .with_context(|| {
            format!(
                "cannot update AGENTS.md at {} — check file permissions (try: chmod u+w AGENTS.md)",
                agents_path.display()
            )
        })? {
            println!("  updated: AGENTS.md (SDLC section refreshed)");
        } else {
            println!("  warning: AGENTS.md has sdlc:start but no sdlc:end marker — skipped (add the closing <!-- sdlc:end --> marker to re-enable auto-updates)");
        }
    } else if existing.contains("## SDLC") {
        // Legacy format without markers — migrate
        let new_content = replace_legacy_sdlc_section(&existing, &marked_section);
        io::atomic_write(&agents_path, new_content.as_bytes()).with_context(|| {
            format!(
                "cannot update AGENTS.md at {} — check file permissions (try: chmod u+w AGENTS.md)",
                agents_path.display()
            )
        })?;
        println!("  updated: AGENTS.md (SDLC section converted to markers)");
    } else {
        // No SDLC section at all — append
        io::append_text(&agents_path, &format!("\n\n{marked_section}\n")).with_context(|| {
            format!(
                "cannot update AGENTS.md at {} — check file permissions (try: chmod u+w AGENTS.md)",
                agents_path.display()
            )
        })?;
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
        - `/sdlc-guideline <slug-or-problem>` — build an evidence-backed guideline through five research perspectives and TOC-first distillation\n\
        - `/sdlc-suggest` — analyze project state and suggest 3-5 ponder topics to explore next\n\
        - `/sdlc-beat [domain | feature:<slug> | --week]` — step back with a senior leadership lens; evaluate if we're building the right thing in the right direction; stores history in `.sdlc/beat.yaml`\n\
        - `/sdlc-recruit <role>` — recruit an expert thought partner as a persistent agent\n\
        - `/sdlc-empathy <subject>` — deep user perspective interviews before decisions\n\
        - `/sdlc-spike <slug> — <need>; [see <ref>]` — research, prototype, validate, and report; produces working prototype + findings in `.sdlc/spikes/<slug>/findings.md`\n\
        - `/sdlc-convo-mine [file or text]` — mine conversation dumps for signal; apply 5 perspective lenses, group themes, launch parallel ponder sessions per group\n\
        - `/sdlc-recap [slug]` — state-aware session recap with forward motion — synthesizes progress, classifies remaining work, and creates tasks or ponder entries so no session ends without a concrete next step\n\n\
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
    io::ensure_dir(commands_dir)
        .with_context(|| format!("cannot create directory {}", commands_dir.display()))?;

    for (filename, content) in commands {
        let path = commands_dir.join(filename);
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())
            .with_context(|| format!("cannot write {display_prefix}/{filename}"))?;
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
    io::ensure_dir(skills_dir)
        .with_context(|| format!("cannot create directory {}", skills_dir.display()))?;

    for (skill_name, content) in skills {
        let skill_dir = skills_dir.join(skill_name);
        io::ensure_dir(&skill_dir)
            .with_context(|| format!("cannot create directory {}", skill_dir.display()))?;
        let path = skill_dir.join("SKILL.md");
        let existed = path.exists();
        io::atomic_write(&path, content.as_bytes())
            .with_context(|| format!("cannot write {display_prefix}/{skill_name}/SKILL.md"))?;
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
            std::fs::remove_file(&path)
                .with_context(|| format!("cannot remove {}", path.display()))?;
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
    let pairs: Vec<(String, &str)> = commands::ALL_COMMANDS
        .iter()
        .map(|c| (c.claude_filename(), c.claude_content))
        .collect();
    let refs: Vec<(&str, &str)> = pairs.iter().map(|(f, c)| (f.as_str(), *c)).collect();
    write_user_command_scaffold(&commands_dir, "~/.claude/commands", &refs)
}

fn write_user_gemini_commands() -> anyhow::Result<()> {
    let commands: Vec<(String, String)> = commands::ALL_COMMANDS
        .iter()
        .map(|c| {
            (
                c.gemini_filename(),
                gemini_command_toml(c.gemini_description, c.playbook),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = commands
        .iter()
        .map(|(f, c)| (f.as_str(), c.as_str()))
        .collect();
    write_user_command_scaffold(
        &paths::user_gemini_commands_dir()?,
        "~/.gemini/commands",
        &refs,
    )
}

fn write_user_opencode_commands() -> anyhow::Result<()> {
    let commands: Vec<(String, String)> = commands::ALL_COMMANDS
        .iter()
        .map(|c| {
            (
                c.opencode_filename(),
                opencode_command_md(c.opencode_description, c.opencode_hint, c.playbook),
            )
        })
        .collect();
    let refs: Vec<(&str, &str)> = commands
        .iter()
        .map(|(f, c)| (f.as_str(), c.as_str()))
        .collect();
    write_user_command_scaffold(
        &paths::user_opencode_commands_dir()?,
        "~/.opencode/command",
        &refs,
    )
}

fn write_user_agents_skills() -> anyhow::Result<()> {
    let skills: Vec<(String, &str)> = commands::ALL_COMMANDS
        .iter()
        .map(|c| (c.skill_dirname(), c.skill))
        .collect();
    let refs: Vec<(&str, &str)> = skills.iter().map(|(d, s)| (d.as_str(), *s)).collect();
    write_user_skill_scaffold(&paths::user_agents_skills_dir()?, "~/.agents/skills", &refs)
}

/// Remove legacy project-level sdlc scaffolding written by older versions of `sdlc init`.
pub fn migrate_legacy_project_scaffolding(root: &Path) -> anyhow::Result<()> {
    let claude_files: Vec<String> = commands::ALL_COMMANDS
        .iter()
        .map(|c| c.claude_filename())
        .collect();
    let claude_refs: Vec<&str> = claude_files.iter().map(String::as_str).collect();

    let gemini_toml_files: Vec<String> = commands::ALL_COMMANDS
        .iter()
        .map(|c| c.gemini_filename())
        .collect();
    let gemini_md_files: Vec<String> = commands::ALL_COMMANDS
        .iter()
        .map(|c| c.claude_filename())
        .collect();
    let mut gemini_refs: Vec<&str> = gemini_toml_files.iter().map(String::as_str).collect();
    gemini_refs.extend(gemini_md_files.iter().map(String::as_str));

    let opencode_files: Vec<String> = commands::ALL_COMMANDS
        .iter()
        .map(|c| c.opencode_filename())
        .collect();
    let opencode_refs: Vec<&str> = opencode_files.iter().map(String::as_str).collect();

    let skill_files: Vec<String> = commands::ALL_COMMANDS
        .iter()
        .map(|c| format!("{}/SKILL.md", c.skill_dirname()))
        .collect();
    let skill_refs: Vec<&str> = skill_files.iter().map(String::as_str).collect();

    remove_if_exists(&paths::claude_commands_dir(root), &claude_refs)?;
    remove_if_exists(&paths::gemini_commands_dir(root), &gemini_refs)?;
    remove_if_exists(&paths::opencode_commands_dir(root), &opencode_refs)?;
    remove_if_exists(&root.join(".opencode/commands"), &opencode_refs)?;
    remove_if_exists(&paths::codex_skills_dir(root), &skill_refs)?;
    remove_if_exists(&root.join(".codex/commands"), &claude_refs)?;

    Ok(())
}

/// Install (or refresh) the SDLC core tool suite into `.sdlc/tools/`.
///
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
        ("agent.ts", TOOL_SHARED_AGENT_TS),
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

    // Dev-driver tool
    let dd_dir = paths::tool_dir(root, "dev-driver");
    io::ensure_dir(&dd_dir)?;

    // dev-driver/tool.ts — always overwrite (managed content)
    let dd_script = paths::tool_script(root, "dev-driver");
    let existed = dd_script.exists();
    io::atomic_write(&dd_script, TOOL_DEV_DRIVER_TS.as_bytes())
        .context("failed to write dev-driver/tool.ts")?;
    println!(
        "  {}: .sdlc/tools/dev-driver/tool.ts",
        if existed { "updated" } else { "created" }
    );

    // dev-driver/README.md — write-if-missing (user may annotate)
    let dd_readme = paths::tool_readme(root, "dev-driver");
    let created = io::write_if_missing(&dd_readme, TOOL_DEV_DRIVER_README_MD.as_bytes())
        .context("failed to write dev-driver/README.md")?;
    println!(
        "  {}: .sdlc/tools/dev-driver/README.md",
        if created { "created" } else { "exists " }
    );

    // Telegram-recap tool
    let tr_dir = paths::tool_dir(root, "telegram-recap");
    io::ensure_dir(&tr_dir)?;

    // telegram-recap/tool.ts — always overwrite (managed content)
    let tr_script = paths::tool_script(root, "telegram-recap");
    let existed = tr_script.exists();
    io::atomic_write(&tr_script, TOOL_TELEGRAM_RECAP_TS.as_bytes())
        .context("failed to write telegram-recap/tool.ts")?;
    println!(
        "  {}: .sdlc/tools/telegram-recap/tool.ts",
        if existed { "updated" } else { "created" }
    );

    // telegram-recap/config.yaml — write-if-missing (user may customize)
    let tr_config = paths::tool_config(root, "telegram-recap");
    let created = io::write_if_missing(&tr_config, TOOL_TELEGRAM_RECAP_CONFIG_YAML.as_bytes())
        .context("failed to write telegram-recap/config.yaml")?;
    println!(
        "  {}: .sdlc/tools/telegram-recap/config.yaml",
        if created { "created" } else { "exists " }
    );

    // telegram-recap/README.md — write-if-missing
    let tr_readme = paths::tool_readme(root, "telegram-recap");
    let created = io::write_if_missing(&tr_readme, TOOL_TELEGRAM_RECAP_README_MD.as_bytes())
        .context("failed to write telegram-recap/README.md")?;
    println!(
        "  {}: .sdlc/tools/telegram-recap/README.md",
        if created { "created" } else { "exists " }
    );

    // Static tools.md — overwrite (sdlc tool sync will regenerate from live metadata)
    let manifest_path = paths::tools_manifest_path(root);
    io::atomic_write(&manifest_path, TOOL_STATIC_TOOLS_MD.as_bytes())
        .context("failed to write tools/tools.md")?;

    // .gitignore — ensure index dirs are excluded
    append_gitignore_entry(root, ".sdlc/tools/*/index/")?;

    // .gitignore — binary runtime databases (redb) must never be committed
    append_gitignore_entry(root, ".sdlc/telemetry.redb")?;
    append_gitignore_entry(root, ".sdlc/orchestrator.redb")?;

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
