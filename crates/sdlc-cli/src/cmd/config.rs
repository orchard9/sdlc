use crate::output::print_json;
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::config::{AgentBackend, Config};
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand types
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum ConfigSubcommand {
    /// Inspect and modify agent backend routing
    Agent {
        #[command(subcommand)]
        subcommand: AgentSubcommand,
    },

    /// Validate the config for common mistakes
    Validate,
}

#[derive(Subcommand)]
pub enum AgentSubcommand {
    /// Show current agent backend configuration
    Show,

    /// Set the default agent backend
    SetDefault {
        /// Backend type: claude, xadk, or human
        #[arg(long = "type", value_name = "TYPE")]
        backend_type: String,
        /// Model name (for claude backend)
        #[arg(long)]
        model: Option<String>,
        /// Agent ID (for xadk backend)
        #[arg(long)]
        agent_id: Option<String>,
        /// Timeout in minutes (for claude backend)
        #[arg(long)]
        timeout: Option<u32>,
    },

    /// Set a per-action backend override
    SetAction {
        /// Action name (snake_case, e.g. create_spec)
        action: String,
        /// Backend type: claude, xadk, or human
        #[arg(long = "type", value_name = "TYPE")]
        backend_type: String,
        /// Model name (for claude backend)
        #[arg(long)]
        model: Option<String>,
        /// Agent ID (for xadk backend)
        #[arg(long)]
        agent_id: Option<String>,
    },

    /// Clear all per-action backend overrides
    Reset,
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcmd: ConfigSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        ConfigSubcommand::Agent { subcommand } => run_agent(root, subcommand, json),
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

    let has_errors = warnings
        .iter()
        .any(|w| w.level == WarnLevel::Error);
    if has_errors {
        anyhow::bail!("config validation found errors");
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Agent subcommand dispatch
// ---------------------------------------------------------------------------

fn run_agent(root: &Path, subcmd: AgentSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        AgentSubcommand::Show => show(root, json),
        AgentSubcommand::SetDefault {
            backend_type,
            model,
            agent_id,
            timeout,
        } => set_default(root, &backend_type, model, agent_id, timeout),
        AgentSubcommand::SetAction {
            action,
            backend_type,
            model,
            agent_id,
        } => set_action(root, &action, &backend_type, model, agent_id),
        AgentSubcommand::Reset => reset(root),
    }
}

// ---------------------------------------------------------------------------
// show
// ---------------------------------------------------------------------------

fn show(root: &Path, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let agents = &config.agents;

    if json {
        let value = serde_json::json!({
            "default": agents.default,
            "actions": agents.actions,
        });
        print_json(&value)?;
        return Ok(());
    }

    println!("Default backend:  {}", backend_display(&agents.default));
    if agents.actions.is_empty() {
        println!("Action overrides: (none)");
    } else {
        println!("Action overrides:");
        let mut actions: Vec<(&String, &AgentBackend)> = agents.actions.iter().collect();
        actions.sort_by_key(|(k, _)| k.as_str());
        for (action, backend) in actions {
            println!("  {:<20} {}", action, backend_display(backend));
        }
    }
    Ok(())
}

fn backend_display(backend: &AgentBackend) -> String {
    match backend {
        AgentBackend::ClaudeAgentSdk {
            model,
            timeout_minutes,
            ..
        } => {
            let timeout_str = timeout_minutes
                .map(|t| format!(", timeout: {t}m"))
                .unwrap_or_default();
            format!("claude_agent_sdk ({model}{timeout_str})")
        }
        AgentBackend::Xadk { agent_id, .. } => format!("xadk (agent: {agent_id})"),
        AgentBackend::Human => "human".to_string(),
    }
}

// ---------------------------------------------------------------------------
// parse_backend helper
// ---------------------------------------------------------------------------

fn parse_backend(
    type_str: &str,
    model: Option<String>,
    agent_id: Option<String>,
    timeout: Option<u32>,
) -> anyhow::Result<AgentBackend> {
    match type_str {
        "claude" => Ok(AgentBackend::ClaudeAgentSdk {
            model: model.unwrap_or_else(|| "claude-opus-4-6".to_string()),
            allowed_tools: vec![
                "Read".to_string(),
                "Write".to_string(),
                "Edit".to_string(),
                "Bash".to_string(),
                "Glob".to_string(),
                "Grep".to_string(),
            ],
            permission_mode: None,
            timeout_minutes: timeout,
        }),
        "xadk" => {
            let id = agent_id
                .ok_or_else(|| anyhow::anyhow!("--agent-id is required for xadk backend"))?;
            Ok(AgentBackend::Xadk {
                agent_id: id,
                read_agents_md: false,
            })
        }
        "human" => Ok(AgentBackend::Human),
        other => {
            anyhow::bail!("unknown backend type '{other}'; valid: claude, xadk, human")
        }
    }
}

// ---------------------------------------------------------------------------
// set-default
// ---------------------------------------------------------------------------

fn set_default(
    root: &Path,
    backend_type: &str,
    model: Option<String>,
    agent_id: Option<String>,
    timeout: Option<u32>,
) -> anyhow::Result<()> {
    let mut config = Config::load(root).context("failed to load config")?;
    let backend = parse_backend(backend_type, model, agent_id, timeout)?;
    config.agents.default = backend;
    config.save(root).context("failed to save config")?;
    println!("Default backend updated to '{backend_type}'.");
    Ok(())
}

// ---------------------------------------------------------------------------
// set-action
// ---------------------------------------------------------------------------

fn set_action(
    root: &Path,
    action: &str,
    backend_type: &str,
    model: Option<String>,
    agent_id: Option<String>,
) -> anyhow::Result<()> {
    let mut config = Config::load(root).context("failed to load config")?;
    let backend = parse_backend(backend_type, model, agent_id, None)?;
    config.agents.actions.insert(action.to_string(), backend);
    config.save(root).context("failed to save config")?;
    println!("Action '{action}' now uses '{backend_type}' backend.");
    Ok(())
}

// ---------------------------------------------------------------------------
// reset
// ---------------------------------------------------------------------------

fn reset(root: &Path) -> anyhow::Result<()> {
    let mut config = Config::load(root).context("failed to load config")?;
    config.agents.actions.clear();
    config.save(root).context("failed to save config")?;
    println!("All action overrides cleared.");
    Ok(())
}
