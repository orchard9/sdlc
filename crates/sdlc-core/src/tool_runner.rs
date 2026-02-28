//! Runtime detection and subprocess invocation for SDLC tool scripts.
//!
//! SDLC tools are TypeScript scripts that speak a JSON stdin/stdout protocol.
//! This module detects the best available runtime (bun > deno > node/npx) and
//! provides a single `run_tool()` function that all callers use.
//!
//! # Protocol
//! - `--meta`:  No stdin. Writes ToolMeta JSON to stdout.
//! - `--run`:   Reads JSON from stdin. Writes ToolResult JSON to stdout.
//! - `--setup`: No stdin. Writes ToolResult JSON to stdout.
//!
//! # Runtime priority
//! 1. bun  — fastest startup, best Bun-specific APIs
//! 2. deno — built-in TypeScript, good permissions model
//! 3. node — fallback via `npx --yes tsx` for TypeScript support

use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::error::{Result, SdlcError};

/// The available JavaScript runtimes, in priority order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Bun,
    Deno,
    Node,
}

impl Runtime {
    pub fn name(&self) -> &'static str {
        match self {
            Runtime::Bun => "bun",
            Runtime::Deno => "deno",
            Runtime::Node => "node (via npx tsx)",
        }
    }
}

/// Detect the best available JavaScript runtime.
/// Returns None if no supported runtime is found.
pub fn detect_runtime() -> Option<Runtime> {
    if which::which("bun").is_ok() {
        return Some(Runtime::Bun);
    }
    if which::which("deno").is_ok() {
        return Some(Runtime::Deno);
    }
    if which::which("npx").is_ok() {
        return Some(Runtime::Node);
    }
    None
}

/// Run a tool script in the given mode, optionally feeding JSON to stdin.
///
/// # Arguments
/// - `script`: Path to the tool.ts file
/// - `mode`: One of `--meta`, `--run`, `--setup`
/// - `stdin_json`: JSON string to feed to the tool's stdin (for `--run` mode)
/// - `root`: Project root (set as `SDLC_ROOT` env var for the subprocess)
///
/// # Returns
/// The tool's stdout as a String (expected to be JSON).
/// Stderr is passed through to the parent process for real-time logging.
pub fn run_tool(
    script: &Path,
    mode: &str,
    stdin_json: Option<&str>,
    root: &Path,
) -> Result<String> {
    let runtime = detect_runtime().ok_or(SdlcError::NoToolRuntime)?;

    let script_str = script.to_str().ok_or_else(|| {
        SdlcError::ToolSpawnFailed("script path contains non-UTF8 characters".into())
    })?;

    let mut cmd = build_command(runtime, script_str, mode);

    // Set SDLC_ROOT so tools know the project root
    cmd.env("SDLC_ROOT", root);
    cmd.current_dir(root);

    // stdin: piped if we have JSON to send, null otherwise
    if stdin_json.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }

    cmd.stdout(Stdio::piped());
    // stderr flows through so tool log lines appear in the terminal/run panel
    cmd.stderr(Stdio::inherit());

    let mut child = cmd
        .spawn()
        .map_err(|e| SdlcError::ToolSpawnFailed(e.to_string()))?;

    // Feed stdin if provided
    if let Some(json) = stdin_json {
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(json.as_bytes())
                .map_err(|e| SdlcError::ToolSpawnFailed(format!("failed to write stdin: {e}")))?;
        }
    }

    let output = child
        .wait_with_output()
        .map_err(|e| SdlcError::ToolSpawnFailed(e.to_string()))?;

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();

    // For --run mode, a non-zero exit code means ok:false (checks failed), not a crash.
    // The JSON result is always in stdout — return it regardless of exit code.
    // For --meta and --setup, a non-zero exit is a genuine error.
    if !output.status.success() && mode != "--run" {
        let hint = stdout.chars().take(500).collect::<String>();
        return Err(SdlcError::ToolFailed(hint));
    }

    Ok(stdout)
}

fn build_command(runtime: Runtime, script: &str, mode: &str) -> Command {
    match runtime {
        Runtime::Bun => {
            let mut cmd = Command::new("bun");
            cmd.args(["run", script, mode]);
            cmd
        }
        Runtime::Deno => {
            let mut cmd = Command::new("deno");
            cmd.args([
                "run",
                "--allow-read",
                "--allow-run",
                "--allow-write",
                "--allow-env",
                "--allow-net",
                script,
                mode,
            ]);
            cmd
        }
        Runtime::Node => {
            let mut cmd = Command::new("npx");
            cmd.args(["--yes", "tsx", script, mode]);
            cmd
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_runtime_returns_some_or_none() {
        // Just verify it doesn't panic — actual runtime depends on test environment
        let _ = detect_runtime();
    }

    #[test]
    fn runtime_names_are_stable() {
        assert_eq!(Runtime::Bun.name(), "bun");
        assert_eq!(Runtime::Deno.name(), "deno");
        assert_eq!(Runtime::Node.name(), "node (via npx tsx)");
    }
}
