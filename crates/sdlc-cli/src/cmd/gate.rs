use sdlc_core::gate::{GateDefinition, GateKind, GateResult};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

/// Execute verification gates and return results.
///
/// Stops early on:
/// - A human gate (always returns passed=false with the gate prompt)
/// - A shell gate that exhausts all retries
///
/// The `action` parameter identifies which action triggered these gates
/// (e.g. `"create_spec"`, `"implement_task"`), matching the spec's
/// `run_gates(root, action, gates)` signature.
///
/// Returns all gate results collected up to the stopping point.
///
/// ## Attempt numbering
///
/// `GateResult.attempt` is 1-indexed: the first attempt is `1`, the second
/// is `2`, etc. Combined with `GateDefinition.max_retries` (which is
/// 0-indexed — `0` means one attempt, `2` means up to three attempts),
/// the total attempts for a gate is `max_retries + 1`.
pub fn run_gates(root: &Path, _action: &str, gates: &[GateDefinition]) -> Vec<GateResult> {
    let mut results = Vec::new();

    for gate in gates {
        match &gate.gate_type {
            GateKind::Human { prompt } => {
                results.push(GateResult {
                    gate_name: gate.name.clone(),
                    passed: false,
                    output: prompt.clone(),
                    attempt: 1,
                    duration_ms: 0,
                });
                return results;
            }
            GateKind::StepBack { questions } => {
                let prompt = questions.join("\n");
                results.push(GateResult {
                    gate_name: gate.name.clone(),
                    passed: false,
                    output: prompt,
                    attempt: 1,
                    duration_ms: 0,
                });
                return results;
            }
            GateKind::Shell { command } => {
                if command.trim().is_empty() {
                    results.push(GateResult {
                        gate_name: gate.name.clone(),
                        passed: false,
                        output: "gate command is empty".to_string(),
                        attempt: 1,
                        duration_ms: 0,
                    });
                    return results;
                }
                let max_attempts = gate.max_retries + 1;
                let mut last_passed = false;
                let timeout = if gate.timeout_seconds == 0 {
                    None
                } else {
                    Some(Duration::from_secs(gate.timeout_seconds as u64))
                };

                for attempt in 1..=max_attempts {
                    let start = std::time::Instant::now();
                    let (passed, output) = execute_shell_gate(command, root, timeout);
                    let duration_ms = start.elapsed().as_millis() as u64;
                    last_passed = passed;

                    results.push(GateResult {
                        gate_name: gate.name.clone(),
                        passed,
                        output,
                        attempt,
                        duration_ms,
                    });

                    if passed {
                        break;
                    }
                }

                if !last_passed {
                    return results;
                }
            }
        }
    }

    results
}

/// Execute a shell command with an optional timeout. Returns (success, combined output).
///
/// Uses dedicated threads for stdout/stderr reading (avoiding pipe-buffer deadlocks)
/// and a waiter thread with `mpsc::recv_timeout` for timeout support (no busy-wait).
///
/// `None` timeout means wait indefinitely.
fn execute_shell_gate(command: &str, cwd: &Path, timeout: Option<Duration>) -> (bool, String) {
    let mut child = match Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(cwd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => return (false, format!("failed to spawn: {e}")),
    };

    let child_pid = child.id();

    // Read stdout/stderr in dedicated threads to avoid pipe-buffer deadlocks
    let stdout_handle = child.stdout.take();
    let stderr_handle = child.stderr.take();

    let stdout_thread = std::thread::spawn(move || -> String {
        let mut buf = String::new();
        if let Some(mut r) = stdout_handle {
            use std::io::Read;
            let _ = r.read_to_string(&mut buf);
        }
        buf
    });
    let stderr_thread = std::thread::spawn(move || -> String {
        let mut buf = String::new();
        if let Some(mut r) = stderr_handle {
            use std::io::Read;
            let _ = r.read_to_string(&mut buf);
        }
        buf
    });

    // Wait for the child process, with optional timeout
    let wait_result = match timeout {
        None => {
            // No timeout — block until the process exits
            child.wait()
        }
        Some(timeout_dur) => {
            // Use a waiter thread + mpsc channel for timeout support.
            // The child is moved to the thread; on timeout we kill by PID.
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let _ = tx.send(child.wait());
            });

            match rx.recv_timeout(timeout_dur) {
                Ok(result) => result,
                Err(_) => {
                    // Timeout — kill the process. The waiter thread will unblock
                    // once the killed process exits; reader threads will get EOF
                    // on the closed pipes and terminate naturally.
                    kill_process(child_pid);
                    let secs = timeout_dur.as_secs();
                    return (false, format!("timed out after {secs}s"));
                }
            }
        }
    };

    // Collect output from reader threads
    let stdout_buf = stdout_thread.join().unwrap_or_default();
    let stderr_buf = stderr_thread.join().unwrap_or_default();

    let status = match wait_result {
        Ok(s) => s,
        Err(e) => return (false, format!("wait failed: {e}")),
    };

    format_output(status.success(), &stdout_buf, &stderr_buf)
}

/// Combine stdout/stderr and cap to 10KB (keeping the tail).
fn format_output(success: bool, stdout: &str, stderr: &str) -> (bool, String) {
    let output = if stderr.is_empty() {
        stdout.to_string()
    } else if stdout.is_empty() {
        stderr.to_string()
    } else {
        format!("{stdout}\n{stderr}")
    };
    // Cap output to 10KB to avoid unbounded memory from verbose gates
    const MAX_OUTPUT: usize = 10 * 1024;
    let trimmed = output.trim();
    let capped = if trimmed.len() > MAX_OUTPUT {
        &trimmed[trimmed.len() - MAX_OUTPUT..]
    } else {
        trimmed
    };
    (success, capped.to_string())
}

/// Terminate a process by PID using SIGKILL. Best-effort; errors are silently ignored.
fn kill_process(pid: u32) {
    let _ = Command::new("kill")
        .arg("-9")
        .arg(pid.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shell_gate(name: &str, command: &str) -> GateDefinition {
        GateDefinition {
            name: name.to_string(),
            gate_type: GateKind::Shell {
                command: command.to_string(),
            },
            auto: true,
            max_retries: 0,
            timeout_seconds: 10,
        }
    }

    fn human_gate(name: &str, prompt: &str) -> GateDefinition {
        GateDefinition {
            name: name.to_string(),
            gate_type: GateKind::Human {
                prompt: prompt.to_string(),
            },
            auto: false,
            max_retries: 0,
            timeout_seconds: 0,
        }
    }

    #[test]
    fn shell_gate_true_passes() {
        let gates = vec![shell_gate("check", "true")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].gate_name, "check");
        assert_eq!(results[0].attempt, 1);
    }

    #[test]
    fn shell_gate_false_fails() {
        let gates = vec![shell_gate("check", "false")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
    }

    #[test]
    fn human_gate_always_stops() {
        let gates = vec![
            shell_gate("build", "true"),
            human_gate("review", "Review the code"),
            shell_gate("lint", "true"),
        ];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
        assert_eq!(results[1].gate_name, "review");
        assert_eq!(results[1].output, "Review the code");
    }

    #[test]
    fn no_gates_returns_empty() {
        let results = run_gates(Path::new("/tmp"), "test_action", &[]);
        assert!(results.is_empty());
    }

    #[test]
    fn shell_gate_with_retries_exhausted() {
        let gates = vec![GateDefinition {
            name: "flaky".to_string(),
            gate_type: GateKind::Shell {
                command: "false".to_string(),
            },
            auto: true,
            max_retries: 2,
            timeout_seconds: 10,
        }];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| !r.passed));
        assert_eq!(results[0].attempt, 1);
        assert_eq!(results[1].attempt, 2);
        assert_eq!(results[2].attempt, 3);
    }

    #[test]
    fn shell_gate_timeout() {
        // Test timeout directly via execute_shell_gate with 150ms timeout
        // to keep wall-clock time well under 1s.
        let (passed, output) = execute_shell_gate(
            "sleep 60",
            Path::new("/tmp"),
            Some(Duration::from_millis(150)),
        );
        assert!(!passed);
        assert!(output.contains("timed out"));
    }

    #[test]
    fn multiple_gates_stop_on_first_failure() {
        let gates = vec![
            shell_gate("build", "true"),
            shell_gate("test", "false"),
            shell_gate("lint", "true"),
        ];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
        assert_eq!(results[1].gate_name, "test");
    }

    #[test]
    fn all_gates_pass() {
        let gates = vec![
            shell_gate("build", "true"),
            shell_gate("test", "true"),
            shell_gate("lint", "true"),
        ];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| r.passed));
    }

    #[test]
    fn gate_captures_stdout() {
        let gates = vec![shell_gate("echo", "echo 'hello world'")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].output, "hello world");
    }

    #[test]
    fn gate_captures_stderr() {
        let gates = vec![shell_gate("warn", "echo 'error msg' >&2 && false")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].output, "error msg");
    }

    #[test]
    fn gate_duration_is_recorded() {
        let gates = vec![shell_gate("sleep", "sleep 0.1")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert!(results[0].duration_ms >= 50);
    }

    #[test]
    fn shell_gate_retries_then_passes() {
        // File-based counter: fails on attempt 1, passes on attempt 2+
        let dir = tempfile::TempDir::new().unwrap();
        let counter = dir.path().join("counter");
        std::fs::write(&counter, "0").unwrap();
        let cmd = format!(
            "c=$(cat {p}); c=$((c+1)); echo $c > {p}; [ $c -ge 2 ]",
            p = counter.display()
        );
        let gates = vec![GateDefinition {
            name: "flaky".to_string(),
            gate_type: GateKind::Shell { command: cmd },
            auto: true,
            max_retries: 2,
            timeout_seconds: 10,
        }];
        let results = run_gates(dir.path(), "test_action", &gates);
        assert_eq!(results.len(), 2);
        assert!(!results[0].passed);
        assert_eq!(results[0].attempt, 1);
        assert!(results[1].passed);
        assert_eq!(results[1].attempt, 2);
    }

    #[test]
    fn empty_command_fails_immediately() {
        let gates = vec![shell_gate("bad", "")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0].output.contains("empty"));
    }

    #[test]
    fn whitespace_only_command_fails_immediately() {
        let gates = vec![shell_gate("bad", "   ")];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert!(results[0].output.contains("empty"));
    }

    #[test]
    fn zero_timeout_means_no_timeout() {
        // timeout_seconds=0 should NOT kill the process immediately
        let gates = vec![GateDefinition {
            name: "quick".to_string(),
            gate_type: GateKind::Shell {
                command: "echo ok".to_string(),
            },
            auto: true,
            max_retries: 0,
            timeout_seconds: 0,
        }];
        let results = run_gates(Path::new("/tmp"), "test_action", &gates);
        assert_eq!(results.len(), 1);
        assert!(results[0].passed);
        assert_eq!(results[0].output, "ok");
    }
}
