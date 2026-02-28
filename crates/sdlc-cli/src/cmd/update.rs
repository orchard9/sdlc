use anyhow::Context;
use sdlc_core::{config::Config, paths};
use std::path::Path;

use super::init::{
    install_user_scaffolding, migrate_legacy_project_scaffolding, stamp_sdlc_version,
    write_agents_md, write_core_tools, write_guidance_md, SDLC_BINARY_VERSION,
};

/// `sdlc update` — refresh agent scaffolding and stamp the current binary version.
///
/// Requires an already-initialized project (`.sdlc/config.yaml` must exist).
/// Does not modify feature state, milestone state, or user-edited config settings.
pub fn run(root: &Path) -> anyhow::Result<()> {
    // Require an initialized project
    if !paths::config_path(root).exists() {
        anyhow::bail!(
            "not initialized: run 'sdlc init' first (no .sdlc/config.yaml found in {})",
            root.display()
        );
    }

    let config = Config::load(root).context("failed to load config.yaml")?;
    let project_name = config.project.name.clone();

    let previous_version = config
        .sdlc_version
        .as_deref()
        .unwrap_or("unversioned")
        .to_string();

    println!("Updating SDLC scaffolding in: {}", root.display());
    println!("  previous: {previous_version}  →  current: {SDLC_BINARY_VERSION}");

    // Ensure any new .sdlc/ directories introduced in later versions exist
    ensure_sdlc_dirs(root)?;

    // Write / refresh engineering guidance
    write_guidance_md(root)?;

    // Write / refresh core tool suite (.sdlc/tools/)
    println!("\nInstalling core tool suite:");
    write_core_tools(root)?;

    // Refresh all user-level agent commands and skills
    println!("\nInstalling user-level command scaffolding:");
    install_user_scaffolding()?;

    // Remove deprecated project-level scaffolding files
    migrate_legacy_project_scaffolding(root)?;

    // Refresh the SDLC section in AGENTS.md (creates, updates markers, or migrates legacy)
    println!();
    write_agents_md(root, &project_name)?;

    // Stamp the current binary version into config.yaml
    stamp_sdlc_version(root)?;

    println!("\nSDLC updated to v{SDLC_BINARY_VERSION}.");

    Ok(())
}

/// Ensure all expected `.sdlc/` subdirectories exist.
/// Idempotent — safe to run on any version of an initialized project.
fn ensure_sdlc_dirs(root: &Path) -> anyhow::Result<()> {
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
        sdlc_core::io::ensure_dir(&root.join(dir))?;
    }
    Ok(())
}
