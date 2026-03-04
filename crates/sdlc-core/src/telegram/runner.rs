use crate::error::Result;
use crate::io;
use chrono::Utc;
use serde_json::json;
use std::path::Path;

use super::{
    digest::DigestBuilder,
    mailer::ResendMailer,
    poll::MessageStore,
    types::{DigestConfig, DigestRunResult},
};

/// Orchestrates: fetch updates → build digest → (optionally) send email → write run record.
pub struct DigestRunner {
    config: DigestConfig,
}

impl DigestRunner {
    pub fn new(config: DigestConfig) -> Self {
        Self { config }
    }

    /// Run the full digest pipeline.
    ///
    /// If `dry_run` is true, the email is not sent but the digest is returned.
    /// A run record is always written to `.sdlc/.runs/` (or `root` if provided).
    pub fn run(&self, dry_run: bool, root: Option<&Path>) -> Result<DigestRunResult> {
        let started_at = Utc::now();

        let result = self.execute(dry_run);

        // Write run record regardless of success/failure (best-effort).
        if let Some(root) = root {
            let run_record = match &result {
                Ok(r) => build_run_record(
                    &started_at,
                    Utc::now(),
                    "completed",
                    &self.config,
                    Some(r),
                    None,
                    dry_run,
                ),
                Err(e) => build_run_record(
                    &started_at,
                    Utc::now(),
                    "failed",
                    &self.config,
                    None,
                    Some(&e.to_string()),
                    dry_run,
                ),
            };

            let run_id = started_at.format("%Y%m%d-%H%M%S-tgd").to_string();
            let runs_dir = root.join(".sdlc").join(".runs");
            let run_path = runs_dir.join(format!("{run_id}.json"));

            // Non-fatal: don't let run record I/O shadow the main error.
            if let Ok(json) = serde_json::to_string_pretty(&run_record) {
                let _ = io::atomic_write(&run_path, json.as_bytes());
            }
        }

        result
    }

    fn execute(&self, dry_run: bool) -> Result<DigestRunResult> {
        let store = MessageStore::open(&self.config.db_path)?;
        let now = Utc::now();
        let updates = store.query_messages_in_window(self.config.window_hours, now)?;

        let builder = DigestBuilder::new(&self.config);
        let summary = builder.build(updates, now);

        if dry_run {
            return Ok(DigestRunResult {
                summary,
                dry_run: true,
                sent_to: vec![],
            });
        }

        if summary.total_messages == 0 {
            // Nothing to send; still succeed.
            return Ok(DigestRunResult {
                summary,
                dry_run: false,
                sent_to: vec![],
            });
        }

        let subject = summary.format_subject(&self.config.subject_prefix);
        let plain = summary.format_plain_text();
        let html = summary.format_html();

        let mailer = ResendMailer::new(self.config.resend.clone());
        mailer.send(&subject, &plain, &html)?;

        let sent_to = self.config.resend.to.clone();
        Ok(DigestRunResult {
            summary,
            dry_run: false,
            sent_to,
        })
    }
}

fn build_run_record(
    started_at: &chrono::DateTime<Utc>,
    completed_at: chrono::DateTime<Utc>,
    status: &str,
    config: &DigestConfig,
    result: Option<&DigestRunResult>,
    error: Option<&str>,
    dry_run: bool,
) -> serde_json::Value {
    let (chat_count, message_count, period_start, period_end) = result
        .map(|r| {
            (
                r.summary.chats.len(),
                r.summary.total_messages,
                r.summary.period_start.to_rfc3339(),
                r.summary.period_end.to_rfc3339(),
            )
        })
        .unwrap_or_default();

    let summary_text = result.map(|r| {
        if dry_run {
            format!(
                "Dry run: {} messages across {} chats",
                r.summary.total_messages,
                r.summary.chats.len()
            )
        } else {
            format!(
                "Sent digest: {} messages across {} chats",
                r.summary.total_messages,
                r.summary.chats.len()
            )
        }
    });

    json!({
        "id": started_at.format("%Y%m%d-%H%M%S-tgd").to_string(),
        "kind": "telegram-digest",
        "status": status,
        "started_at": started_at.to_rfc3339(),
        "completed_at": completed_at.to_rfc3339(),
        "summary": summary_text,
        "error": error,
        "metadata": {
            "period_start": period_start,
            "period_end": period_end,
            "chat_count": chat_count,
            "message_count": message_count,
            "recipients": result.map(|r| r.sent_to.clone()).unwrap_or_default(),
            "dry_run": dry_run,
            // Never include bot_token or api_key.
            "resend_from": config.resend.from,
        }
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegram::types::{DigestConfig, ResendConfig};

    fn make_config(bot_token: &str) -> DigestConfig {
        DigestConfig {
            bot_token: bot_token.to_string(),
            chat_ids: vec![],
            resend: ResendConfig {
                api_key: "re_secret_key".to_string(),
                from: "from@example.com".to_string(),
                to: vec!["team@example.com".to_string()],
            },
            window_hours: 24,
            subject_prefix: "[Test]".to_string(),
            max_messages_per_chat: 100,
            db_path: std::path::PathBuf::from("/dev/null"),
        }
    }

    #[test]
    fn run_record_has_no_credentials() {
        let cfg = make_config("secret-bot-token");
        let now = Utc::now();
        let record = build_run_record(
            &now,
            Utc::now(),
            "failed",
            &cfg,
            None,
            Some("auth error"),
            false,
        );
        let json_str = record.to_string();
        // Credentials must NOT appear in the run record
        assert!(!json_str.contains("secret-bot-token"));
        assert!(!json_str.contains("re_secret_key"));
        // From address is acceptable in metadata
        assert!(json_str.contains("from@example.com"));
    }

    #[test]
    fn run_record_structure() {
        use chrono::TimeZone;
        let cfg = make_config("tok");
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let record = build_run_record(&now, now, "completed", &cfg, None, None, true);
        assert_eq!(record["kind"], "telegram-digest");
        assert_eq!(record["status"], "completed");
        assert_eq!(record["metadata"]["dry_run"], true);
    }
}
