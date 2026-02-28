use std::pin::Pin;
use std::task::{Context, Poll};

use futures::Stream;
use tokio::sync::mpsc;

use crate::process::ClaudeProcess;
use crate::types::{Message, QueryOptions};
use crate::Result;

// ─── QueryStream ──────────────────────────────────────────────────────────

/// An async stream of [`Message`]s from a Claude subprocess.
///
/// Backed by a Tokio mpsc channel. A background task owns [`ClaudeProcess`]
/// and forwards messages until it receives a terminal `Result` message or
/// the process exits. Dropping `QueryStream` closes the receiver, which
/// causes the background task to exit on the next send attempt.
///
/// ```rust,ignore
/// use claude_agent::{query, Message, QueryOptions};
/// use futures::StreamExt;
///
/// let mut stream = query("say hello", QueryOptions::default());
/// while let Some(msg) = stream.next().await {
///     if let Ok(Message::Result(r)) = msg {
///         println!("result: {:?}", r.result_text());
///     }
/// }
/// ```
pub struct QueryStream {
    rx: mpsc::Receiver<Result<Message>>,
}

impl QueryStream {
    pub(crate) fn new(prompt: String, opts: QueryOptions) -> Self {
        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut process = match ClaudeProcess::spawn(&prompt, &opts).await {
                Ok(p) => p,
                Err(e) => {
                    let _ = tx.send(Err(e)).await;
                    return;
                }
            };

            let mut got_result = false;
            loop {
                match process.next_message().await {
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                    Ok(None) => break, // EOF — process exited
                    Ok(Some(msg)) => {
                        let is_terminal = matches!(msg, Message::Result(_));
                        if is_terminal {
                            got_result = true;
                        }
                        if tx.send(Ok(msg)).await.is_err() {
                            break; // Receiver dropped
                        }
                        if is_terminal {
                            break;
                        }
                    }
                }
            }

            // If the process exited without sending a Result message, check
            // for a non-zero exit code and surface stderr (matches TS SDK's
            // `getProcessExitError` pattern).
            if !got_result {
                if let Some(exit_err) = process.wait_exit_error().await {
                    let _ = tx.send(Err(exit_err)).await;
                }
            }

            process.kill().await;
        });

        QueryStream { rx }
    }

    /// Test-only constructor: wrap a raw mpsc receiver as a `QueryStream`.
    /// Used by `runner` tests to inject pre-built message sequences.
    #[cfg(test)]
    pub(crate) fn from_channel(rx: mpsc::Receiver<Result<Message>>) -> Self {
        Self { rx }
    }
}

impl Stream for QueryStream {
    type Item = Result<Message>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.rx.poll_recv(cx)
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResultMessage;
    use futures::StreamExt;
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tokio::process::Command;

    /// Write JSON lines to a temp file, then `cat` it as the mock process.
    fn mock_stream(lines: &[&str]) -> QueryStream {
        let mut f = NamedTempFile::new().unwrap();
        for line in lines {
            writeln!(f, "{}", line).unwrap();
        }
        let path = f.path().to_owned();
        // Keep the file alive for the duration of the test
        std::mem::forget(f);

        let (tx, rx) = mpsc::channel(32);

        tokio::spawn(async move {
            let mut cmd = Command::new("cat");
            cmd.arg(&path);
            let mut process = ClaudeProcess::spawn_command(cmd).unwrap();

            loop {
                match process.next_message().await {
                    Err(e) => {
                        let _ = tx.send(Err(e)).await;
                        break;
                    }
                    Ok(None) => break,
                    Ok(Some(msg)) => {
                        let terminal = matches!(msg, Message::Result(_));
                        if tx.send(Ok(msg)).await.is_err() {
                            break;
                        }
                        if terminal {
                            break;
                        }
                    }
                }
            }
            process.kill().await;
        });

        QueryStream { rx }
    }

    const INIT_LINE: &str = r#"{"type":"system","subtype":"init","session_id":"s1","model":"m","tools":[],"mcp_servers":[],"permission_mode":"default","claude_code_version":"0.0.0","cwd":"/tmp"}"#;
    const RESULT_LINE: &str = r#"{"type":"result","subtype":"success","session_id":"s1","result":"Hello from mock!","duration_ms":1,"duration_api_ms":1,"is_error":false,"num_turns":1,"stop_reason":"end_turn","total_cost_usd":0.0,"usage":{"input_tokens":1,"output_tokens":1}}"#;

    #[tokio::test]
    async fn stream_yields_all_messages() {
        let stream = mock_stream(&[INIT_LINE, RESULT_LINE]);
        let messages: Vec<_> = stream.collect().await;
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().all(|m| m.is_ok()));
    }

    #[tokio::test]
    async fn stream_terminates_after_result() {
        // Add an extra line after result — stream must not emit it
        let extra = r#"{"type":"system","subtype":"init","session_id":"s2","model":"m","tools":[],"mcp_servers":[],"permission_mode":"default","claude_code_version":"0.0.0","cwd":"/tmp"}"#;
        let stream = mock_stream(&[INIT_LINE, RESULT_LINE, extra]);
        let messages: Vec<_> = stream.collect().await;
        // Stream must stop at the result; the extra line is never consumed
        assert_eq!(messages.len(), 2);
    }

    #[tokio::test]
    async fn stream_last_message_is_result() {
        let stream = mock_stream(&[INIT_LINE, RESULT_LINE]);
        let messages: Vec<_> = stream.collect().await;
        let last = messages.last().unwrap().as_ref().unwrap();
        assert!(matches!(last, Message::Result(ResultMessage::Success(_))));
    }

    #[tokio::test]
    async fn stream_extracts_session_id_and_result_text() {
        let stream = mock_stream(&[INIT_LINE, RESULT_LINE]);
        let messages: Vec<_> = stream.collect().await;

        // First message: system/init — session_id accessible
        let first = messages[0].as_ref().unwrap();
        assert_eq!(first.session_id(), "s1");

        // Last message: result — result_text accessible
        let last = messages.last().unwrap().as_ref().unwrap();
        if let Message::Result(r) = last {
            assert_eq!(r.result_text(), Some("Hello from mock!"));
            assert_eq!(r.session_id(), "s1");
        } else {
            panic!("expected Result");
        }
    }

    #[tokio::test]
    async fn stream_handles_empty_lines_in_output() {
        // Claude's output sometimes contains blank lines between JSON objects
        let stream = mock_stream(&[INIT_LINE, "", "  ", RESULT_LINE]);
        let messages: Vec<_> = stream.collect().await;
        // Blank lines are skipped; we still get exactly 2 real messages
        assert_eq!(messages.len(), 2);
    }
}
