use std::process::Stdio;

use tokio::io::{AsyncBufReadExt, BufReader, Lines};
use tokio::process::{Child, ChildStdout, Command};

use crate::types::{Message, PermissionMode, QueryOptions};
use crate::{ClaudeAgentError, Result};

// ─── ClaudeProcess ────────────────────────────────────────────────────────

/// A running `claude --print --output-format stream-json` subprocess.
///
/// Reads one JSONL message per call to `next_message`. Callers drive the
/// read loop; `QueryStream` wraps this in an async `Stream`.
pub(crate) struct ClaudeProcess {
    child: Child,
    lines: Lines<BufReader<ChildStdout>>,
}

impl ClaudeProcess {
    /// Spawn the real `claude` binary with the given prompt and options.
    ///
    /// `CLAUDECODE` is removed from the environment so this works both from a
    /// terminal and from inside a running Claude session (e.g., during `sdlc run`).
    pub(crate) fn spawn(prompt: &str, opts: &QueryOptions) -> Result<Self> {
        let mut cmd = build_command(prompt, opts);
        cmd.env_remove("CLAUDECODE");
        Self::from_command(cmd)
    }

    /// Spawn an arbitrary command as a mock Claude process.
    /// Used in unit tests to inject a command that emits fixed JSON lines.
    #[cfg(test)]
    pub(crate) fn spawn_command(cmd: Command) -> Result<Self> {
        Self::from_command(cmd)
    }

    fn from_command(mut cmd: Command) -> Result<Self> {
        cmd.stdout(Stdio::piped()).stderr(Stdio::null());

        let mut child = cmd.spawn().map_err(ClaudeAgentError::Io)?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ClaudeAgentError::Process("stdout not captured".into()))?;

        let lines = BufReader::new(stdout).lines();
        Ok(Self { child, lines })
    }

    /// Read the next non-empty JSONL line from stdout and deserialize it.
    ///
    /// Returns `Ok(None)` on EOF (process exited normally).
    pub(crate) async fn next_message(&mut self) -> Result<Option<Message>> {
        loop {
            match self.lines.next_line().await {
                Err(e) => return Err(ClaudeAgentError::Io(e)),
                Ok(None) => return Ok(None),
                Ok(Some(line)) => {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    return serde_json::from_str(trimmed).map(Some).map_err(|e| {
                        ClaudeAgentError::Parse {
                            line: trimmed.to_owned(),
                            source: e,
                        }
                    });
                }
            }
        }
    }

    /// Kill the subprocess (best-effort; errors are silently ignored).
    pub(crate) async fn kill(&mut self) {
        let _ = self.child.kill().await;
    }
}

// ─── Command builder ──────────────────────────────────────────────────────

fn build_command(prompt: &str, opts: &QueryOptions) -> Command {
    let mut cmd = Command::new("claude");

    // Non-interactive streaming mode
    cmd.arg("--print").arg("--output-format").arg("stream-json");

    if let Some(model) = &opts.model {
        cmd.arg("--model").arg(model);
    }

    if let Some(max_turns) = opts.max_turns {
        cmd.arg("--max-turns").arg(max_turns.to_string());
    }

    if !opts.allowed_tools.is_empty() {
        cmd.arg("--allowed-tools").args(&opts.allowed_tools);
    }

    if !opts.disallowed_tools.is_empty() {
        cmd.arg("--disallowed-tools").args(&opts.disallowed_tools);
    }

    if opts.permission_mode != PermissionMode::Default {
        cmd.arg("--permission-mode")
            .arg(opts.permission_mode.as_str());
    }

    if let Some(sp) = &opts.system_prompt {
        cmd.arg("--system-prompt").arg(sp);
    }

    if let Some(append) = &opts.append_system_prompt {
        cmd.arg("--append-system-prompt").arg(append);
    }

    if let Some(resume) = &opts.resume {
        cmd.arg("--resume").arg(resume);
    }

    if !opts.mcp_servers.is_empty() {
        if let Ok(json) = build_mcp_config_json(&opts.mcp_servers) {
            cmd.arg("--mcp-config").arg(json);
        }
    }

    if let Some(cwd) = &opts.cwd {
        cmd.current_dir(cwd);
    }

    // Prompt is the final positional argument
    cmd.arg(prompt);

    cmd
}

/// Serialise `McpServerConfig` entries into the JSON string expected by
/// `claude --mcp-config '...'`.
///
/// Format: `{"mcpServers":{"<name>":{"type":"stdio","command":"...","args":[...],"env":{...}}}}`
fn build_mcp_config_json(servers: &[crate::types::McpServerConfig]) -> Result<String> {
    let mut mcp_servers = serde_json::Map::new();

    for srv in servers {
        let mut cfg = serde_json::Map::new();
        cfg.insert("type".into(), serde_json::Value::String("stdio".into()));
        cfg.insert(
            "command".into(),
            serde_json::Value::String(srv.command.clone()),
        );

        if !srv.args.is_empty() {
            cfg.insert(
                "args".into(),
                serde_json::Value::Array(
                    srv.args
                        .iter()
                        .map(|a| serde_json::Value::String(a.clone()))
                        .collect(),
                ),
            );
        }

        if !srv.env.is_empty() {
            let env: serde_json::Map<String, serde_json::Value> = srv
                .env
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone())))
                .collect();
            cfg.insert("env".into(), serde_json::Value::Object(env));
        }

        mcp_servers.insert(srv.name.clone(), serde_json::Value::Object(cfg));
    }

    Ok(serde_json::json!({ "mcpServers": mcp_servers }).to_string())
}
