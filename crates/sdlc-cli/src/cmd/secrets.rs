use crate::output::print_table;
use clap::Subcommand;
use sdlc_core::secrets;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Subcommand tree
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum SecretsSubcommand {
    /// Manage authorized decryption keys
    Keys {
        #[command(subcommand)]
        subcommand: SecretsKeysSubcommand,
    },
    /// Manage encrypted environment files
    Env {
        #[command(subcommand)]
        subcommand: SecretsEnvSubcommand,
    },
}

#[derive(Subcommand)]
pub enum SecretsKeysSubcommand {
    /// List authorized keys
    List,
    /// Add an authorized key (SSH public key or native age key)
    Add {
        /// Display name for this key
        #[arg(long)]
        name: String,
        /// Public key string (ssh-ed25519 ..., ssh-rsa ..., or age1...)
        #[arg(long)]
        key: String,
    },
    /// Remove an authorized key by name
    Remove {
        /// Key name to remove
        name: String,
    },
    /// Re-encrypt all env files with the current key list
    ///
    /// Run this after adding or removing keys to keep access in sync.
    Rekey {
        /// Private key for decryption (default: ~/.ssh/id_ed25519 or ~/.ssh/id_rsa)
        #[arg(long, env = "SDLC_AGE_IDENTITY")]
        identity: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
pub enum SecretsEnvSubcommand {
    /// List encrypted env files
    List,
    /// List key names in an env (no decryption required)
    Names {
        /// Environment name (e.g. production, staging)
        env: String,
    },
    /// Decrypt an env file and print KEY=VALUE pairs to stdout
    Export {
        /// Environment name
        env: String,
        /// Private key for decryption (default: ~/.ssh/id_ed25519 or ~/.ssh/id_rsa)
        #[arg(long, env = "SDLC_AGE_IDENTITY")]
        identity: Option<PathBuf>,
    },
    /// Set KEY=VALUE pairs in an env (creates the env if it doesn't exist)
    Set {
        /// Environment name
        env: String,
        /// KEY=VALUE pairs to set
        #[arg(value_parser = parse_kv)]
        pairs: Vec<(String, String)>,
        /// Private key for decryption when updating an existing env
        #[arg(long, env = "SDLC_AGE_IDENTITY")]
        identity: Option<PathBuf>,
    },
    /// Delete an env file and its metadata sidecar, or remove specific keys
    Delete {
        /// Environment name
        env: String,
        /// Key names to remove (omit to delete the entire env file)
        keys: Vec<String>,
        /// Private key for decryption when removing specific keys
        #[arg(long, env = "SDLC_AGE_IDENTITY")]
        identity: Option<PathBuf>,
    },
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn parse_kv(s: &str) -> Result<(String, String), String> {
    match s.split_once('=') {
        Some((k, _)) if k.trim().is_empty() => Err(format!("key cannot be empty in: {s}")),
        Some((k, v)) => Ok((k.trim().to_string(), v.to_string())),
        None => Err(format!("expected KEY=VALUE, got: {s}")),
    }
}

fn resolve_identity(explicit: Option<PathBuf>) -> anyhow::Result<PathBuf> {
    if let Some(p) = explicit {
        return Ok(p);
    }
    secrets::default_identity().ok_or_else(|| {
        anyhow::anyhow!(
            "no identity key found\n\
             Set SDLC_AGE_IDENTITY or use --identity <path>\n\
             (tried ~/.ssh/id_ed25519 and ~/.ssh/id_rsa)"
        )
    })
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcommand: SecretsSubcommand, _json: bool) -> anyhow::Result<()> {
    match subcommand {
        SecretsSubcommand::Keys { subcommand } => run_keys(root, subcommand),
        SecretsSubcommand::Env { subcommand } => run_env(root, subcommand),
    }
}

// ---------------------------------------------------------------------------
// Keys subcommands
// ---------------------------------------------------------------------------

fn run_keys(root: &Path, subcommand: SecretsKeysSubcommand) -> anyhow::Result<()> {
    match subcommand {
        SecretsKeysSubcommand::List => {
            let keys = secrets::list_keys(root)?;
            if keys.is_empty() {
                println!("no keys configured");
                println!(
                    "\nAdd your key:  sdlc secrets keys add --name <name> --key \"$(cat ~/.ssh/id_ed25519.pub)\""
                );
                return Ok(());
            }
            print_table(
                &["NAME", "TYPE", "ID", "ADDED"],
                keys.iter()
                    .map(|k| {
                        vec![
                            k.name.clone(),
                            k.key_type.to_string(),
                            k.short_id(),
                            k.added_at.format("%Y-%m-%d").to_string(),
                        ]
                    })
                    .collect(),
            );
            Ok(())
        }

        SecretsKeysSubcommand::Add { name, key } => {
            let key_type = secrets::KeyType::infer(&key);
            secrets::add_key(root, &name, key_type, &key)?;
            println!("added key '{name}'");
            let envs = secrets::list_envs(root)?;
            if !envs.is_empty() {
                println!(
                    "\nExisting env files need rekeying to include this key:\n  sdlc secrets keys rekey"
                );
            }
            Ok(())
        }

        SecretsKeysSubcommand::Remove { name } => {
            secrets::remove_key(root, &name)?;
            println!("removed key '{name}'");
            let envs = secrets::list_envs(root)?;
            if !envs.is_empty() {
                println!(
                    "\nRekey to revoke access from existing env files:\n  sdlc secrets keys rekey"
                );
            }
            Ok(())
        }

        SecretsKeysSubcommand::Rekey { identity } => {
            let id = resolve_identity(identity)?;
            let rekeyed = secrets::rekey(root, &id)?;
            if rekeyed.is_empty() {
                println!("no env files to rekey");
            } else {
                for env in &rekeyed {
                    println!("rekeyed: {env}");
                }
                println!("\n{} env(s) rekeyed", rekeyed.len());
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Env subcommands
// ---------------------------------------------------------------------------

fn run_env(root: &Path, subcommand: SecretsEnvSubcommand) -> anyhow::Result<()> {
    match subcommand {
        SecretsEnvSubcommand::List => {
            let envs = secrets::list_envs(root)?;
            if envs.is_empty() {
                println!("no environments configured");
                println!("\nCreate one:  sdlc secrets env set production KEY=value");
                return Ok(());
            }
            print_table(
                &["ENV", "KEYS", "UPDATED"],
                envs.iter()
                    .map(|e| {
                        vec![
                            e.env.clone(),
                            e.key_names.len().to_string(),
                            e.updated_at.format("%Y-%m-%d").to_string(),
                        ]
                    })
                    .collect(),
            );
            Ok(())
        }

        SecretsEnvSubcommand::Names { env } => {
            let meta = secrets::load_env_meta(root, &env)?;
            if meta.key_names.is_empty() {
                println!("(no key names recorded for '{env}')");
            } else {
                for name in &meta.key_names {
                    println!("{name}");
                }
            }
            Ok(())
        }

        SecretsEnvSubcommand::Export { env, identity } => {
            let id = resolve_identity(identity)?;
            let content = secrets::export_env(root, &env, &id)?;
            // Print without trailing newline modification — let the shell eval handle it.
            print!("{content}");
            Ok(())
        }

        SecretsEnvSubcommand::Set {
            env,
            pairs,
            identity,
        } => {
            if pairs.is_empty() {
                anyhow::bail!("no KEY=VALUE pairs provided");
            }
            // Only need identity if the env file already exists (decrypt + merge).
            let env_exists = sdlc_core::paths::secrets_env_path(root, &env).exists();
            let id_path = if env_exists {
                Some(resolve_identity(identity)?)
            } else {
                None
            };
            secrets::set_env_pairs(root, &env, &pairs, id_path.as_deref())?;
            println!("updated {} key(s) in '{env}'", pairs.len());
            Ok(())
        }

        SecretsEnvSubcommand::Delete {
            env,
            keys,
            identity,
        } => {
            if keys.is_empty() {
                secrets::delete_env(root, &env)?;
                println!("deleted env '{env}'");
            } else {
                let id = resolve_identity(identity)?;
                secrets::unset_env_keys(root, &env, &keys, &id)?;
                println!("removed {} key(s) from '{env}'", keys.len());
            }
            Ok(())
        }
    }
}
