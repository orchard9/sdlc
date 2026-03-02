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
    #[error(
        "orch-tunnel not found\n\n\
         sdlc ui --tunnel requires orch-tunnel.\n\n\
         Install:\n\
           macOS    brew install orch-tunnel\n\
           Linux/Windows  gh release download --repo orchard9/tunnel \\\n\
                            --pattern 'orch-tunnel-*' -D /usr/local/bin\n\
                          chmod +x /usr/local/bin/orch-tunnel\n\n\
         Then re-run: sdlc ui --tunnel"
    )]
    NotFound,

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

pub fn find_orch_tunnel() -> Result<PathBuf, TunnelError> {
    which::which("orch-tunnel").map_err(|_| TunnelError::NotFound)
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
}
