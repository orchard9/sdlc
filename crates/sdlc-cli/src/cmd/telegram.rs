use anyhow::{Context, Result};
use clap::Subcommand;
use sdlc_core::config::Config;
use sdlc_core::telegram::{
    get_me, poll_loop, DigestConfig, DigestRunner, MessageStore, ResendConfig, TelegramConfig,
};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::output::print_json;

#[derive(Subcommand)]
pub enum TelegramSubcommand {
    /// Start polling the Telegram Bot API and storing incoming messages.
    ///
    /// Runs until interrupted (Ctrl-C). Messages are stored in
    /// `.sdlc/telegram/messages.db`. Requires TELEGRAM_BOT_TOKEN to be set
    /// (or `telegram.bot_token` in `.sdlc/config.yaml`).
    Poll,

    /// Show bot identity and stored message statistics.
    Status,

    /// Fetch messages from configured Telegram chats and send a digest email via Resend.
    ///
    /// Requires: TELEGRAM_BOT_TOKEN (or telegram.bot_token in config) and Resend credentials.
    /// Schedule with cron for a daily digest:
    ///   0 8 * * * sdlc telegram digest >> /var/log/sdlc-digest.log 2>&1
    Digest {
        /// Print digest to stdout without sending email.
        #[arg(long)]
        dry_run: bool,

        /// Override time window in hours (default: from config or 24).
        #[arg(long)]
        window: Option<u32>,

        /// Override chat IDs (repeatable; overrides config list).
        #[arg(long = "chat")]
        chats: Vec<String>,

        /// Emit JSON run summary to stdout.
        #[arg(long)]
        json: bool,

        /// Show verbose per-message details during processing.
        #[arg(short, long)]
        verbose: bool,
    },
}

pub fn run(root: &Path, subcommand: TelegramSubcommand) -> Result<()> {
    let config = Config::load(root).context("failed to load .sdlc/config.yaml")?;

    match subcommand {
        TelegramSubcommand::Poll => {
            let tg_config = TelegramConfig::from_env_and_yaml(config.telegram.as_ref(), root)
                .context("failed to resolve Telegram configuration")?;
            run_poll(&tg_config)
        }
        TelegramSubcommand::Status => {
            let tg_config = TelegramConfig::from_env_and_yaml(config.telegram.as_ref(), root)
                .context("failed to resolve Telegram configuration")?;
            run_status(&tg_config)
        }
        TelegramSubcommand::Digest {
            dry_run,
            window,
            chats,
            json,
            verbose,
        } => run_digest(root, &config, dry_run, window, chats, json, verbose),
    }
}

fn run_poll(config: &TelegramConfig) -> Result<()> {
    let store = MessageStore::open(&config.db_path).context("failed to open message database")?;

    let bot = get_me(config).context("failed to call getMe — check bot token")?;
    println!(
        "Bot: @{} (id: {})",
        if bot.username.is_empty() {
            bot.first_name.clone()
        } else {
            bot.username.clone()
        },
        bot.id
    );
    println!("Polling for messages... (Ctrl-C to stop)");

    // Graceful shutdown flag
    let shutdown = Arc::new(AtomicBool::new(false));
    let shutdown_clone = Arc::clone(&shutdown);

    ctrlc::set_handler(move || {
        shutdown_clone.store(true, Ordering::Relaxed);
    })
    .context("failed to register Ctrl-C handler")?;

    poll_loop(config, &store, Arc::clone(&shutdown)).context("polling error")?;

    let count = store
        .message_count()
        .context("failed to read message count")?;
    println!("Stopped. Total messages stored: {}", count);

    Ok(())
}

fn run_status(config: &TelegramConfig) -> Result<()> {
    let bot = get_me(config).context("failed to call getMe — check bot token")?;

    let username_display = if bot.username.is_empty() {
        bot.first_name.clone()
    } else {
        format!("@{}", bot.username)
    };

    println!("Bot:      {} (id: {})", username_display, bot.id);

    // Open the DB if it exists — not an error if it's absent (no messages yet)
    if config.db_path.exists() {
        let store =
            MessageStore::open(&config.db_path).context("failed to open message database")?;

        let count = store
            .message_count()
            .context("failed to read message count")?;

        let offset = store
            .get_offset()
            .context("failed to read polling offset")?;

        println!("Messages: {}", count);

        match store.time_range().context("failed to read time range")? {
            Some((oldest, newest)) => {
                let oldest_dt = chrono::DateTime::from_timestamp(oldest, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| format!("unix:{}", oldest));
                let newest_dt = chrono::DateTime::from_timestamp(newest, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                    .unwrap_or_else(|| format!("unix:{}", newest));
                println!("Oldest:   {}", oldest_dt);
                println!("Newest:   {}", newest_dt);
            }
            None => {
                println!("Oldest:   (no messages)");
                println!("Newest:   (no messages)");
            }
        }

        match offset {
            Some(n) => println!("Offset:   {} (last update_id + 1)", n),
            None => println!("Offset:   (never polled)"),
        }
    } else {
        println!("Messages: 0 (database not yet created — run `sdlc telegram poll` to start)");
    }

    Ok(())
}

fn run_digest(
    root: &Path,
    config: &Config,
    dry_run: bool,
    window_override: Option<u32>,
    chat_override: Vec<String>,
    json: bool,
    verbose: bool,
) -> Result<()> {
    let digest_config = build_digest_config(root, config, window_override, chat_override)?;
    let runner = DigestRunner::new(digest_config);

    let result = runner
        .run(dry_run, Some(root))
        .context("telegram digest run failed")?;

    if json {
        let summary = &result.summary;
        print_json(&serde_json::json!({
            "dry_run": result.dry_run,
            "total_messages": summary.total_messages,
            "chat_count": summary.chats.len(),
            "period_start": summary.period_start.to_rfc3339(),
            "period_end": summary.period_end.to_rfc3339(),
            "sent_to": result.sent_to,
        }))?;
    } else if dry_run {
        print!("{}", result.summary.format_plain_text());
        eprintln!(
            "Dry run: {} messages across {} chats. Email not sent.",
            result.summary.total_messages,
            result.summary.chats.len()
        );
    } else {
        if verbose {
            print!("{}", result.summary.format_plain_text());
        }
        if result.sent_to.is_empty() {
            println!("No messages in the configured time window. Nothing to send.");
        } else {
            println!(
                "Digest sent: {} messages across {} chats → {}",
                result.summary.total_messages,
                result.summary.chats.len(),
                result.sent_to.join(", ")
            );
        }
    }

    Ok(())
}

/// Build a `DigestConfig` from the loaded `Config`, applying env var lookups and CLI overrides.
///
/// All secrets are read from environment variables (injected via `sdlc secrets env export telegram`):
///   TELEGRAM_BOT_TOKEN  — bot API token
///   RESEND_API_KEY      — Resend API key (re_*)
///   RESEND_FROM         — from address (must be a verified Resend sender domain)
///   RESEND_TO           — recipient address(es), comma-separated
///   TELEGRAM_CHAT_IDS   — optional comma-separated chat IDs (overrides config.yaml list)
fn build_digest_config(
    root: &Path,
    config: &Config,
    window_override: Option<u32>,
    chat_override: Vec<String>,
) -> Result<DigestConfig> {
    let tg = config.telegram.as_ref();

    let bot_token = std::env::var("TELEGRAM_BOT_TOKEN")
        .ok()
        .or_else(|| tg.and_then(|t| t.bot_token.clone()))
        .context(
            "Telegram bot token not configured. Set TELEGRAM_BOT_TOKEN env var \
             (sdlc secrets env export telegram) or add telegram.bot_token to .sdlc/config.yaml",
        )?;

    let resend_api_key = std::env::var("RESEND_API_KEY").context(
        "Resend API key not configured. Set RESEND_API_KEY env var \
             (sdlc secrets env export telegram)",
    )?;

    let resend_from = std::env::var("RESEND_FROM").context(
        "Resend from address not configured. Set RESEND_FROM env var \
             (sdlc secrets env export telegram). Must be a verified Resend sender domain.",
    )?;

    let resend_to: Vec<String> = std::env::var("RESEND_TO")
        .context(
            "Resend recipients not configured. Set RESEND_TO env var (comma-separated) \
             (sdlc secrets env export telegram)",
        )?
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // Chat IDs: CLI flag > TELEGRAM_CHAT_IDS env var > config.yaml
    let chat_ids = if !chat_override.is_empty() {
        chat_override
    } else if let Ok(env_ids) = std::env::var("TELEGRAM_CHAT_IDS") {
        env_ids
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        tg.map(|t| t.chat_ids.clone()).unwrap_or_default()
    };

    let window_hours = window_override
        .or_else(|| tg.and_then(|t| t.window_hours))
        .unwrap_or(24);

    let subject_prefix = tg
        .and_then(|t| t.subject_prefix.clone())
        .unwrap_or_else(|| "[sdlc Digest]".to_string());

    let max_messages_per_chat = tg.and_then(|t| t.max_messages_per_chat).unwrap_or(100);

    let db_path = tg
        .and_then(|t| t.db_path.as_deref())
        .map(|p| root.join(p))
        .unwrap_or_else(|| root.join(".sdlc").join("telegram").join("messages.db"));

    Ok(DigestConfig {
        bot_token,
        chat_ids,
        resend: ResendConfig {
            api_key: resend_api_key,
            from: resend_from,
            to: resend_to,
        },
        window_hours,
        subject_prefix,
        max_messages_per_chat,
        db_path,
    })
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::config::Config;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Serialize tests that mutate env vars to prevent races.
    static ENV_MUTEX: Mutex<()> = Mutex::new(());

    fn set_required_env_vars() {
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
        std::env::set_var("RESEND_API_KEY", "re_test_key");
        std::env::set_var("RESEND_FROM", "from@test.com");
        std::env::set_var("RESEND_TO", "to@test.com");
    }

    fn clear_env_vars() {
        std::env::remove_var("TELEGRAM_BOT_TOKEN");
        std::env::remove_var("RESEND_API_KEY");
        std::env::remove_var("RESEND_FROM");
        std::env::remove_var("RESEND_TO");
        std::env::remove_var("TELEGRAM_CHAT_IDS");
    }

    #[test]
    fn missing_bot_token_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let err = build_digest_config(dir.path(), &config, None, vec![]).unwrap_err();
        assert!(
            err.to_string().contains("TELEGRAM_BOT_TOKEN"),
            "error: {err}"
        );
    }

    #[test]
    fn missing_resend_api_key_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let err = build_digest_config(dir.path(), &config, None, vec![]).unwrap_err();
        assert!(err.to_string().contains("RESEND_API_KEY"), "error: {err}");
    }

    #[test]
    fn missing_resend_from_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
        std::env::set_var("RESEND_API_KEY", "re_test_key");
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let err = build_digest_config(dir.path(), &config, None, vec![]).unwrap_err();
        assert!(err.to_string().contains("RESEND_FROM"), "error: {err}");
    }

    #[test]
    fn missing_resend_to_returns_error() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
        std::env::set_var("RESEND_API_KEY", "re_test_key");
        std::env::set_var("RESEND_FROM", "from@test.com");
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let err = build_digest_config(dir.path(), &config, None, vec![]).unwrap_err();
        assert!(err.to_string().contains("RESEND_TO"), "error: {err}");
    }

    #[test]
    fn happy_path_returns_correct_fields() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, None, vec![]).unwrap();
        assert_eq!(cfg.bot_token, "test-token");
        assert_eq!(cfg.resend.api_key, "re_test_key");
        assert_eq!(cfg.resend.from, "from@test.com");
        assert_eq!(cfg.resend.to, vec!["to@test.com".to_string()]);
        clear_env_vars();
    }

    #[test]
    fn telegram_chat_ids_comma_split() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        std::env::set_var("TELEGRAM_CHAT_IDS", "-100111,-100222,-100333");
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, None, vec![]).unwrap();
        assert_eq!(
            cfg.chat_ids,
            vec!["-100111", "-100222", "-100333"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        clear_env_vars();
    }

    #[test]
    fn chat_override_beats_env_var() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        std::env::set_var("TELEGRAM_CHAT_IDS", "-100111");
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let override_ids = vec!["-100999".to_string()];
        let cfg = build_digest_config(dir.path(), &config, None, override_ids).unwrap();
        assert_eq!(cfg.chat_ids, vec!["-100999".to_string()]);
        clear_env_vars();
    }

    #[test]
    fn window_override_beats_config() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, Some(48), vec![]).unwrap();
        assert_eq!(cfg.window_hours, 48);
        clear_env_vars();
    }

    #[test]
    fn db_path_defaults_to_sdlc_telegram() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, None, vec![]).unwrap();
        let expected = dir
            .path()
            .join(".sdlc")
            .join("telegram")
            .join("messages.db");
        assert_eq!(cfg.db_path, expected);
        clear_env_vars();
    }

    #[test]
    fn resend_to_comma_split_trims_whitespace() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        std::env::set_var("TELEGRAM_BOT_TOKEN", "test-token");
        std::env::set_var("RESEND_API_KEY", "re_key");
        std::env::set_var("RESEND_FROM", "from@test.com");
        std::env::set_var(
            "RESEND_TO",
            "alice@example.com , bob@example.com , charlie@example.com",
        );
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, None, vec![]).unwrap();
        assert_eq!(
            cfg.resend.to,
            vec![
                "alice@example.com".to_string(),
                "bob@example.com".to_string(),
                "charlie@example.com".to_string()
            ]
        );
        clear_env_vars();
    }

    #[test]
    fn window_defaults_to_24_when_no_override() {
        let _lock = ENV_MUTEX.lock().unwrap();
        clear_env_vars();
        set_required_env_vars();
        let dir = TempDir::new().unwrap();
        let config = Config::new("test");
        let cfg = build_digest_config(dir.path(), &config, None, vec![]).unwrap();
        assert_eq!(cfg.window_hours, 24);
        clear_env_vars();
    }
}
