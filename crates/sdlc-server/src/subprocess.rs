use crate::state::{RunEvent, RunHandle};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::broadcast;

/// Spawn a subprocess and stream its output through a broadcast channel.
/// Returns the run handle (with sender) that SSE subscribers can tap into.
pub fn spawn_process(argv: Vec<String>, cwd: &Path) -> RunHandle {
    let (tx, initial_rx) = broadcast::channel(1024);
    let completed = Arc::new(AtomicBool::new(false));
    let completed_flag = completed.clone();
    let handle = RunHandle {
        tx: tx.clone(),
        initial_rx: std::sync::Mutex::new(Some(initial_rx)),
        completed,
    };
    let cwd = cwd.to_path_buf();

    tokio::spawn(async move {
        let start = Instant::now();

        let result = Command::new(&argv[0])
            .args(&argv[1..])
            .current_dir(&cwd)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn();

        let mut child = match result {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(RunEvent::Error {
                    message: format!("failed to spawn '{}': {e}", argv[0]),
                });
                completed_flag.store(true, Ordering::Relaxed);
                return;
            }
        };

        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                let _ = tx.send(RunEvent::Error {
                    message: "failed to capture stdout".into(),
                });
                completed_flag.store(true, Ordering::Relaxed);
                return;
            }
        };
        let stderr = match child.stderr.take() {
            Some(s) => s,
            None => {
                let _ = tx.send(RunEvent::Error {
                    message: "failed to capture stderr".into(),
                });
                completed_flag.store(true, Ordering::Relaxed);
                return;
            }
        };

        let tx_out = tx.clone();
        let stdout_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_out.send(RunEvent::Stdout { line });
            }
        });

        let tx_err = tx.clone();
        let stderr_task = tokio::spawn(async move {
            let mut lines = BufReader::new(stderr).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = tx_err.send(RunEvent::Stderr { line });
            }
        });

        let _ = tokio::join!(stdout_task, stderr_task);

        let exit_code = match child.wait().await {
            Ok(status) => status.code().unwrap_or(-1),
            Err(e) => {
                let _ = tx.send(RunEvent::Error {
                    message: format!("wait failed: {e}"),
                });
                -1
            }
        };

        let duration = start.elapsed().as_secs_f64();
        let _ = tx.send(RunEvent::Finished {
            exit_code,
            duration_seconds: duration,
        });
        completed_flag.store(true, Ordering::Relaxed);
    });

    handle
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn spawn_process_creates_valid_handle() {
        let handle = spawn_process(
            vec!["echo".into(), "hello".into()],
            std::path::Path::new("/tmp"),
        );

        // initial_rx should be populated before any subscriber takes it
        assert!(handle.initial_rx.lock().unwrap().is_some());
        // completed flag should start as false
        assert!(!handle.completed.load(Ordering::Relaxed));

        // Wait for the short-lived process to finish
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        assert!(handle.completed.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn initial_rx_receives_stdout_and_finished() {
        let handle = spawn_process(
            vec!["echo".into(), "test-line".into()],
            std::path::Path::new("/tmp"),
        );

        let mut rx = handle.initial_rx.lock().unwrap().take().unwrap();

        // Collect events until Finished
        let mut events = vec![];
        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), rx.recv()).await {
                Ok(Ok(event)) => {
                    let is_finished = matches!(event, RunEvent::Finished { .. });
                    events.push(event);
                    if is_finished {
                        break;
                    }
                }
                Ok(Err(_)) => break, // channel closed
                Err(_) => panic!("timed out waiting for RunEvent"),
            }
        }

        // Should have at least a Stdout event with "test-line"
        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::Stdout { line } if line == "test-line")),
            "expected Stdout event with 'test-line', got: {events:?}"
        );

        // Should have a Finished event with exit_code 0
        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::Finished { exit_code: 0, .. })),
            "expected Finished event with exit_code 0, got: {events:?}"
        );
    }

    #[tokio::test]
    async fn spawn_invalid_command_sends_error_event() {
        let handle = spawn_process(
            vec!["__nonexistent_command_xyz__".into()],
            std::path::Path::new("/tmp"),
        );

        let mut rx = handle.initial_rx.lock().unwrap().take().unwrap();

        let mut got_error = false;
        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), rx.recv()).await {
                Ok(Ok(RunEvent::Error { message })) => {
                    assert!(
                        message.contains("__nonexistent_command_xyz__"),
                        "error message should contain the command name, got: {message}"
                    );
                    got_error = true;
                    break;
                }
                Ok(Ok(_)) => continue,
                Ok(Err(_)) => break,
                Err(_) => panic!("timed out waiting for error event"),
            }
        }

        assert!(got_error, "expected an Error event for nonexistent command");

        // Wait briefly, then verify completed flag is set
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(handle.completed.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn initial_rx_take_leaves_none() {
        let handle = spawn_process(
            vec!["echo".into(), "hi".into()],
            std::path::Path::new("/tmp"),
        );

        // First take succeeds
        let rx = handle.initial_rx.lock().unwrap().take();
        assert!(rx.is_some());

        // Second take returns None (first subscriber already claimed it)
        let rx2 = handle.initial_rx.lock().unwrap().take();
        assert!(rx2.is_none());
    }

    #[tokio::test]
    async fn stderr_is_captured() {
        // Use sh -c to write to stderr
        let handle = spawn_process(
            vec!["sh".into(), "-c".into(), "echo err-output >&2".into()],
            std::path::Path::new("/tmp"),
        );

        let mut rx = handle.initial_rx.lock().unwrap().take().unwrap();

        let mut events = vec![];
        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(5), rx.recv()).await {
                Ok(Ok(event)) => {
                    let is_finished = matches!(event, RunEvent::Finished { .. });
                    events.push(event);
                    if is_finished {
                        break;
                    }
                }
                Ok(Err(_)) => break,
                Err(_) => panic!("timed out waiting for stderr events"),
            }
        }

        assert!(
            events
                .iter()
                .any(|e| matches!(e, RunEvent::Stderr { line } if line == "err-output")),
            "expected Stderr event with 'err-output', got: {events:?}"
        );
    }

    /// Helper: collect all events from a broadcast receiver until Finished or
    /// channel closure, with a per-event timeout.
    async fn collect_events(
        mut rx: broadcast::Receiver<RunEvent>,
        timeout_secs: u64,
    ) -> Vec<RunEvent> {
        let mut events = vec![];
        loop {
            match tokio::time::timeout(tokio::time::Duration::from_secs(timeout_secs), rx.recv())
                .await
            {
                Ok(Ok(event)) => {
                    let is_finished = matches!(event, RunEvent::Finished { .. });
                    events.push(event);
                    if is_finished {
                        break;
                    }
                }
                Ok(Err(_)) => break, // channel closed
                Err(_) => panic!("timed out after {timeout_secs}s waiting for RunEvent"),
            }
        }
        events
    }

    #[tokio::test]
    async fn concurrent_10_processes_no_event_loss() {
        let cwd = std::path::Path::new("/tmp");

        // Spawn 10 processes and take their initial receivers immediately.
        let mut receivers = Vec::with_capacity(10);
        for n in 0..10 {
            let handle = spawn_process(vec!["echo".into(), format!("test-{n}")], cwd);
            let rx = handle.initial_rx.lock().unwrap().take().unwrap();
            receivers.push((n, rx));
        }

        // Drive all 10 subscribers concurrently with a 10-second outer timeout.
        let mut tasks = tokio::task::JoinSet::new();
        for (n, rx) in receivers {
            tasks.spawn(async move {
                let events = collect_events(rx, 10).await;
                (n, events)
            });
        }

        let mut results: Vec<(usize, Vec<RunEvent>)> = Vec::with_capacity(10);
        while let Some(join_result) = tasks.join_next().await {
            results.push(join_result.expect("task panicked"));
        }

        assert_eq!(results.len(), 10, "expected results from all 10 processes");

        for (n, events) in &results {
            let expected_line = format!("test-{n}");

            // Must have at least one Stdout event with the expected line.
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, RunEvent::Stdout { line } if line == &expected_line)),
                "process {n}: missing Stdout event with '{expected_line}', got: {events:?}"
            );

            // Must have a Finished event with exit_code 0.
            assert!(
                events
                    .iter()
                    .any(|e| matches!(e, RunEvent::Finished { exit_code: 0, .. })),
                "process {n}: missing Finished(exit_code=0), got: {events:?}"
            );
        }
    }

    #[tokio::test]
    async fn concurrent_processes_with_mixed_exit_codes() {
        let cwd = std::path::Path::new("/tmp");

        // 3 succeeding processes (echo ok) and 2 failing processes (exit 1).
        struct Spec {
            argv: Vec<String>,
            expected_exit: i32,
            label: &'static str,
        }

        let specs = vec![
            Spec {
                argv: vec!["echo".into(), "ok".into()],
                expected_exit: 0,
                label: "echo-0",
            },
            Spec {
                argv: vec!["echo".into(), "ok".into()],
                expected_exit: 0,
                label: "echo-1",
            },
            Spec {
                argv: vec!["echo".into(), "ok".into()],
                expected_exit: 0,
                label: "echo-2",
            },
            Spec {
                argv: vec!["sh".into(), "-c".into(), "exit 1".into()],
                expected_exit: 1,
                label: "fail-0",
            },
            Spec {
                argv: vec!["sh".into(), "-c".into(), "exit 1".into()],
                expected_exit: 1,
                label: "fail-1",
            },
        ];

        let mut tasks = tokio::task::JoinSet::new();
        for spec in specs {
            let handle = spawn_process(spec.argv, cwd);
            let rx = handle.initial_rx.lock().unwrap().take().unwrap();
            let expected_exit = spec.expected_exit;
            let label = spec.label;

            tasks.spawn(async move {
                let events = collect_events(rx, 10).await;
                (label, expected_exit, events)
            });
        }

        let mut results = Vec::with_capacity(5);
        while let Some(join_result) = tasks.join_next().await {
            results.push(join_result.expect("task panicked"));
        }

        assert_eq!(results.len(), 5, "expected results from all 5 processes");

        for (label, expected_exit, events) in &results {
            // Every process must produce a Finished event.
            let finished = events.iter().find_map(|e| match e {
                RunEvent::Finished { exit_code, .. } => Some(*exit_code),
                _ => None,
            });

            assert!(
                finished.is_some(),
                "{label}: no Finished event found, got: {events:?}"
            );
            assert_eq!(
                finished.unwrap(),
                *expected_exit,
                "{label}: expected exit_code={expected_exit}, got: {}",
                finished.unwrap()
            );
        }
    }

    #[tokio::test]
    async fn broadcast_channel_handles_slow_consumer() {
        let cwd = std::path::Path::new("/tmp");

        // Spawn a process that outputs 500 lines.
        let handle = spawn_process(vec!["seq".into(), "1".into(), "500".into()], cwd);

        let mut rx = handle.initial_rx.lock().unwrap().take().unwrap();

        let mut received_events: Vec<RunEvent> = Vec::new();
        let mut lagged_count: u64 = 0;

        let outcome = tokio::time::timeout(tokio::time::Duration::from_secs(10), async {
            loop {
                // Introduce a small delay to simulate a slow consumer.
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                match rx.recv().await {
                    Ok(event) => {
                        let is_finished = matches!(event, RunEvent::Finished { .. });
                        received_events.push(event);
                        if is_finished {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        lagged_count += n;
                        // Continue reading — lagged is recoverable.
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        })
        .await;

        assert!(
            outcome.is_ok(),
            "timed out waiting for slow-consumer test to complete"
        );

        // The total number of events we actually received plus the number we
        // were told we missed due to lagging must account for the full stream.
        // At minimum we must see the Finished event (it is always the last
        // thing sent and the buffer should have room for it).
        let stdout_count = received_events
            .iter()
            .filter(|e| matches!(e, RunEvent::Stdout { .. }))
            .count() as u64;

        let got_finished = received_events
            .iter()
            .any(|e| matches!(e, RunEvent::Finished { exit_code: 0, .. }));

        assert!(
            got_finished,
            "slow consumer must still receive the Finished event, \
             received {} stdout events, lagged {lagged_count}, events: {:?}",
            stdout_count,
            received_events.last()
        );

        // Either we got all 500 lines, or we lagged but the total is accounted for.
        // The channel capacity is 1024, so with only 500 lines and a 1ms delay
        // we may or may not lag. Both outcomes are acceptable.
        if lagged_count == 0 {
            assert_eq!(
                stdout_count, 500,
                "with no lag, all 500 stdout lines should be received"
            );
        } else {
            // We lagged: received + lagged should cover the full output.
            assert!(
                stdout_count + lagged_count >= 500,
                "received {stdout_count} stdout + lagged {lagged_count} \
                 should account for all 500 lines"
            );
        }
    }
}
