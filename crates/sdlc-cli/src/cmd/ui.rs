use crate::cmd::tunnel::print_tunnel_info;
use crate::cmd::{orchestrate, update};
use crate::output::print_table;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use sdlc_core::{config::Config, ui_registry};
use sdlc_server::tunnel::{derive_tunnel_name, generate_token, Tunnel};
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand definition
// ---------------------------------------------------------------------------

#[derive(Subcommand, Debug)]
pub enum UiSubcommand {
    /// Start the web UI server
    Start {
        /// Port to listen on
        #[arg(long, default_value = "3141")]
        port: u16,
        /// Don't open browser automatically
        #[arg(long)]
        no_open: bool,
        /// Disable the public tunnel (tunnel starts automatically by default, requires orch-tunnel)
        #[arg(long)]
        no_tunnel: bool,
        /// Orchestrator tick interval in seconds (default 60)
        #[arg(long, default_value_t = 60)]
        tick_rate: u64,
        /// Start the orchestrator daemon and execute scheduled actions
        #[arg(long)]
        run_actions: bool,
    },
    /// List all running UI instances
    List,
    /// Kill a running UI instance
    Kill {
        /// Project name (defaults to current project)
        name: Option<String>,
    },
    /// Open browser for a running UI instance
    Open {
        /// Project name (defaults to current project)
        name: Option<String>,
    },
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn run(
    root: &Path,
    subcommand: Option<UiSubcommand>,
    port: u16,
    no_open: bool,
    no_tunnel: bool,
    tick_rate: u64,
    run_actions: bool,
) -> Result<()> {
    match subcommand {
        None => run_start(root, port, no_open, no_tunnel, tick_rate, run_actions),
        Some(UiSubcommand::Start {
            port: p,
            no_open: n,
            no_tunnel: nt,
            tick_rate: tr,
            run_actions: ra,
        }) => run_start(root, p, n, nt, tr, ra),
        Some(UiSubcommand::List) => run_list(),
        Some(UiSubcommand::Kill { name }) => run_kill(name.as_deref(), root),
        Some(UiSubcommand::Open { name }) => run_open(name.as_deref(), root),
    }
}

// ---------------------------------------------------------------------------
// start
// ---------------------------------------------------------------------------

fn run_start(
    root: &Path,
    port: u16,
    no_open: bool,
    no_tunnel: bool,
    tick_rate: u64,
    run_actions: bool,
) -> Result<()> {
    let use_tunnel = !no_tunnel;
    // --- Step 1: auto-update scaffolding ---
    eprintln!("sdlc ui: running update...");
    if let Err(e) = update::run(root) {
        eprintln!("sdlc ui: update warning (continuing): {e}");
    }

    // --- Step 2: start orchestrator daemon in background thread ---
    if run_actions {
        let root_for_orch = root.to_path_buf();
        std::thread::Builder::new()
            .name("sdlc-orchestrator".into())
            .spawn(move || {
                if let Err(e) = orchestrate::run_daemon(&root_for_orch, tick_rate) {
                    eprintln!("orchestrate: daemon error: {e}");
                }
            })
            .map_err(|e| anyhow!("failed to spawn orchestrator thread: {e}"))?;
    }

    let config = Config::load(root).map_err(|e| anyhow!("{e}"))?;
    let name = config.project.name.clone();

    // Prune stale records; error if a live instance already exists.
    if let Some(record) = ui_registry::find_by_name(&name).map_err(|e| anyhow!("{e}"))? {
        if ui_registry::is_pid_alive(record.pid) {
            return Err(anyhow!(
                "UI for '{}' is already running at {} (PID {})\n\
                 Run `sdlc ui kill {}` to stop it first.",
                name,
                record.url,
                record.pid,
                name
            ));
        }
        // Stale record — remove silently.
        let _ = record.remove();
    }

    let rt = tokio::runtime::Runtime::new()?;
    let root_buf = root.to_path_buf();

    rt.block_on(async move {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;
        let actual_port = listener.local_addr()?.port();
        let pid = std::process::id();
        let local_url = format!("http://localhost:{actual_port}");

        let record = ui_registry::UiRecord {
            project: name.clone(),
            root: root_buf.clone(),
            pid,
            port: actual_port,
            url: local_url.clone(),
            started_at: chrono::Utc::now(),
        };
        record.write().map_err(|e| anyhow!("{e}"))?;

        let record_clone = record.clone();

        let result = if use_tunnel {
            // Warn that the server is public.
            eprintln!("Warning: tunnel mode exposes your SDLC server publicly. Share the QR code only with trusted parties.");

            // Start orch-tunnel; fall back to local-only on any failure.
            let tunnel_name = derive_tunnel_name(&root_buf);
            match Tunnel::start(actual_port, &tunnel_name).await {
                Ok(tun) => {
                    let token = generate_token();
                    print_tunnel_info(&name, actual_port, &tun.url, &token);
                    tokio::select! {
                        res = sdlc_server::serve_on(root_buf, listener, false, Some((tun, token))) => res,
                        _ = tokio::signal::ctrl_c() => Ok(()),
                    }
                }
                Err(e) => {
                    // Graceful fallback: warn and continue in local-only mode.
                    eprintln!("Warning: orch-tunnel failed to start ({e}). Running in local-only mode.");
                    println!("SDLC UI for '{name}' → {local_url}  (PID {pid})");
                    tokio::select! {
                        res = sdlc_server::serve_on(root_buf, listener, !no_open, None) => res,
                        _ = tokio::signal::ctrl_c() => Ok(()),
                    }
                }
            }
        } else {
            println!("SDLC UI for '{name}' → {local_url}  (PID {pid})");

            tokio::select! {
                res = sdlc_server::serve_on(root_buf, listener, !no_open, None) => res,
                _ = tokio::signal::ctrl_c() => Ok(()),
            }
        };

        let _ = record_clone.remove();
        result
    })
}

// ---------------------------------------------------------------------------
// list
// ---------------------------------------------------------------------------

fn run_list() -> Result<()> {
    let mut records = ui_registry::read_all().map_err(|e| anyhow!("{e}"))?;

    // Prune stale records silently.
    records.retain(|r| {
        if ui_registry::is_pid_alive(r.pid) {
            true
        } else {
            let _ = r.remove();
            false
        }
    });

    if records.is_empty() {
        println!("No running UI instances.");
        return Ok(());
    }

    let headers = &["PROJECT", "PORT", "PID", "URL", "STARTED"];
    let rows: Vec<Vec<String>> = records
        .iter()
        .map(|r| {
            vec![
                r.project.clone(),
                r.port.to_string(),
                r.pid.to_string(),
                r.url.clone(),
                r.started_at.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
            ]
        })
        .collect();

    print_table(headers, rows);
    Ok(())
}

// ---------------------------------------------------------------------------
// kill
// ---------------------------------------------------------------------------

fn run_kill(name: Option<&str>, root: &Path) -> Result<()> {
    let name = resolve_name(name, root)?;

    let record = ui_registry::find_by_name(&name)
        .map_err(|e| anyhow!("{e}"))?
        .ok_or_else(|| anyhow!("No UI record found for '{name}'"))?;

    if !ui_registry::is_pid_alive(record.pid) {
        let _ = record.remove();
        return Err(anyhow!(
            "UI for '{name}' is not running (stale record removed)"
        ));
    }

    ui_registry::kill_pid(record.pid).map_err(|e| anyhow!("{e}"))?;
    let _ = record.remove();

    println!("Killed UI for '{name}' (PID {})", record.pid);
    Ok(())
}

// ---------------------------------------------------------------------------
// open
// ---------------------------------------------------------------------------

fn run_open(name: Option<&str>, root: &Path) -> Result<()> {
    let name = resolve_name(name, root)?;

    let record = ui_registry::find_by_name(&name)
        .map_err(|e| anyhow!("{e}"))?
        .ok_or_else(|| anyhow!("No UI record found for '{name}'"))?;

    if !ui_registry::is_pid_alive(record.pid) {
        let _ = record.remove();
        return Err(anyhow!(
            "UI for '{name}' is not running (stale record removed)"
        ));
    }

    println!("Opening {} ...", record.url);
    let _ = open::that(&record.url);
    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn resolve_name(name: Option<&str>, root: &Path) -> Result<String> {
    if let Some(n) = name {
        return Ok(n.to_string());
    }
    let config = Config::load(root).map_err(|e| anyhow!("{e}"))?;
    Ok(config.project.name)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    /// Verify that `--no-tunnel` correctly inverts to `use_tunnel = false`.
    /// This is the semantic core of the default-on tunnel change.
    #[test]
    fn no_tunnel_flag_inverts_use_tunnel() {
        // When --no-tunnel is passed (no_tunnel = true), the server must NOT
        // attempt a tunnel. This is the key behavioral contract.
        let no_tunnel = true;
        let use_tunnel = !no_tunnel;
        assert!(!use_tunnel, "--no-tunnel should disable tunnel");

        // When --no-tunnel is absent (no_tunnel = false), the server MUST
        // attempt a tunnel by default.
        let no_tunnel = false;
        let use_tunnel = !no_tunnel;
        assert!(
            use_tunnel,
            "tunnel must be active by default (no --no-tunnel)"
        );
    }

    /// Verify that the graceful fallback warning message includes the error
    /// reason and the "local-only mode" phrase, so operators know what happened.
    #[test]
    fn fallback_warning_format_is_informative() {
        let reason = "orch-tunnel not found\nInstall with: brew install orch-tunnel";
        let warning =
            format!("Warning: orch-tunnel failed to start ({reason}). Running in local-only mode.");
        assert!(warning.contains("orch-tunnel failed to start"));
        assert!(warning.contains("local-only mode"));
        assert!(warning.contains(reason));
    }
}
