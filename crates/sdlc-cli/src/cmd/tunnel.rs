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
        "cloudflared not found\n\n\
         sdlc ui --tunnel requires cloudflared to open a public HTTPS tunnel.\n\
         It is not required for any other sdlc functionality.\n\n\
         Install it:\n\
         \n\
           macOS    brew install cloudflare/cloudflare/cloudflared\n\
           Windows  winget install Cloudflare.cloudflared\n\
           Linux    https://pkg.cloudflare.com/index.html\n\
         \n\
         Then re-run: sdlc ui --tunnel"
    )]
    NotFound,

    #[error(
        "cloudflared tunnel did not start within {0} seconds.\n\
         Check your network connection and try again."
    )]
    Timeout(u64),

    #[error(
        "cloudflared exited unexpectedly.\n\
         Try running manually: cloudflared tunnel --url http://localhost:{port}"
    )]
    ExitedEarly { port: u16 },

    #[error("process error: {0}")]
    Process(#[from] std::io::Error),
}

// ---------------------------------------------------------------------------
// Tunnel
// ---------------------------------------------------------------------------

/// A running cloudflared quick-tunnel.
///
/// The tunnel process is killed when [`stop`] is called or when this value is
/// dropped (via `kill_on_drop(true)`).
pub struct Tunnel {
    /// Public HTTPS URL (e.g. `https://fancy-rabbit.trycloudflare.com`).
    pub url: String,
    process: Child,
}

impl Tunnel {
    /// Spawn cloudflared and wait until a `trycloudflare.com` URL appears on
    /// stderr. Times out after `SDLC_TUNNEL_TIMEOUT_SECS` seconds (default 15).
    pub async fn start(port: u16) -> Result<Self, TunnelError> {
        let binary = find_cloudflared()?;

        let timeout_secs: u64 = std::env::var("SDLC_TUNNEL_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(15);

        let mut child = Command::new(binary)
            .args([
                "tunnel",
                "--url",
                &format!("http://localhost:{port}"),
                "--no-autoupdate",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;

        let stderr = child.stderr.take().expect("stderr was configured as piped");

        let url_result = timeout(
            Duration::from_secs(timeout_secs),
            read_tunnel_url(stderr, port),
        )
        .await;

        match url_result {
            Ok(Ok((url, remaining_reader))) => {
                // Drain cloudflared's stderr in the background.
                // Without this, cloudflared gets SIGPIPE when it tries to log
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

    /// Kill the cloudflared process and wait for it to exit.
    pub async fn stop(mut self) {
        let _ = self.process.kill().await;
        let _ = self.process.wait().await;
    }
}

async fn read_tunnel_url(
    stderr: tokio::process::ChildStderr,
    port: u16,
) -> Result<(String, BufReader<tokio::process::ChildStderr>), TunnelError> {
    let reader = BufReader::new(stderr);
    let mut lines = reader.lines();
    while let Ok(Some(line)) = lines.next_line().await {
        if let Some(url) = extract_tunnel_url(&line) {
            // Return the reader so the caller can keep the pipe open.
            // If we drop it here, cloudflared gets SIGPIPE and dies.
            return Ok((url.to_string(), lines.into_inner()));
        }
    }
    Err(TunnelError::ExitedEarly { port })
}

// ---------------------------------------------------------------------------
// QR code + terminal output
// ---------------------------------------------------------------------------

/// Print the tunnel URL, QR code, and passcode to stdout.
pub fn print_tunnel_info(project_name: &str, local_port: u16, tunnel_base_url: &str, token: &str) {
    let auth_url = format!("{tunnel_base_url}/?auth={token}");

    println!();
    println!("SDLC UI for '{project_name}'");
    println!("  Local:   http://localhost:{local_port}  (no auth)");
    println!("  Tunnel:  {tunnel_base_url}");
    println!();

    match render_qr(&auth_url) {
        Ok(qr) => print_qr_boxed(&qr),
        Err(_) => {
            // QR rendering failed — fall back to plain URL.
            println!("  {auth_url}");
        }
    }

    println!();
    println!("  Passcode:  {token}");
    println!("  (embedded in QR — scan to access)");
    println!();
    println!("Ctrl+C to stop");
    println!();
}

fn print_qr_boxed(qr: &str) {
    let lines: Vec<&str> = qr.lines().collect();
    let content_width = lines.iter().map(|l| l.chars().count()).max().unwrap_or(0);
    // 2 spaces padding on each side
    let inner = content_width + 4;
    let border = "─".repeat(inner);

    println!("  ┌{border}┐");
    println!("  │{}│", " ".repeat(inner));
    for line in &lines {
        let pad = inner.saturating_sub(line.chars().count() + 2);
        println!("  │  {line}{}│", " ".repeat(pad));
    }
    println!("  │{}│", " ".repeat(inner));
    println!("  └{border}┘");
}

fn render_qr(url: &str) -> Result<String, qrcode::types::QrError> {
    use qrcode::{render::unicode, QrCode};
    let code = QrCode::new(url.as_bytes())?;
    Ok(code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Dark)
        .light_color(unicode::Dense1x2::Light)
        .build())
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

fn find_cloudflared() -> Result<PathBuf, TunnelError> {
    which::which("cloudflared").map_err(|_| TunnelError::NotFound)
}

/// Extract a `https://*.trycloudflare.com` URL from a cloudflared log line.
fn extract_tunnel_url(line: &str) -> Option<&str> {
    let start = line.find("https://")?;
    let rest = &line[start..];
    let end = rest
        .find(|c: char| c.is_whitespace() || c == '"' || c == '\'')
        .unwrap_or(rest.len());
    let url = &rest[..end];
    if url.contains(".trycloudflare.com") {
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
        let line = "https://fancy-rabbit-deluxe.trycloudflare.com";
        assert_eq!(
            extract_tunnel_url(line),
            Some("https://fancy-rabbit-deluxe.trycloudflare.com")
        );
    }

    #[test]
    fn extract_url_embedded_in_log_line() {
        let line =
            "Your quick Tunnel has been created! Visit https://fancy-rabbit.trycloudflare.com now";
        assert_eq!(
            extract_tunnel_url(line),
            Some("https://fancy-rabbit.trycloudflare.com")
        );
    }

    #[test]
    fn extract_url_ignores_non_cloudflare() {
        assert_eq!(extract_tunnel_url("https://example.com"), None);
        assert_eq!(extract_tunnel_url("Starting tunnel..."), None);
    }

    #[test]
    fn token_is_8_alphanumeric_chars() {
        let tok = generate_token();
        assert_eq!(tok.len(), 8);
        assert!(tok.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn tokens_differ_across_calls() {
        // Astronomically unlikely to collide; tests basic RNG functionality.
        let tokens: std::collections::HashSet<_> = (0..20).map(|_| generate_token()).collect();
        assert!(
            tokens.len() > 15,
            "expected distinct tokens, got many duplicates"
        );
    }
}
