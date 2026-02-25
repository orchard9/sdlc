use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::config::{Config, PlatformArg};
use std::path::Path;

#[derive(Subcommand)]
pub enum PlatformSubcommand {
    /// List all configured platform commands
    List,
    /// Run a platform command (e.g. deploy, logs, dev start)
    #[command(external_subcommand)]
    External(Vec<String>),
}

pub fn run(root: &Path, subcmd: PlatformSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        PlatformSubcommand::List => list(root, json),
        PlatformSubcommand::External(args) => dispatch(root, &args),
    }
}

fn list(root: &Path, json: bool) -> anyhow::Result<()> {
    let config = Config::load(root).context("failed to load config")?;
    let platform = config
        .platform
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("no platform commands configured in .sdlc/config.yaml\nRun: sdlc init --platform <name>"))?;

    if json {
        let items: Vec<serde_json::Value> = platform
            .commands
            .iter()
            .map(|(name, cmd)| {
                let subs: Vec<&str> = cmd.subcommands.keys().map(|s| s.as_str()).collect();
                serde_json::json!({
                    "name": name,
                    "description": cmd.description,
                    "script": cmd.script,
                    "subcommands": subs,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if platform.commands.is_empty() {
        println!("No platform commands configured.");
        return Ok(());
    }

    let mut rows: Vec<Vec<String>> = platform
        .commands
        .iter()
        .map(|(name, cmd)| {
            let detail = if cmd.subcommands.is_empty() {
                cmd.script.clone()
            } else {
                let subs: Vec<&str> = cmd.subcommands.keys().map(|s| s.as_str()).collect();
                format!("subcommands: {}", subs.join(", "))
            };
            vec![name.clone(), cmd.description.clone(), detail]
        })
        .collect();
    rows.sort_by(|a, b| a[0].cmp(&b[0]));
    print_table(&["COMMAND", "DESCRIPTION", "SCRIPT / SUBCOMMANDS"], rows);
    Ok(())
}

fn dispatch(root: &Path, args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        anyhow::bail!("no platform command specified; run 'sdlc platform list'");
    }

    let config = Config::load(root).context("failed to load config")?;
    let platform = config.platform.as_ref().ok_or_else(|| {
        anyhow::anyhow!(
            "no platform commands configured in .sdlc/config.yaml\nRun: sdlc init --platform <name>"
        )
    })?;

    let cmd_name = &args[0];
    let cmd_config = platform.commands.get(cmd_name.as_str()).ok_or_else(|| {
        anyhow::anyhow!(
            "unknown platform command '{}'; run 'sdlc platform list' to see available commands",
            cmd_name
        )
    })?;

    // Subcommand dispatch (e.g. `sdlc platform dev start`)
    if !cmd_config.subcommands.is_empty() {
        let sub_name = args.get(1).ok_or_else(|| {
            let available: Vec<&str> =
                cmd_config.subcommands.keys().map(|s| s.as_str()).collect();
            anyhow::anyhow!(
                "command '{}' requires a subcommand: {}",
                cmd_name,
                available.join(", ")
            )
        })?;
        let script = cmd_config.subcommands.get(sub_name.as_str()).ok_or_else(|| {
            let available: Vec<&str> =
                cmd_config.subcommands.keys().map(|s| s.as_str()).collect();
            anyhow::anyhow!(
                "unknown subcommand '{}' for '{}'; available: {}",
                sub_name,
                cmd_name,
                available.join(", ")
            )
        })?;
        let script_args: Vec<&str> = args[2..].iter().map(|s| s.as_str()).collect();
        return exec_script(root, script, &script_args);
    }

    // Positional arg validation
    let script_args: Vec<&str> = args[1..].iter().map(|s| s.as_str()).collect();
    validate_args(&cmd_config.args, &script_args, cmd_name)?;

    exec_script(root, &cmd_config.script, &script_args)
}

fn validate_args(
    specs: &[PlatformArg],
    provided: &[&str],
    cmd_name: &str,
) -> anyhow::Result<()> {
    for (i, spec) in specs.iter().enumerate() {
        match provided.get(i) {
            None if spec.required => {
                anyhow::bail!(
                    "missing required argument '{}' for '{}'\nUsage: sdlc platform {} {}",
                    spec.name,
                    cmd_name,
                    cmd_name,
                    specs.iter().map(|a| format!("<{}>", a.name)).collect::<Vec<_>>().join(" ")
                );
            }
            Some(val) if !spec.choices.is_empty() && !spec.choices.contains(&val.to_string()) => {
                anyhow::bail!(
                    "invalid value '{}' for '{}'; valid choices: {}",
                    val,
                    spec.name,
                    spec.choices.join(", ")
                );
            }
            _ => {}
        }
    }
    Ok(())
}

fn exec_script(root: &Path, script: &str, args: &[&str]) -> anyhow::Result<()> {
    if script.is_empty() {
        anyhow::bail!("no script configured for this command");
    }
    let script_path = root.join(script);
    if !script_path.exists() {
        anyhow::bail!(
            "platform script not found: {}\nRun 'sdlc init --platform <name>' to scaffold scripts",
            script_path.display()
        );
    }

    let status = std::process::Command::new("sh")
        .arg(&script_path)
        .args(args)
        .status()
        .with_context(|| format!("failed to execute '{}'", script_path.display()))?;

    if !status.success() {
        std::process::exit(status.code().unwrap_or(1));
    }
    Ok(())
}
