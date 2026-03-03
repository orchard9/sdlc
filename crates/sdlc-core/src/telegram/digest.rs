use chrono::{DateTime, Datelike, Timelike, Utc, Weekday};
use std::collections::HashMap;

use super::types::{ChatDigest, DigestConfig, DigestMessage, DigestSummary, TelegramUpdate};

// ---------------------------------------------------------------------------
// DigestBuilder
// ---------------------------------------------------------------------------

pub struct DigestBuilder {
    window_hours: u32,
    chat_ids: Vec<String>,
    max_messages: u32,
}

impl DigestBuilder {
    pub fn new(config: &DigestConfig) -> Self {
        Self {
            window_hours: config.window_hours,
            chat_ids: config.chat_ids.clone(),
            max_messages: config.max_messages_per_chat,
        }
    }

    /// Build a `DigestSummary` from raw updates, filtered to the time window ending at `now`.
    pub fn build(&self, updates: Vec<TelegramUpdate>, now: DateTime<Utc>) -> DigestSummary {
        let window_secs = self.window_hours as i64 * 3600;
        let period_start = now - chrono::Duration::seconds(window_secs);
        let period_end = now;

        // Group messages by chat ID.
        let mut by_chat: HashMap<i64, (String, Vec<DigestMessage>)> = HashMap::new();

        for update in updates {
            let Some(msg) = update.message else { continue };
            let ts = msg.timestamp();

            // Time window filter
            if ts < period_start || ts > period_end {
                continue;
            }

            // Chat ID filter (empty = all chats allowed)
            let chat_id_str = msg.chat.id.to_string();
            if !self.chat_ids.is_empty() && !self.chat_ids.contains(&chat_id_str) {
                continue;
            }

            // Skip messages without text
            let text = match &msg.text {
                Some(t) if !t.is_empty() => t.clone(),
                _ => continue,
            };

            let author = msg
                .from
                .as_ref()
                .map(|u| u.display_name())
                .unwrap_or_else(|| "Unknown".to_string());

            let entry = by_chat
                .entry(msg.chat.id)
                .or_insert_with(|| (msg.chat.display_name(), Vec::new()));

            entry.1.push(DigestMessage {
                timestamp: ts,
                author,
                text,
            });
        }

        // Sort each chat's messages by timestamp and truncate to max.
        let mut chats: Vec<ChatDigest> = by_chat
            .into_iter()
            .map(|(chat_id, (name, mut messages))| {
                messages.sort_by_key(|m| m.timestamp);
                // Keep most recent N
                let max = self.max_messages as usize;
                if messages.len() > max {
                    messages = messages.into_iter().rev().take(max).rev().collect();
                }
                ChatDigest {
                    chat_id: chat_id.to_string(),
                    chat_name: name,
                    messages,
                }
            })
            .collect();

        // Sort chats by first-message timestamp for deterministic ordering.
        chats.sort_by_key(|c| {
            c.messages
                .first()
                .map(|m| m.timestamp)
                .unwrap_or(period_start)
        });

        let total_messages = chats.iter().map(|c| c.messages.len()).sum();

        DigestSummary {
            period_start,
            period_end,
            chats,
            total_messages,
        }
    }
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

impl DigestSummary {
    /// Email subject line.
    pub fn format_subject(&self, prefix: &str) -> String {
        let date = self.period_end.format("%Y-%m-%d");
        let chats = self.chats.len();
        let msgs = self.total_messages;
        format!("{prefix} {date} — {msgs} messages across {chats} chats")
    }

    /// Plain-text email body.
    pub fn format_plain_text(&self) -> String {
        let weekday = weekday_name(self.period_end.weekday());
        let date_long = self.period_end.format("%B %-d %Y");
        let start_str = self.period_start.format("%Y-%m-%d %H:%M UTC");
        let end_str = self.period_end.format("%Y-%m-%d %H:%M UTC");

        let mut out = String::new();
        out.push_str(&format!("sdlc Daily Digest — {weekday}, {date_long}\n"));
        out.push_str(&format!("Period: {start_str} → {end_str}\n"));

        for chat in &self.chats {
            out.push('\n');
            let header = format!("── {} ({} messages) ", chat.chat_name, chat.messages.len());
            let fill = "─".repeat(60usize.saturating_sub(header.len()));
            out.push_str(&format!("{header}{fill}\n"));

            for msg in &chat.messages {
                let hhmm = format!("{:02}:{:02}", msg.timestamp.hour(), msg.timestamp.minute());
                // Truncate long messages at 120 chars for readability
                let text = if msg.text.len() > 120 {
                    format!("{}…", &msg.text[..117])
                } else {
                    msg.text.clone()
                };
                out.push_str(&format!(" {hhmm}  {:<15}  {text}\n", msg.author));
            }
        }

        out.push('\n');
        out.push_str(&format!("{sep}\n", sep = "─".repeat(60)));
        out.push_str(&format!(
            "{} total messages  |  Generated by sdlc telegram digest\n",
            self.total_messages
        ));

        out
    }

    /// HTML email body (inline-CSS, no external resources).
    pub fn format_html(&self) -> String {
        let weekday = weekday_name(self.period_end.weekday());
        let date_long = self.period_end.format("%B %-d %Y");
        let start_str = self.period_start.format("%Y-%m-%d %H:%M UTC");
        let end_str = self.period_end.format("%Y-%m-%d %H:%M UTC");

        let mut body = String::new();

        body.push_str(&format!(
            r#"<div style="font-family:monospace;max-width:700px;margin:0 auto;padding:16px">
<h2 style="margin:0 0 4px">sdlc Daily Digest</h2>
<p style="margin:0 0 12px;color:#666">{weekday}, {date_long} &nbsp;·&nbsp; {start_str} → {end_str}</p>
"#
        ));

        for chat in &self.chats {
            body.push_str(&format!(
                r#"<h3 style="margin:16px 0 4px;border-bottom:1px solid #ccc;padding-bottom:4px">
{} <span style="font-size:0.85em;color:#888">({} messages)</span>
</h3>
<table style="width:100%;border-collapse:collapse;font-size:0.9em">
"#,
                html_escape(&chat.chat_name),
                chat.messages.len()
            ));

            for msg in &chat.messages {
                let hhmm = format!("{:02}:{:02}", msg.timestamp.hour(), msg.timestamp.minute());
                let text = if msg.text.len() > 200 {
                    format!("{}…", html_escape(&msg.text[..197]))
                } else {
                    html_escape(&msg.text)
                };
                body.push_str(&format!(
                    r#"<tr>
  <td style="color:#888;white-space:nowrap;padding:2px 8px 2px 0;vertical-align:top">{hhmm}</td>
  <td style="white-space:nowrap;padding:2px 8px 2px 0;vertical-align:top;font-weight:bold">{}</td>
  <td style="padding:2px 0;vertical-align:top">{text}</td>
</tr>
"#,
                    html_escape(&msg.author)
                ));
            }

            body.push_str("</table>\n");
        }

        body.push_str(&format!(
            r#"<hr style="margin:16px 0;border:none;border-top:1px solid #ddd">
<p style="color:#888;font-size:0.8em">{} total messages &nbsp;·&nbsp; Generated by sdlc telegram digest</p>
</div>"#,
            self.total_messages
        ));

        format!(
            "<!DOCTYPE html><html><head><meta charset=\"utf-8\"></head><body>{body}</body></html>"
        )
    }
}

fn weekday_name(w: Weekday) -> &'static str {
    match w {
        Weekday::Mon => "Monday",
        Weekday::Tue => "Tuesday",
        Weekday::Wed => "Wednesday",
        Weekday::Thu => "Thursday",
        Weekday::Fri => "Friday",
        Weekday::Sat => "Saturday",
        Weekday::Sun => "Sunday",
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegram::types::{SmtpConfig, TelegramChat, TelegramMessage, TelegramUser};
    use chrono::TimeZone;

    fn make_config(chat_ids: Vec<String>, window_hours: u32, max_messages: u32) -> DigestConfig {
        DigestConfig {
            bot_token: "tok".to_string(),
            chat_ids,
            smtp: SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                username: "user".to_string(),
                password: "pass".to_string(),
                from: "from@example.com".to_string(),
                to: vec!["to@example.com".to_string()],
            },
            window_hours,
            subject_prefix: "[Test]".to_string(),
            max_messages_per_chat: max_messages,
        }
    }

    fn make_update(
        update_id: i64,
        chat_id: i64,
        chat_title: &str,
        date: i64,
        text: &str,
        author: &str,
    ) -> TelegramUpdate {
        TelegramUpdate {
            update_id,
            message: Some(TelegramMessage {
                message_id: update_id * 10,
                from: Some(TelegramUser {
                    id: update_id,
                    first_name: author.to_string(),
                    last_name: None,
                    username: None,
                }),
                chat: TelegramChat {
                    id: chat_id,
                    title: Some(chat_title.to_string()),
                    username: None,
                    type_: "supergroup".to_string(),
                },
                date,
                text: Some(text.to_string()),
            }),
        }
    }

    #[test]
    fn empty_updates_returns_empty_summary() {
        let cfg = make_config(vec![], 24, 100);
        let builder = DigestBuilder::new(&cfg);
        let now = Utc::now();
        let summary = builder.build(vec![], now);
        assert_eq!(summary.total_messages, 0);
        assert!(summary.chats.is_empty());
    }

    #[test]
    fn time_window_filtering() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let in_window = now.timestamp() - 3600; // 1 hour ago — inside 24h window
        let out_of_window = now.timestamp() - 25 * 3600; // 25 hours ago — outside

        let cfg = make_config(vec![], 24, 100);
        let builder = DigestBuilder::new(&cfg);
        let updates = vec![
            make_update(1, -1001, "Alpha", in_window, "hello", "Alice"),
            make_update(2, -1001, "Alpha", out_of_window, "old msg", "Bob"),
        ];
        let summary = builder.build(updates, now);
        assert_eq!(summary.total_messages, 1);
        assert_eq!(summary.chats[0].messages[0].text, "hello");
    }

    #[test]
    fn chat_id_filtering() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let ts = now.timestamp() - 60;

        let cfg = make_config(vec!["-1001".to_string()], 24, 100);
        let builder = DigestBuilder::new(&cfg);
        let updates = vec![
            make_update(1, -1001, "Alpha", ts, "msg from alpha", "Alice"),
            make_update(2, -1002, "Beta", ts, "msg from beta", "Bob"),
        ];
        let summary = builder.build(updates, now);
        assert_eq!(summary.total_messages, 1);
        assert_eq!(summary.chats[0].chat_id, "-1001");
    }

    #[test]
    fn max_messages_truncation() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let base_ts = now.timestamp() - 3600;
        let cfg = make_config(vec![], 24, 5);
        let builder = DigestBuilder::new(&cfg);

        let updates: Vec<TelegramUpdate> = (0..10)
            .map(|i| make_update(i, -1001, "Alpha", base_ts + i, &format!("msg {i}"), "Alice"))
            .collect();

        let summary = builder.build(updates, now);
        assert_eq!(summary.chats[0].messages.len(), 5);
        // Should keep the most recent 5
        assert_eq!(summary.chats[0].messages[0].text, "msg 5");
    }

    #[test]
    fn multi_chat_grouping() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let ts = now.timestamp() - 60;
        let cfg = make_config(vec![], 24, 100);
        let builder = DigestBuilder::new(&cfg);

        let updates = vec![
            make_update(1, -1001, "Alpha", ts, "a1", "Alice"),
            make_update(2, -1002, "Beta", ts + 1, "b1", "Bob"),
            make_update(3, -1001, "Alpha", ts + 2, "a2", "Alice"),
        ];

        let summary = builder.build(updates, now);
        assert_eq!(summary.total_messages, 3);
        assert_eq!(summary.chats.len(), 2);
    }

    #[test]
    fn format_subject() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 18, 0, 0).unwrap();
        let summary = DigestSummary {
            period_start: now - chrono::Duration::hours(24),
            period_end: now,
            chats: vec![
                ChatDigest {
                    chat_id: "-1001".to_string(),
                    chat_name: "Alpha".to_string(),
                    messages: vec![
                        DigestMessage {
                            timestamp: now - chrono::Duration::hours(1),
                            author: "Alice".to_string(),
                            text: "hello".to_string(),
                        };
                        30
                    ],
                },
                ChatDigest {
                    chat_id: "-1002".to_string(),
                    chat_name: "Beta".to_string(),
                    messages: vec![
                        DigestMessage {
                            timestamp: now - chrono::Duration::hours(2),
                            author: "Bob".to_string(),
                            text: "world".to_string(),
                        };
                        12
                    ],
                },
            ],
            total_messages: 42,
        };

        let subject = summary.format_subject("[sdlc Digest]");
        assert_eq!(
            subject,
            "[sdlc Digest] 2026-03-03 — 42 messages across 2 chats"
        );
    }

    #[test]
    fn format_plain_text_structure() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).unwrap();
        let summary = DigestSummary {
            period_start: now - chrono::Duration::hours(24),
            period_end: now,
            chats: vec![ChatDigest {
                chat_id: "-1001".to_string(),
                chat_name: "Project Alpha".to_string(),
                messages: vec![DigestMessage {
                    timestamp: Utc.with_ymd_and_hms(2026, 3, 3, 10, 30, 0).unwrap(),
                    author: "Alice".to_string(),
                    text: "Deployed v1.4".to_string(),
                }],
            }],
            total_messages: 1,
        };

        let text = summary.format_plain_text();
        assert!(text.contains("sdlc Daily Digest"));
        assert!(text.contains("Project Alpha"));
        assert!(text.contains("Alice"));
        assert!(text.contains("Deployed v1.4"));
        assert!(text.contains("10:30"));
        assert!(text.contains("1 total messages"));
    }

    #[test]
    fn format_html_structure() {
        let now = Utc.with_ymd_and_hms(2026, 3, 3, 12, 0, 0).unwrap();
        let summary = DigestSummary {
            period_start: now - chrono::Duration::hours(24),
            period_end: now,
            chats: vec![ChatDigest {
                chat_id: "-1001".to_string(),
                chat_name: "Project Alpha".to_string(),
                messages: vec![DigestMessage {
                    timestamp: Utc.with_ymd_and_hms(2026, 3, 3, 10, 30, 0).unwrap(),
                    author: "Alice".to_string(),
                    text: "Deployed v1.4".to_string(),
                }],
            }],
            total_messages: 1,
        };

        let html = summary.format_html();
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Project Alpha"));
        assert!(html.contains("Alice"));
        assert!(html.contains("Deployed v1.4"));
        assert!(!html.contains("http")); // no external resources
    }

    #[test]
    fn html_escape_handles_special_chars() {
        let escaped = html_escape("<script>alert('xss')</script>");
        assert!(escaped.contains("&lt;script&gt;"));
        assert!(!escaped.contains('<'));
        assert!(!escaped.contains('>'));
    }
}
