//! Telegram long-polling support: `poll_loop`, `get_me`, `MessageStore`, `TelegramConfig`.

use crate::config::TelegramConfigYaml;
use crate::error::{Result, SdlcError};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// TelegramConfig (resolved, ready to use)
// ---------------------------------------------------------------------------

/// Resolved Telegram configuration: bot token, db path.
pub struct TelegramConfig {
    pub bot_token: String,
    pub db_path: PathBuf,
    pub poll_timeout_secs: u64,
    pub base_url: String,
}

impl TelegramConfig {
    /// Resolve configuration from environment variables and optional YAML config.
    ///
    /// Precedence: env var > YAML field > default.
    pub fn from_env_and_yaml(yaml: Option<&TelegramConfigYaml>, root: &Path) -> Result<Self> {
        let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
            .ok()
            .or_else(|| yaml.and_then(|y| y.bot_token.clone()))
            .ok_or(SdlcError::TelegramTokenMissing)?;

        let db_path = yaml
            .and_then(|y| y.db_path.as_deref())
            .map(PathBuf::from)
            .unwrap_or_else(|| root.join(".sdlc").join("telegram").join("messages.db"));

        let poll_timeout_secs = yaml.and_then(|y| y.poll_timeout_secs).unwrap_or(30);

        Ok(Self {
            bot_token,
            db_path,
            poll_timeout_secs,
            base_url: "https://api.telegram.org".to_string(),
        })
    }
}

// ---------------------------------------------------------------------------
// Bot identity
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct BotUser {
    pub id: i64,
    pub first_name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub is_bot: bool,
}

#[derive(Deserialize)]
struct ApiResult<T> {
    ok: bool,
    result: Option<T>,
    description: Option<String>,
}

/// Call `getMe` and return the bot's identity.
pub fn get_me(config: &TelegramConfig) -> Result<BotUser> {
    let url = format!("{}/bot{}/getMe", config.base_url, config.bot_token);
    let resp: ApiResult<BotUser> = reqwest::blocking::get(&url)
        .map_err(|e| SdlcError::TelegramApi(format!("network error: {e}")))?
        .json()
        .map_err(|e| SdlcError::TelegramApi(format!("parse error: {e}")))?;

    if !resp.ok {
        return Err(SdlcError::TelegramApi(
            resp.description
                .unwrap_or_else(|| "getMe failed".to_string()),
        ));
    }

    resp.result
        .ok_or_else(|| SdlcError::TelegramApi("getMe returned no result".to_string()))
}

// ---------------------------------------------------------------------------
// MessageStore (SQLite)
// ---------------------------------------------------------------------------

/// Persistent store for Telegram messages backed by SQLite.
pub struct MessageStore {
    conn: rusqlite::Connection,
}

impl MessageStore {
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn =
            rusqlite::Connection::open(db_path).map_err(|e| SdlcError::Sqlite(e.to_string()))?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS messages (
                update_id   INTEGER PRIMARY KEY,
                chat_id     INTEGER NOT NULL,
                chat_title  TEXT,
                user_id     INTEGER,
                username    TEXT,
                first_name  TEXT,
                date        INTEGER NOT NULL,
                text        TEXT
            );
            CREATE TABLE IF NOT EXISTS meta (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )
        .map_err(|e| SdlcError::Sqlite(e.to_string()))?;

        Ok(Self { conn })
    }

    /// Return total number of stored messages.
    pub fn message_count(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM messages", [], |r| r.get(0))
            .map_err(|e| SdlcError::Sqlite(e.to_string()))?;
        Ok(count)
    }

    /// Return the stored polling offset (last update_id + 1), if any.
    pub fn get_offset(&self) -> Result<Option<i64>> {
        let result: rusqlite::Result<String> =
            self.conn
                .query_row("SELECT value FROM meta WHERE key = 'offset'", [], |r| {
                    r.get(0)
                });
        match result {
            Ok(v) => Ok(v.parse().ok()),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(SdlcError::Sqlite(e.to_string())),
        }
    }

    /// Return the (oldest, newest) Unix timestamps of stored messages, if any.
    pub fn time_range(&self) -> Result<Option<(i64, i64)>> {
        let result: rusqlite::Result<(i64, i64)> =
            self.conn
                .query_row("SELECT MIN(date), MAX(date) FROM messages", [], |r| {
                    Ok((r.get(0)?, r.get(1)?))
                });
        match result {
            Ok((min, max)) => Ok(Some((min, max))),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(rusqlite::Error::InvalidColumnType(..)) => Ok(None),
            Err(e) => Err(SdlcError::Sqlite(e.to_string())),
        }
    }

    fn set_offset(&self, offset: i64) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO meta (key, value) VALUES ('offset', ?1)",
                rusqlite::params![offset.to_string()],
            )
            .map_err(|e| SdlcError::Sqlite(e.to_string()))?;
        Ok(())
    }

    fn insert_message(&self, update_id: i64, msg: &serde_json::Value) -> Result<()> {
        let chat = &msg["chat"];
        let from = &msg["from"];

        self.conn
            .execute(
                "INSERT OR IGNORE INTO messages
                 (update_id, chat_id, chat_title, user_id, username, first_name, date, text)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    update_id,
                    chat["id"].as_i64().unwrap_or(0),
                    chat["title"].as_str().unwrap_or(""),
                    from["id"].as_i64().unwrap_or(0),
                    from["username"].as_str().unwrap_or(""),
                    from["first_name"].as_str().unwrap_or(""),
                    msg["date"].as_i64().unwrap_or(0),
                    msg["text"].as_str().unwrap_or(""),
                ],
            )
            .map_err(|e| SdlcError::Sqlite(e.to_string()))?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// poll_loop
// ---------------------------------------------------------------------------

/// Long-poll loop: fetches updates from Telegram and stores them in `store`.
///
/// Runs until `shutdown` is set to true or an unrecoverable error occurs.
pub fn poll_loop(
    config: &TelegramConfig,
    store: &MessageStore,
    shutdown: Arc<AtomicBool>,
) -> Result<()> {
    let mut offset: Option<i64> = store.get_offset()?;
    let client = reqwest::blocking::Client::new();

    while !shutdown.load(Ordering::Relaxed) {
        let mut url = format!(
            "{}/bot{}/getUpdates?timeout={}&allowed_updates=[\"message\"]",
            config.base_url, config.bot_token, config.poll_timeout_secs
        );
        if let Some(off) = offset {
            url.push_str(&format!("&offset={off}"));
        }

        let resp = match client.get(&url).send() {
            Ok(r) => r,
            Err(e) => {
                eprintln!("warn: poll request failed: {e}");
                std::thread::sleep(std::time::Duration::from_secs(5));
                continue;
            }
        };

        if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
            return Err(SdlcError::TelegramTokenMissing);
        }

        let body: serde_json::Value = match resp.json() {
            Ok(v) => v,
            Err(e) => {
                eprintln!("warn: failed to parse poll response: {e}");
                std::thread::sleep(std::time::Duration::from_secs(2));
                continue;
            }
        };

        if !body["ok"].as_bool().unwrap_or(false) {
            eprintln!(
                "warn: Telegram API error: {}",
                body["description"].as_str().unwrap_or("unknown")
            );
            std::thread::sleep(std::time::Duration::from_secs(5));
            continue;
        }

        let updates = match body["result"].as_array() {
            Some(arr) => arr.clone(),
            None => continue,
        };

        for update in &updates {
            let update_id = match update["update_id"].as_i64() {
                Some(id) => id,
                None => continue,
            };

            if let Some(msg) = update["message"].as_object() {
                let msg_val = serde_json::Value::Object(msg.clone());
                if let Err(e) = store.insert_message(update_id, &msg_val) {
                    eprintln!("warn: failed to store message {update_id}: {e}");
                }
            }

            let new_offset = update_id + 1;
            offset = Some(new_offset);
            if let Err(e) = store.set_offset(new_offset) {
                eprintln!("warn: failed to save offset: {e}");
            }
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Serialize tests that mutate TELEGRAM_BOT_TOKEN to prevent races.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn make_store() -> (MessageStore, TempDir) {
        let dir = TempDir::new().unwrap();
        let db_path = dir.path().join("messages.db");
        let store = MessageStore::open(&db_path).unwrap();
        (store, dir)
    }

    #[test]
    fn test_open_creates_schema() {
        let (store, _dir) = make_store();
        // Both tables created; zero messages initially.
        assert_eq!(store.message_count().unwrap(), 0);
        // Offset starts as None.
        assert_eq!(store.get_offset().unwrap(), None);
    }

    #[test]
    fn test_insert_and_count() {
        let (store, _dir) = make_store();
        let msg = serde_json::json!({
            "chat": { "id": 100, "title": "Test Chat" },
            "from": { "id": 1, "username": "tester", "first_name": "Test" },
            "date": 1_000_001_i64, "text": "hello"
        });
        store.insert_message(1, &msg).unwrap();
        assert_eq!(store.message_count().unwrap(), 1);
    }

    #[test]
    fn test_insert_deduplication() {
        let (store, _dir) = make_store();
        let msg = serde_json::json!({
            "chat": { "id": 100 }, "from": { "id": 1 },
            "date": 1000_i64, "text": "dup"
        });
        // First insert.
        store.insert_message(42, &msg).unwrap();
        // Second insert with the same update_id — silently ignored.
        store.insert_message(42, &msg).unwrap();
        // Only one row.
        assert_eq!(store.message_count().unwrap(), 1);
    }

    #[test]
    fn test_offset_round_trip() {
        let (store, _dir) = make_store();
        assert_eq!(store.get_offset().unwrap(), None);
        store.set_offset(42).unwrap();
        assert_eq!(store.get_offset().unwrap(), Some(42));
        store.set_offset(100).unwrap();
        assert_eq!(store.get_offset().unwrap(), Some(100));
    }

    #[test]
    fn test_time_range_empty() {
        let (store, _dir) = make_store();
        assert_eq!(store.time_range().unwrap(), None);
    }

    #[test]
    fn test_time_range_with_messages() {
        let (store, _dir) = make_store();
        store
            .insert_message(
                1,
                &serde_json::json!({
                    "chat": { "id": 1 }, "from": { "id": 1 },
                    "date": 1000_i64, "text": "a"
                }),
            )
            .unwrap();
        store
            .insert_message(
                2,
                &serde_json::json!({
                    "chat": { "id": 1 }, "from": { "id": 1 },
                    "date": 2000_i64, "text": "b"
                }),
            )
            .unwrap();
        assert_eq!(store.time_range().unwrap(), Some((1000, 2000)));
    }

    #[test]
    fn test_config_missing_token_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        let dir = TempDir::new().unwrap();
        let result = TelegramConfig::from_env_and_yaml(None, dir.path());
        assert!(
            matches!(result, Err(crate::error::SdlcError::TelegramTokenMissing)),
            "expected TelegramTokenMissing error"
        );
    }

    #[test]
    fn test_config_from_env() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "env_token_123");
        let dir = TempDir::new().unwrap();
        let cfg = TelegramConfig::from_env_and_yaml(None, dir.path()).unwrap();
        assert_eq!(cfg.bot_token, "env_token_123");
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
    }

    #[test]
    fn test_config_env_overrides_yaml() {
        let _lock = ENV_MUTEX.lock().unwrap();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "env_wins");
        let yaml = crate::config::TelegramConfigYaml {
            bot_token: Some("yaml_token".to_string()),
            ..Default::default()
        };
        let dir = TempDir::new().unwrap();
        let cfg = TelegramConfig::from_env_and_yaml(Some(&yaml), dir.path()).unwrap();
        assert_eq!(cfg.bot_token, "env_wins");
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
    }
}
