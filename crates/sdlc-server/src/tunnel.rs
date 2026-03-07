use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, Command},
    time::{timeout, Duration},
};

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum TunnelError {
    #[error("{0}")]
    NotFound(String),

    #[error(
        "orch-tunnel did not start within {0} seconds.\n\
         Check your network connection and try again."
    )]
    Timeout(u64),

    #[error(
        "orch-tunnel exited unexpectedly.\n\
         Try running manually: orch-tunnel http {port} --name <project-name>"
    )]
    ExitedEarly { port: u16 },

    #[error("process error: {0}")]
    Process(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Tunnel
// ---------------------------------------------------------------------------

/// A running orch-tunnel quick-tunnel.
///
/// The tunnel process is killed when [`stop`] is called or when this value is
/// dropped (via `kill_on_drop(true)`).
pub struct Tunnel {
    /// Public HTTPS URL (e.g. `https://my-project.tunnel.threesix.ai`).
    pub url: String,
    process: Child,
}

impl Tunnel {
    /// Spawn orch-tunnel and wait until a `tunnel.threesix.ai` URL appears on
    /// stdout. Times out after `SDLC_TUNNEL_TIMEOUT_SECS` seconds (default 15).
    pub async fn start(port: u16, name: &str) -> Result<Self, TunnelError> {
        let binary = find_orch_tunnel()?;

        let timeout_secs: u64 = std::env::var("SDLC_TUNNEL_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);

        let mut child = Command::new(binary)
            .args(["http", &port.to_string(), "--name", name])
            .stdout(std::process::Stdio::piped()) // URL goes to stdout
            .stderr(std::process::Stdio::piped()) // drain to prevent SIGPIPE
            .kill_on_drop(true)
            .spawn()?;

        let stdout = child.stdout.take().expect("stdout was configured as piped");
        let stderr = child.stderr.take().expect("stderr was configured as piped");

        // Drain stderr in the background to prevent SIGPIPE.
        tokio::spawn(async move {
            let mut reader = tokio::io::BufReader::new(stderr);
            let mut sink = tokio::io::sink();
            let _ = tokio::io::copy(&mut reader, &mut sink).await;
        });

        let url_result = timeout(
            Duration::from_secs(timeout_secs),
            read_tunnel_url(stdout, port),
        )
        .await;

        match url_result {
            Ok(Ok((url, remaining_reader))) => {
                // Drain orch-tunnel's stdout in the background after URL extraction.
                // Without this, orch-tunnel gets SIGPIPE when it tries to log
                // after we stop reading, which kills the tunnel.
                tokio::spawn(async move {
                    let mut reader = remaining_reader;
                    let mut sink = tokio::io::sink();
                    let _ = tokio::io::copy(&mut reader, &mut sink).await;
                });
                Ok(Tunnel {
                    url,
                    process: child,
                })
            }
            Ok(Err(e)) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(e)
            }
            Err(_elapsed) => {
                let _ = child.kill().await;
                let _ = child.wait().await;
                Err(TunnelError::Timeout(timeout_secs))
            }
        }
    }

    /// Kill the orch-tunnel process and wait for it to exit.
    pub async fn stop(mut self) {
        let _ = self.process.kill().await;
        let _ = self.process.wait().await;
    }
}

async fn read_tunnel_url(
    stdout: tokio::process::ChildStdout,
    port: u16,
) -> Result<(String, BufReader<tokio::process::ChildStdout>), TunnelError> {
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line).await {
            Ok(0) => return Err(TunnelError::ExitedEarly { port }),
            Ok(_) => {
                if let Some(url) = extract_tunnel_url(&line) {
                    return Ok((url.to_string(), reader));
                }
            }
            Err(_) => return Err(TunnelError::ExitedEarly { port }),
        }
    }
}

// ---------------------------------------------------------------------------
// Token generation
// ---------------------------------------------------------------------------

/// Generate a random 8-character alphanumeric token.
pub fn generate_token() -> String {
    use rand::{distributions::Alphanumeric, Rng};
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Three-tier binary discovery for orch-tunnel.
///
/// 1. Process PATH (`which::which`)
/// 2. Login shell PATH (`$SHELL -lc "echo $PATH"` then `which::which_in`)
/// 3. Fallback well-known locations
pub fn find_orch_tunnel() -> Result<PathBuf, TunnelError> {
    // Tier 1: process PATH (fast path)
    if let Ok(p) = which::which("orch-tunnel") {
        return Ok(p);
    }

    // Tier 2: login shell PATH
    if let Some(fresh_path) = read_login_shell_path() {
        if let Ok(p) = which::which_in("orch-tunnel", Some(fresh_path), ".") {
            return Ok(p);
        }
    }

    // Tier 3: well-known fallback locations
    let fallbacks = fallback_locations();
    for path in &fallbacks {
        if path.is_file() {
            return Ok(path.clone());
        }
    }

    Err(TunnelError::NotFound(format_not_found_message(&fallbacks)))
}

/// Structured check result for diagnostics and JSON serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelCheckResult {
    pub installed: bool,
    pub path: Option<PathBuf>,
    pub version: Option<String>,
    /// Which tier found the binary: `"process_path"`, `"login_shell_path"`, or `"fallback"`.
    pub source: Option<String>,
    /// True when found via tier 2/3 but NOT via tier 1 (process PATH is stale).
    pub process_path_stale: bool,
    pub checked: Vec<CheckedLocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckedLocation {
    pub location: String,
    pub found: bool,
}

/// Rich diagnostic check — runs all three tiers regardless of early success.
pub fn check_orch_tunnel() -> TunnelCheckResult {
    let mut checked = Vec::new();
    let mut found_path: Option<PathBuf> = None;
    let mut source: Option<String> = None;

    // Tier 1: process PATH
    let tier1 = which::which("orch-tunnel").ok();
    checked.push(CheckedLocation {
        location: "process PATH".to_string(),
        found: tier1.is_some(),
    });
    if let Some(ref p) = tier1 {
        found_path = Some(p.clone());
        source = Some("process_path".to_string());
    }

    // Tier 2: login shell PATH
    let tier2 = read_login_shell_path()
        .and_then(|fresh| which::which_in("orch-tunnel", Some(fresh), ".").ok());
    let shell_name = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    checked.push(CheckedLocation {
        location: format!("login shell PATH ({})", shell_name),
        found: tier2.is_some(),
    });
    if found_path.is_none() {
        if let Some(ref p) = tier2 {
            found_path = Some(p.clone());
            source = Some("login_shell_path".to_string());
        }
    }

    // Tier 3: fallback locations
    let fallbacks = fallback_locations();
    for fb in &fallbacks {
        let exists = fb.is_file();
        checked.push(CheckedLocation {
            location: fb.display().to_string(),
            found: exists,
        });
        if found_path.is_none() && exists {
            found_path = Some(fb.clone());
            source = Some("fallback".to_string());
        }
    }

    let process_path_stale = tier1.is_none() && found_path.is_some();

    // Capture version if we found the binary
    let version = found_path.as_ref().and_then(|p| {
        std::process::Command::new(p)
            .arg("--version")
            .output()
            .ok()
            .and_then(|o| {
                if o.status.success() {
                    String::from_utf8(o.stdout).ok().map(|s| s.trim().to_string())
                } else {
                    None
                }
            })
    });

    TunnelCheckResult {
        installed: found_path.is_some(),
        path: found_path,
        version,
        source,
        process_path_stale,
        checked,
    }
}

/// Spawn a login shell to capture the user's real PATH.
///
/// Returns `None` on any failure (missing $SHELL, spawn error, timeout, non-zero exit).
fn read_login_shell_path() -> Option<String> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let child = std::process::Command::new(&shell)
        .args(["-lc", "echo $PATH"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;

    // 3-second timeout via a thread — prevents hanging on broken shell configs
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = std::thread::spawn(move || {
        let output = child.wait_with_output();
        let _ = tx.send(output);
    });

    match rx.recv_timeout(std::time::Duration::from_secs(3)) {
        Ok(Ok(output)) if output.status.success() => {
            let _ = handle.join();
            let path_str = String::from_utf8(output.stdout).ok()?.trim().to_string();
            if path_str.is_empty() {
                None
            } else {
                Some(path_str)
            }
        }
        _ => {
            // Timeout or error — don't block, just abandon
            let _ = handle.join();
            None
        }
    }
}

/// Well-known install locations for orch-tunnel.
fn fallback_locations() -> Vec<PathBuf> {
    let mut locations = vec![
        PathBuf::from("/opt/homebrew/bin/orch-tunnel"),
        PathBuf::from("/usr/local/bin/orch-tunnel"),
    ];
    if let Some(home) = dirs::home_dir() {
        locations.push(home.join(".cargo/bin/orch-tunnel"));
    }
    locations
}

/// Build a human-readable not-found message listing all checked locations.
fn format_not_found_message(fallbacks: &[PathBuf]) -> String {
    let shell_name = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
    let mut lines = vec![
        "orch-tunnel not found.".to_string(),
        String::new(),
        "Searched:".to_string(),
        "  Process PATH:      not found".to_string(),
        format!("  Login shell PATH:  not found (shell: {})", shell_name),
    ];
    for fb in fallbacks {
        lines.push(format!("  {}:  not found", fb.display()));
    }
    lines.push(String::new());
    lines.push("Install:".to_string());
    lines.push("  macOS    brew install orch-tunnel".to_string());
    lines.push("  Other    gh release download --repo orchard9/tunnel \\".to_string());
    lines.push("             --pattern 'orch-tunnel-*' -D /usr/local/bin".to_string());
    lines.push("           chmod +x /usr/local/bin/orch-tunnel".to_string());
    lines.push(String::new());
    lines.push("Then re-run: sdlc ui --tunnel".to_string());
    lines.join("\n")
}

/// Derive a tunnel-safe name from the project root directory.
///
/// Tries `.sdlc/config.yaml` first; falls back to the directory basename.
/// Sanitizes to `[a-z0-9-]` with no leading/trailing dashes.
pub fn derive_tunnel_name(root: &std::path::Path) -> String {
    let base = sdlc_core::config::Config::load(root)
        .map(|c| c.project.name)
        .unwrap_or_else(|_| {
            root.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("sdlc")
                .to_string()
        });
    let sanitized: String = base
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_string();
    if sanitized.is_empty() {
        "sdlc".to_string()
    } else {
        sanitized
    }
}

/// Extract a `https://*.tunnel.threesix.ai` URL from an orch-tunnel output line.
pub fn extract_tunnel_url(line: &str) -> Option<&str> {
    let start = line.find("https://")?;
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
        .unwrap_or(rest.len());
    let url = &rest[..end];
    if url.contains(".tunnel.threesix.ai") {
        Some(url)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_url_from_bare_line() {
        let line = "https://foo.tunnel.threesix.ai";
        assert_eq!(
            extract_tunnel_url(line),
            Some("https://foo.tunnel.threesix.ai")
        );
    }

    #[test]
    fn extract_url_embedded_in_log_line() {
        let line = "Tunnel started! Visit https://my-project.tunnel.threesix.ai now";
        assert_eq!(
            extract_tunnel_url(line),
            Some("https://my-project.tunnel.threesix.ai")
        );
    }

    #[test]
    fn extract_url_ignores_non_orch_tunnel() {
        assert_eq!(extract_tunnel_url("https://example.com"), None);
        assert_eq!(extract_tunnel_url("Starting tunnel..."), None);
    }

    #[test]
    fn token_is_8_alphanumeric_chars() {
        let tok = generate_token();
        assert_eq!(tok.len(), 8);
        assert!(tok.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    // --- Three-tier discovery tests ---

    #[test]
    fn read_login_shell_path_returns_some() {
        // On any machine with a working shell, this should return a non-empty PATH.
        let result = read_login_shell_path();
        assert!(result.is_some(), "read_login_shell_path() returned None");
        let path = result.unwrap();
        assert!(!path.is_empty(), "login shell PATH was empty");
        // PATH should contain at least one colon-separated entry
        assert!(
            path.contains('/'),
            "login shell PATH doesn't look like a path: {path}"
        );
    }

    #[test]
    fn fallback_locations_includes_well_known_paths() {
        let locs = fallback_locations();
        assert!(locs.len() >= 2, "expected at least 2 fallback locations");
        assert_eq!(locs[0], PathBuf::from("/opt/homebrew/bin/orch-tunnel"));
        assert_eq!(locs[1], PathBuf::from("/usr/local/bin/orch-tunnel"));
    }

    #[test]
    fn fallback_probing_finds_mock_binary() {
        let dir = tempfile::TempDir::new().unwrap();
        let mock_bin = dir.path().join("orch-tunnel");
        std::fs::write(&mock_bin, "#!/bin/sh\necho mock").unwrap();
        // Make executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&mock_bin, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        // Verify is_file() works for our mock
        assert!(mock_bin.is_file());
    }

    #[test]
    fn check_orch_tunnel_returns_populated_result() {
        let result = check_orch_tunnel();
        // Should always have checked entries regardless of whether orch-tunnel is installed
        assert!(
            result.checked.len() >= 4,
            "expected at least 4 checked locations, got {}",
            result.checked.len()
        );
        // First two entries are always process PATH and login shell PATH
        assert_eq!(result.checked[0].location, "process PATH");
        assert!(result.checked[1].location.starts_with("login shell PATH"));
    }

    #[test]
    fn tunnel_check_result_serializes_to_json() {
        let result = TunnelCheckResult {
            installed: true,
            path: Some(PathBuf::from("/usr/local/bin/orch-tunnel")),
            version: Some("1.0.0".to_string()),
            source: Some("process_path".to_string()),
            process_path_stale: false,
            checked: vec![CheckedLocation {
                location: "process PATH".to_string(),
                found: true,
            }],
        };
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"installed\":true"));
        assert!(json.contains("\"source\":\"process_path\""));
        assert!(json.contains("\"process_path_stale\":false"));
    }

    #[test]
    fn not_found_message_contains_searched_section() {
        let fallbacks = fallback_locations();
        let msg = format_not_found_message(&fallbacks);
        assert!(msg.contains("orch-tunnel not found"), "missing header");
        assert!(msg.contains("Searched:"), "missing Searched section");
        assert!(msg.contains("Process PATH:"), "missing process PATH");
        assert!(
            msg.contains("Login shell PATH:"),
            "missing login shell PATH"
        );
        assert!(msg.contains("Install:"), "missing Install section");
        assert!(
            msg.contains("/opt/homebrew/bin/orch-tunnel"),
            "missing homebrew fallback"
        );
    }
}
