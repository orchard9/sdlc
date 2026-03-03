use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// SMTP delivery configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SmtpConfig {
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub username: String,
    pub password: String,
    pub from: String,
    pub to: Vec<String>,
}

fn default_smtp_port() -> u16 {
    587
}

/// Top-level Telegram digest configuration (nested under `telegram:` in config.yaml).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DigestConfig {
    pub bot_token: String,
    #[serde(default)]
    pub chat_ids: Vec<String>,
    pub smtp: SmtpConfig,
    #[serde(default = "default_window_hours")]
    pub window_hours: u32,
    #[serde(default = "default_subject_prefix")]
    pub subject_prefix: String,
    #[serde(default = "default_max_messages")]
    pub max_messages_per_chat: u32,
}

fn default_window_hours() -> u32 {
    24
}

fn default_subject_prefix() -> String {
    "[sdlc Digest]".to_string()
}

fn default_max_messages() -> u32 {
    100
}

impl DigestConfig {
    /// Resolve `${ENV_VAR}` placeholders in sensitive fields using `std::env::var`.
    /// Returns an error if a referenced env var is not set.
    pub fn resolve_env(mut self) -> Result<Self, String> {
        self.bot_token = resolve_placeholder(&self.bot_token)?;
        self.smtp.username = resolve_placeholder(&self.smtp.username)?;
        self.smtp.password = resolve_placeholder(&self.smtp.password)?;
        self.smtp.host = resolve_placeholder(&self.smtp.host)?;
        self.smtp.from = resolve_placeholder(&self.smtp.from)?;
        Ok(self)
    }
}

/// Expand a single `${VAR}` placeholder. Returns the literal string if no
/// placeholder is present. Errors if the referenced variable is unset.
fn resolve_placeholder(value: &str) -> Result<String, String> {
    if let Some(inner) = value.strip_prefix("${").and_then(|s| s.strip_suffix('}')) {
        std::env::var(inner).map_err(|_| {
            format!("env var '{inner}' is not set (referenced in config as '${{{inner}}}')")
        })
    } else {
        Ok(value.to_string())
    }
}

// ---------------------------------------------------------------------------
// Telegram API types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

impl TelegramUser {
    /// Friendly display name: username > first+last > first.
    pub fn display_name(&self) -> String {
        if let Some(u) = &self.username {
            return format!("@{u}");
        }
        match &self.last_name {
            Some(last) => format!("{} {last}", self.first_name),
            None => self.first_name.clone(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TelegramChat {
    pub id: i64,
    pub title: Option<String>,
    pub username: Option<String>,
    #[serde(rename = "type")]
    pub type_: String,
}

impl TelegramChat {
    pub fn display_name(&self) -> String {
        if let Some(title) = &self.title {
            return title.clone();
        }
        if let Some(u) = &self.username {
            return format!("@{u}");
        }
        format!("Chat {}", self.id)
    }
}

/// A single Telegram message as returned by `getUpdates`.
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramMessage {
    pub message_id: i64,
    pub from: Option<TelegramUser>,
    pub chat: TelegramChat,
    /// Unix timestamp (seconds since epoch).
    pub date: i64,
    pub text: Option<String>,
}

impl TelegramMessage {
    pub fn timestamp(&self) -> DateTime<Utc> {
        DateTime::from_timestamp(self.date, 0).unwrap_or(Utc::now())
    }
}

/// A single update item from `getUpdates`.
#[derive(Debug, Clone, Deserialize)]
pub struct TelegramUpdate {
    pub update_id: i64,
    pub message: Option<TelegramMessage>,
}

// ---------------------------------------------------------------------------
// Digest types
// ---------------------------------------------------------------------------

/// A single message in the formatted digest.
#[derive(Debug, Clone)]
pub struct DigestMessage {
    pub timestamp: DateTime<Utc>,
    pub author: String,
    pub text: String,
}

/// All messages from a single chat, ready for rendering.
#[derive(Debug, Clone)]
pub struct ChatDigest {
    pub chat_id: String,
    pub chat_name: String,
    pub messages: Vec<DigestMessage>,
}

/// Full digest summary ready for email formatting.
#[derive(Debug, Clone)]
pub struct DigestSummary {
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub chats: Vec<ChatDigest>,
    pub total_messages: usize,
}

// ---------------------------------------------------------------------------
// Run result
// ---------------------------------------------------------------------------

/// Result returned from a full DigestRunner::run() call.
#[derive(Debug)]
pub struct DigestRunResult {
    pub summary: DigestSummary,
    pub dry_run: bool,
    /// Recipient addresses actually sent to (empty when dry_run = true).
    pub sent_to: Vec<String>,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_placeholder_literal() {
        let result = resolve_placeholder("smtp.example.com").unwrap();
        assert_eq!(result, "smtp.example.com");
    }

    #[test]
    fn resolve_placeholder_env_var() {
        std::env::set_var("TEST_TG_TOKEN_AAA", "my-secret-token");
        let result = resolve_placeholder("${TEST_TG_TOKEN_AAA}").unwrap();
        assert_eq!(result, "my-secret-token");
        std::env::remove_var("TEST_TG_TOKEN_AAA");
    }

    #[test]
    fn resolve_placeholder_missing_var() {
        std::env::remove_var("__SURELY_UNSET_XYZ__");
        let result = resolve_placeholder("${__SURELY_UNSET_XYZ__}");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("__SURELY_UNSET_XYZ__"));
    }

    #[test]
    fn telegram_user_display_name_username() {
        let user = TelegramUser {
            id: 1,
            first_name: "Alice".to_string(),
            last_name: None,
            username: Some("alice_dev".to_string()),
        };
        assert_eq!(user.display_name(), "@alice_dev");
    }

    #[test]
    fn telegram_user_display_name_full_name() {
        let user = TelegramUser {
            id: 2,
            first_name: "Bob".to_string(),
            last_name: Some("Smith".to_string()),
            username: None,
        };
        assert_eq!(user.display_name(), "Bob Smith");
    }

    #[test]
    fn telegram_user_display_name_first_only() {
        let user = TelegramUser {
            id: 3,
            first_name: "Charlie".to_string(),
            last_name: None,
            username: None,
        };
        assert_eq!(user.display_name(), "Charlie");
    }

    #[test]
    fn config_defaults() {
        let yaml = r#"
bot_token: "tok"
chat_ids: []
smtp:
  host: smtp.example.com
  username: user
  password: pass
  from: from@example.com
  to: ["to@example.com"]
"#;
        let cfg: DigestConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.window_hours, 24);
        assert_eq!(cfg.max_messages_per_chat, 100);
        assert_eq!(cfg.subject_prefix, "[sdlc Digest]");
        assert_eq!(cfg.smtp.port, 587);
    }

    #[test]
    fn resolve_env_applies_to_all_sensitive_fields() {
        std::env::set_var("TG_RESOLVE_TEST_TOKEN", "bot-token-value");
        std::env::set_var("TG_RESOLVE_TEST_USER", "smtp-user");
        std::env::set_var("TG_RESOLVE_TEST_PASS", "smtp-pass");

        let cfg = DigestConfig {
            bot_token: "${TG_RESOLVE_TEST_TOKEN}".to_string(),
            chat_ids: vec![],
            smtp: SmtpConfig {
                host: "smtp.example.com".to_string(),
                port: 587,
                username: "${TG_RESOLVE_TEST_USER}".to_string(),
                password: "${TG_RESOLVE_TEST_PASS}".to_string(),
                from: "from@example.com".to_string(),
                to: vec!["to@example.com".to_string()],
            },
            window_hours: 24,
            subject_prefix: "[Test]".to_string(),
            max_messages_per_chat: 100,
        };

        let resolved = cfg.resolve_env().unwrap();
        assert_eq!(resolved.bot_token, "bot-token-value");
        assert_eq!(resolved.smtp.username, "smtp-user");
        assert_eq!(resolved.smtp.password, "smtp-pass");

        std::env::remove_var("TG_RESOLVE_TEST_TOKEN");
        std::env::remove_var("TG_RESOLVE_TEST_USER");
        std::env::remove_var("TG_RESOLVE_TEST_PASS");
    }
}
