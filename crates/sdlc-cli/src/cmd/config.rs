use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::config::Config;
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand types
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Validate the config for common mistakes
    Validate,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcmd: ConfigSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ConfigSubcommand::Validate => validate(root, json),
    }
}

// ---------------------------------------------------------------------------
// validate
// ---------------------------------------------------------------------------

fn validate(root: &Path, json: bool) -> anyhow::Result<()> {
    use sdlc_core::config::WarnLevel;

    let config = Config::load(root).context("failed to load config")?;
    let warnings = config.validate();

    if json {
        let value = serde_json::json!({
            "warnings": warnings,
        });
        print_json(&value)?;
    } else if warnings.is_empty() {
        println!("Config is valid. No warnings.");
    } else {
        for w in &warnings {
            let prefix = match w.level {
                WarnLevel::Warning => "warning",
                WarnLevel::Error => "error",
            };
            println!("[{prefix}] {}", w.message);
        }
    }

    let has_errors = warnings.iter().any(|w| w.level == WarnLevel::Error);
    if has_errors {
        anyhow::bail!("config validation found errors");
    }

    Ok(())
}
