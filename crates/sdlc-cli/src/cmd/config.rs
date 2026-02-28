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

    /// Show the project configuration as JSON
    Show {
        /// Output as JSON (default when flag not set: human-readable)
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcmd: ConfigSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ConfigSubcommand::Validate => validate(root, json),
        ConfigSubcommand::Show { json: show_json } => show_config(root, json || show_json),
    }
}

// ---------------------------------------------------------------------------
// validate
// ---------------------------------------------------------------------------

fn show_config(root: &Path, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;

    if json {
        print_json(&config)?;
    } else {
        // Human-readable: pretty-print as YAML
        let yaml = serde_yaml::to_string(&config).context("failed to serialize config")?;
        print!("{yaml}");
    }

    Ok(())
}

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
