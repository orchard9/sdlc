//! Telegram Bot API integration.
//!
//! ## Digest (cron email)
//! - `DigestConfig` — loaded from `.sdlc/config.yaml` under the `telegram:` key
//! - `DigestRunner::new(config).run(dry_run, root)` — full pipeline
//! - `DigestSummary` — result of the digest build step (formatting methods)
//!
//! ## Long-polling
//! - `TelegramConfig::from_env_and_yaml` — resolved config
//! - `get_me` — fetch bot identity
//! - `poll_loop` — long-poll loop writing to `MessageStore`
//! - `MessageStore` — SQLite-backed message storage

pub mod digest;
pub mod mailer;
pub mod poll;
pub mod runner;
pub mod types;

pub use poll::{get_me, poll_loop, BotUser, MessageStore, TelegramConfig};
pub use runner::DigestRunner;
pub use types::{DigestConfig, DigestRunResult, DigestSummary, ResendConfig};
