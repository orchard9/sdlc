use crate::cmd::tunnel::{generate_token, print_tunnel_info, Tunnel, TunnelError};
use crate::output::print_table;
use anyhow::{anyhow, Result};
use clap::Subcommand;
use sdlc_core::{config::Config, ui_registry};
use sdlc_server::auth::TunnelConfig;
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
        /// Open a public tunnel and print a QR code for remote access (requires cloudflared)
        #[arg(long)]
        tunnel: bool,
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
    tunnel: bool,
) -> Result<()> {
    match subcommand {
        None => run_start(root, port, no_open, tunnel),
        Some(UiSubcommand::Start {
            port: p,
            no_open: n,
            tunnel: t,
        }) => run_start(root, p, n, t),
        Some(UiSubcommand::List) => run_list(),
        Some(UiSubcommand::Kill { name }) => run_kill(name.as_deref(), root),
        Some(UiSubcommand::Open { name }) => run_open(name.as_deref(), root),
    }
}

// ---------------------------------------------------------------------------
// start
// ---------------------------------------------------------------------------

fn run_start(root: &Path, port: u16, no_open: bool, use_tunnel: bool) -> Result<()> {
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

            // Start cloudflared tunnel.
            let tun = match Tunnel::start(actual_port).await {
                Ok(t) => t,
                Err(TunnelError::NotFound) => {
                    let _ = record_clone.remove();
                    return Err(anyhow!("{}", TunnelError::NotFound));
                }
                Err(e) => {
                    let _ = record_clone.remove();
                    return Err(anyhow!("{e}"));
                }
            };

            let token = generate_token();
            let tunnel_config = TunnelConfig::with_token(token.clone());

            print_tunnel_info(&name, actual_port, &tun.url, &token);

            let result = tokio::select! {
                res = sdlc_server::serve_on(root_buf, listener, false, tunnel_config) => res,
                _ = tokio::signal::ctrl_c() => Ok(()),
            };

            tun.stop().await;
            result
        } else {
            println!("SDLC UI for '{name}' → {local_url}  (PID {pid})");

            tokio::select! {
                res = sdlc_server::serve_on(root_buf, listener, !no_open, TunnelConfig::none()) => res,
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
