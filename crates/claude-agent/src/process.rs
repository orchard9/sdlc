use std::process::Stdio;
use std::sync::{Arc, Mutex};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

use crate::types::{Message, PermissionMode, QueryOptions};
use crate::{ClaudeAgentError, Result};

// ─── ClaudeProcess ────────────────────────────────────────────────────────

/// A running `claude --output-format stream-json --input-format stream-json`
/// subprocess using bidirectional streaming.
///
/// The prompt is sent as a JSON message on stdin (matching the TypeScript SDK
/// protocol), and responses are read as JSONL from stdout. Stderr is captured
/// in a background task and surfaced on process exit errors.
pub(crate) struct ClaudeProcess {
    child: Child,
    lines: Lines<BufReader<ChildStdout>>,
    stdin: Option<ChildStdin>,
    /// Stderr output collected by a background reader task.
    stderr_buf: Arc<Mutex<String>>,
}

impl ClaudeProcess {
    /// Spawn the real `claude` binary with the given prompt and options.
    ///
    /// The prompt is sent as a user message on stdin (bidirectional stream-json
    /// protocol). After sending, stdin is closed for single-turn operation.
    ///
    /// `CLAUDECODE` is removed from the environment so this works both from a
    /// terminal and from inside a running Claude session (e.g., during `sdlc run`).
    pub(crate) async fn spawn(prompt: &str, opts: &QueryOptions) -> Result<Self> {
        let mut cmd = build_command(opts);
        cmd.env_remove("CLAUDECODE");

        // Apply additional env vars from options
        for (k, v) in &opts.env {
            cmd.env(k, v);
        }

        let mut process = Self::from_command(cmd)?;

        // Send the initial prompt as a user message via stdin
        let user_msg = serde_json::json!({
            "type": "user",
            "message": {
                "role": "user",
                "content": [{"type": "text", "text": prompt}]
            }
        });
        process.send_message(&user_msg).await?;
        process.close_stdin();

        Ok(process)
    }

    /// Spawn an arbitrary command as a mock Claude process.
    /// Used in unit tests to inject a command that emits fixed JSON lines.
    #[cfg(test)]
    pub(crate) fn spawn_command(cmd: Command) -> Result<Self> {
        Self::from_command(cmd)
    }

    fn from_command(mut cmd: Command) -> Result<Self> {
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(ClaudeAgentError::Io)?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| ClaudeAgentError::Process("stdout not captured".into()))?;

        let stdin = child.stdin.take();

        // Spawn a background task to drain stderr into a buffer.
        // This matches the TS SDK pattern: stderr is captured and surfaced
        // when the process exits with an error.
        let stderr_buf = Arc::new(Mutex::new(String::new()));
        if let Some(stderr) = child.stderr.take() {
            let buf = Arc::clone(&stderr_buf);
            tokio::spawn(async move {
                let mut reader = BufReader::new(stderr).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    if let Ok(mut b) = buf.lock() {
                        if !b.is_empty() {
                            b.push('\n');
                        }
                        b.push_str(&line);
                    }
                }
            });
        }

        let lines = BufReader::new(stdout).lines();
        Ok(Self {
            child,
            lines,
            stdin,
            stderr_buf,
        })
    }

    /// Write a JSON message to the subprocess stdin.
    pub(crate) async fn send_message(&mut self, msg: &serde_json::Value) -> Result<()> {
        let stdin = self
            .stdin
            .as_mut()
            .ok_or_else(|| ClaudeAgentError::Process("stdin already closed".into()))?;

        let mut buf = serde_json::to_vec(msg).map_err(|e| {
            ClaudeAgentError::Process(format!("failed to serialize stdin message: {e}"))
        })?;
        buf.push(b'\n');

        stdin.write_all(&buf).await.map_err(ClaudeAgentError::Io)?;
        stdin.flush().await.map_err(ClaudeAgentError::Io)?;

        Ok(())
    }

    /// Close stdin, signalling no more input (single-turn mode).
    pub(crate) fn close_stdin(&mut self) {
        self.stdin.take();
    }

    /// Read the next non-empty JSONL line from stdout and deserialize it.
    ///
    /// Unknown message types (e.g. `rate_limit_event`) are silently skipped,
    /// matching the TS SDK's behaviour of ignoring types it doesn't recognise.
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
                    match serde_json::from_str::<Message>(trimmed) {
                        Ok(msg) => return Ok(Some(msg)),
                        Err(e) => {
                            // If the line is valid JSON with an unknown "type",
                            // skip it rather than failing the stream.
                            if is_unknown_message_type(trimmed) {
                                continue;
                            }
                            return Err(ClaudeAgentError::Parse {
                                line: trimmed.to_owned(),
                                source: e,
                            });
                        }
                    }
                }
            }
        }
    }

    /// Wait for the child to exit and return an error if the exit code is
    /// non-zero or the process was killed by a signal.
    ///
    /// Matches the TS SDK's `getProcessExitError()` — checks exit code and
    /// includes captured stderr in the error message.
    pub(crate) async fn wait_exit_error(&mut self) -> Option<ClaudeAgentError> {
        let status = match self.child.wait().await {
            Ok(s) => s,
            Err(e) => return Some(ClaudeAgentError::Io(e)),
        };

        if status.success() {
            return None;
        }

        let stderr = self
            .stderr_buf
            .lock()
            .ok()
            .map(|b| b.clone())
            .unwrap_or_default();

        let msg = if let Some(code) = status.code() {
            if stderr.is_empty() {
                format!("Claude Code process exited with code {code}")
            } else {
                format!("Claude Code process exited with code {code}\nstderr: {stderr}")
            }
        } else {
            // Killed by signal (Unix)
            if stderr.is_empty() {
                "Claude Code process terminated by signal".to_string()
            } else {
                format!("Claude Code process terminated by signal\nstderr: {stderr}")
            }
        };

        Some(ClaudeAgentError::Process(msg))
    }

    /// Kill the subprocess (best-effort; errors are silently ignored).
    pub(crate) async fn kill(&mut self) {
        let _ = self.child.kill().await;
    }
}

/// Check if a JSON line has a `"type"` field with a value we don't recognise.
/// If it's valid JSON with a type field, it's an unknown message type and
/// should be skipped. If it's not valid JSON, it's a genuine parse error.
fn is_unknown_message_type(line: &str) -> bool {
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
        // It's valid JSON — if it has a "type" field, it's just an unknown
        // message type (e.g. rate_limit_event, hook_progress, etc.)
        v.get("type").is_some()
    } else {
        false
    }
}

// ─── Command builder ──────────────────────────────────────────────────────

fn build_command(opts: &QueryOptions) -> Command {
    let exe = opts.path_to_executable.as_deref().unwrap_or("claude");
    let mut cmd = Command::new(exe);

    // Bidirectional streaming protocol (matches TS SDK)
    cmd.arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--input-format")
        .arg("stream-json");

    if let Some(model) = &opts.model {
        cmd.arg("--model").arg(model);
    }

    if let Some(max_turns) = opts.max_turns {
        cmd.arg("--max-turns").arg(max_turns.to_string());
    }

    if let Some(budget) = opts.max_budget_usd {
        cmd.arg("--max-budget-usd").arg(budget.to_string());
    }

    if let Some(effort) = &opts.effort {
        cmd.arg("--effort").arg(effort.as_str());
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

    if opts.continue_conversation {
        cmd.arg("--continue");
    }

    if let Some(sid) = &opts.session_id {
        cmd.arg("--session-id").arg(sid);
    }

    if !opts.mcp_servers.is_empty() {
        if let Ok(json) = build_mcp_config_json(&opts.mcp_servers) {
            cmd.arg("--mcp-config").arg(json);
        }
    }

    for dir in &opts.additional_directories {
        cmd.arg("--add-dir").arg(dir);
    }

    if opts.debug {
        cmd.arg("--debug");
    }

    if opts.include_partial_messages {
        cmd.arg("--include-partial-messages");
    }

    if opts.no_session_persistence {
        cmd.arg("--no-session-persistence");
    }

    if let Some(cwd) = &opts.cwd {
        cmd.current_dir(cwd);
    }

    // NOTE: prompt is NOT a positional arg — it's sent via stdin

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
