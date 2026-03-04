use crate::output::print_table;
use clap::Subcommand;
use sdlc_core::auth_config;
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand tree
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum AuthSubcommand {
    /// Manage named tunnel-access tokens stored in .sdlc/auth.yaml
    Token {
        #[command(subcommand)]
        subcommand: AuthTokenSubcommand,
    },
}

#[derive(Subcommand)]
pub enum AuthTokenSubcommand {
    /// List all named tokens (names and creation dates; no token values)
    List,

    /// Generate and add a new named token
    Add {
        /// Unique name for this token (e.g. "jordan", "ci-bot")
        name: String,
    },

    /// Remove a named token by name
    Remove {
        /// Token name to revoke
        name: String,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcommand: AuthSubcommand, _json: bool) -> anyhow::Result<()> {
    match subcommand {
        AuthSubcommand::Token { subcommand } => run_token(root, subcommand),
    }
}

// ---------------------------------------------------------------------------
// Token subcommands
// ---------------------------------------------------------------------------

fn run_token(root: &Path, subcommand: AuthTokenSubcommand) -> anyhow::Result<()> {
    match subcommand {
        AuthTokenSubcommand::List => {
            let config = auth_config::load(root)?;
            if config.tokens.is_empty() {
                println!("no tokens configured — auth is open (anyone with the URL can access)");
                println!("\nAdd a token:  sdlc auth token add <name>");
                return Ok(());
            }
            print_table(
                &["NAME", "CREATED"],
                config
                    .tokens
                    .iter()
                    .map(|t| vec![t.name.clone(), t.created_at.clone()])
                    .collect(),
            );
            Ok(())
        }

        AuthTokenSubcommand::Add { name } => {
            let token = auth_config::add_token(root, &name)?;
            println!("token '{name}' added");
            println!("\nToken (shown once — copy it now):\n  {token}");
            println!("\nUse as: ?auth={token}  or  Authorization: Bearer {token}");
            Ok(())
        }

        AuthTokenSubcommand::Remove { name } => {
            auth_config::remove_token(root, &name)?;
            println!("token '{name}' removed");
            Ok(())
        }
    }
}
