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

/// Scaffold a new tool skeleton in `.sdlc/tools/<name>/`.
///
/// Creates `tool.ts`, `config.yaml`, and `README.md`.
/// Returns `SdlcError::ToolExists` if the tool directory already exists.
/// Returns `SdlcError::InvalidSlug` if `name` contains invalid characters.
pub fn scaffold_tool(root: &Path, name: &str, description: &str) -> Result<()> {
    crate::paths::validate_slug(name)?;

    let tool_dir = crate::paths::tool_dir(root, name);
    if tool_dir.exists() {
        return Err(crate::error::SdlcError::ToolExists(name.to_string()));
    }

    crate::io::ensure_dir(&tool_dir)?;

    // tool.ts — scaffold skeleton
    let script_content = build_scaffold_ts(name, description);
    let script_path = crate::paths::tool_script(root, name);
    crate::io::atomic_write(&script_path, script_content.as_bytes())?;

    // config.yaml
    let config_content = format!(
        "name: {name}\nversion: \"0.1.0\"\ndescription: {description:?}\n# Add tool-specific config here\n"
    );
    let config_path = crate::paths::tool_config(root, name);
    crate::io::atomic_write(&config_path, config_content.as_bytes())?;

    // README.md
    let readme_content = build_scaffold_readme(name, description);
    let readme_path = crate::paths::tool_readme(root, name);
    crate::io::atomic_write(&readme_path, readme_content.as_bytes())?;

    Ok(())
}

fn build_scaffold_ts(name: &str, description: &str) -> String {
    let display_name = to_display_name(name);
    let underline = "=".repeat(display_name.len());
    format!(
        r#"/**
 * {display_name}
 * {underline}
 * {description}
 *
 * WHAT IT DOES
 * ------------
 * --run:   Reads JSON from stdin: {{ ... }}
 *          Returns JSON to stdout: {{ ... }}
 * --meta:  Writes ToolMeta JSON to stdout. Used by `sdlc tool sync`.
 */

import type {{ ToolMeta, ToolResult }} from '../_shared/types.ts'
import {{ makeLogger }} from '../_shared/log.ts'
import {{ getArgs, readStdin, exit }} from '../_shared/runtime.ts'

const log = makeLogger('{name}')

export const meta: ToolMeta = {{
  name: '{name}',
  display_name: '{display_name}',
  description: '{description}',
  version: '0.1.0',
  requires_setup: false,
  input_schema: {{
    type: 'object',
    required: [],
    properties: {{}}
  }},
  output_schema: {{
    type: 'object',
    properties: {{}}
  }},
}}

// TODO: implement run()
export async function run(
  input: Record<string, unknown>,
  _root: string,
): Promise<ToolResult<Record<string, unknown>>> {{
  log.info('running {name}')
  return {{ ok: true, data: {{ result: 'TODO: implement run()' }} }}
}}

const mode = getArgs()[0] ?? '--run'
const root = process.env.SDLC_ROOT ?? process.cwd()

if (mode === '--meta') {{
  console.log(JSON.stringify(meta))
  exit(0)
}} else if (mode === '--run') {{
  readStdin()
    .then(raw => run(JSON.parse(raw || '{{}}') as Record<string, unknown>, root))
    .then(result => {{ console.log(JSON.stringify(result)); exit(result.ok ? 0 : 1) }})
    .catch(e => {{ console.log(JSON.stringify({{ ok: false, error: String(e) }})); exit(1) }})
}} else {{
  console.error(`Unknown mode: ${{mode}}. Use --meta or --run.`)
  exit(1)
}}
"#
    )
}

fn build_scaffold_readme(name: &str, description: &str) -> String {
    let display_name = to_display_name(name);
    format!("# {display_name}\n\n{description}\n\n## Usage\n\n```bash\nsdlc tool run {name}\n```\n")
}

fn to_display_name(slug: &str) -> String {
    slug.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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

    #[test]
    fn to_display_name_capitalizes_each_word() {
        assert_eq!(to_display_name("my-tool"), "My Tool");
        assert_eq!(to_display_name("quality-check"), "Quality Check");
        assert_eq!(to_display_name("ama"), "Ama");
    }

    #[test]
    fn scaffold_tool_creates_files() {
        let dir = tempfile::TempDir::new().unwrap();
        scaffold_tool(dir.path(), "my-tool", "A test tool").unwrap();
        assert!(crate::paths::tool_script(dir.path(), "my-tool").exists());
        assert!(crate::paths::tool_config(dir.path(), "my-tool").exists());
        assert!(crate::paths::tool_readme(dir.path(), "my-tool").exists());
    }

    #[test]
    fn scaffold_tool_returns_tool_exists_for_duplicate() {
        let dir = tempfile::TempDir::new().unwrap();
        scaffold_tool(dir.path(), "my-tool", "First").unwrap();
        let err = scaffold_tool(dir.path(), "my-tool", "Second").unwrap_err();
        assert!(matches!(err, crate::error::SdlcError::ToolExists(_)));
    }

    #[test]
    fn scaffold_tool_returns_invalid_slug_for_bad_name() {
        let dir = tempfile::TempDir::new().unwrap();
        let err = scaffold_tool(dir.path(), "BAD NAME", "desc").unwrap_err();
        assert!(matches!(err, crate::error::SdlcError::InvalidSlug(_)));
    }
}
