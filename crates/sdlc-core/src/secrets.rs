//! Encrypted secrets management using the `age` binary.
//!
//! Layout:
//!   .sdlc/secrets/
//!     keys.yaml              — AGE recipients (public keys, safe to commit)
//!     envs/
//!       production.age       — encrypted env file (safe to commit)
//!       production.meta.yaml — key names sidecar  (safe to commit, no values)
//!       staging.age
//!       staging.meta.yaml

use crate::error::{Result, SdlcError};
use crate::io;
use crate::paths;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyType {
    Ssh,
    Age,
}

impl std::fmt::Display for KeyType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyType::Ssh => write!(f, "ssh"),
            KeyType::Age => write!(f, "age"),
        }
    }
}

impl std::str::FromStr for KeyType {
    type Err = SdlcError;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "ssh" => Ok(KeyType::Ssh),
            "age" => Ok(KeyType::Age),
            _ => Err(SdlcError::InvalidSecretKeyType(s.to_string())),
        }
    }
}

impl KeyType {
    /// Infer key type from the public key string.
    /// Native age keys start with `age1`; everything else is treated as SSH.
    pub fn infer(public_key: &str) -> Self {
        if public_key.trim().starts_with("age1") {
            KeyType::Age
        } else {
            KeyType::Ssh
        }
    }
}

/// An authorized AGE recipient (SSH public key or native age key).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsKey {
    pub name: String,
    #[serde(rename = "type")]
    pub key_type: KeyType,
    pub public_key: String,
    pub added_at: DateTime<Utc>,
}

impl SecretsKey {
    /// Display-friendly short identifier — last 8 chars of the key material.
    pub fn short_id(&self) -> String {
        let key = self.public_key.trim();
        // "ssh-ed25519 BASE64 comment" — use the base64 field; "age1..." — use the whole key.
        let material = key.split_whitespace().nth(1).unwrap_or(key);
        if material.len() >= 8 {
            format!("\u{2026}{}", &material[material.len() - 8..])
        } else {
            material.to_string()
        }
    }
}

/// Authorized recipients stored in `keys.yaml`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SecretsConfig {
    #[serde(default)]
    pub keys: Vec<SecretsKey>,
}

/// Metadata stored alongside each encrypted env file.
/// Contains only key names (not values) — safe to commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretsEnvMeta {
    pub env: String,
    #[serde(default)]
    pub key_names: Vec<String>,
    pub updated_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// Config loading / saving
// ---------------------------------------------------------------------------

pub fn load_config(root: &Path) -> Result<SecretsConfig> {
    let path = paths::secrets_keys_path(root);
    if !path.exists() {
        return Ok(SecretsConfig::default());
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&content)?)
}

pub fn save_config(root: &Path, config: &SecretsConfig) -> Result<()> {
    let path = paths::secrets_keys_path(root);
    io::ensure_dir(&paths::secrets_dir(root))?;
    let content = serde_yaml::to_string(config)?;
    io::atomic_write(&path, content.as_bytes())
}

// ---------------------------------------------------------------------------
// Key management
// ---------------------------------------------------------------------------

pub fn list_keys(root: &Path) -> Result<Vec<SecretsKey>> {
    Ok(load_config(root)?.keys)
}

pub fn add_key(root: &Path, name: &str, key_type: KeyType, public_key: &str) -> Result<()> {
    let mut config = load_config(root)?;
    if config.keys.iter().any(|k| k.name == name) {
        return Err(SdlcError::SecretKeyExists(name.to_string()));
    }
    config.keys.push(SecretsKey {
        name: name.to_string(),
        key_type,
        public_key: public_key.trim().to_string(),
        added_at: Utc::now(),
    });
    save_config(root, &config)
}

pub fn remove_key(root: &Path, name: &str) -> Result<()> {
    let mut config = load_config(root)?;
    let before = config.keys.len();
    config.keys.retain(|k| k.name != name);
    if config.keys.len() == before {
        return Err(SdlcError::SecretKeyNotFound(name.to_string()));
    }
    save_config(root, &config)
}

// ---------------------------------------------------------------------------
// Env metadata
// ---------------------------------------------------------------------------

pub fn list_envs(root: &Path) -> Result<Vec<SecretsEnvMeta>> {
    let dir = paths::secrets_envs_dir(root);
    if !dir.exists() {
        return Ok(vec![]);
    }
    let mut envs = Vec::new();
    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.ends_with(".age") {
            let env_name = name_str.trim_end_matches(".age").to_string();
            let meta = load_env_meta(root, &env_name).unwrap_or_else(|_| SecretsEnvMeta {
                env: env_name.clone(),
                key_names: vec![],
                updated_at: Utc::now(),
            });
            envs.push(meta);
        }
    }
    envs.sort_by(|a, b| a.env.cmp(&b.env));
    Ok(envs)
}

pub fn load_env_meta(root: &Path, env_name: &str) -> Result<SecretsEnvMeta> {
    let path = paths::secrets_env_meta_path(root, env_name);
    if !path.exists() {
        // Fall back to the .age file's modification time so the timestamp is
        // stable across polls instead of changing to Utc::now() on every call.
        let age_path = paths::secrets_env_path(root, env_name);
        let updated_at = age_path
            .metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .map(DateTime::<Utc>::from)
            .unwrap_or_else(Utc::now);
        return Ok(SecretsEnvMeta {
            env: env_name.to_string(),
            key_names: vec![],
            updated_at,
        });
    }
    let content = std::fs::read_to_string(&path)?;
    Ok(serde_yaml::from_str(&content)?)
}

fn save_env_meta(root: &Path, meta: &SecretsEnvMeta) -> Result<()> {
    let path = paths::secrets_env_meta_path(root, &meta.env);
    let content = serde_yaml::to_string(meta)?;
    io::atomic_write(&path, content.as_bytes())?;
    // Keep the meta sidecar out of git — it records key names which may leak
    // information about what secrets exist. Add the specific path so only
    // this env's sidecar is ignored (not the .age ciphertext, which is safe to commit).
    let gitignore_entry = format!(".sdlc/secrets/envs/{}.meta.yaml", meta.env);
    io::ensure_gitignore_entry(root, &gitignore_entry)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// AGE binary and identity resolution
// ---------------------------------------------------------------------------

fn age_bin() -> Result<PathBuf> {
    which::which("age").map_err(|_| SdlcError::AgeNotInstalled)
}

/// Resolve the default identity path (private key for decryption).
/// Tries `~/.ssh/id_ed25519`, then `~/.ssh/id_rsa`.
pub fn default_identity() -> Option<PathBuf> {
    let home = home::home_dir()?;
    let candidates = [
        home.join(".ssh").join("id_ed25519"),
        home.join(".ssh").join("id_rsa"),
    ];
    candidates.into_iter().find(|p| p.exists())
}

// ---------------------------------------------------------------------------
// Encryption / decryption
// ---------------------------------------------------------------------------

/// Decrypt an env file and return the KEY=VALUE content.
pub fn export_env(root: &Path, env_name: &str, identity: &Path) -> Result<String> {
    let env_path = paths::secrets_env_path(root, env_name);
    if !env_path.exists() {
        return Err(SdlcError::SecretEnvNotFound(env_name.to_string()));
    }
    let age_bin = age_bin()?;
    let output = std::process::Command::new(&age_bin)
        .args([
            "--decrypt",
            "--identity",
            identity.to_str().unwrap_or(""),
            env_path.to_str().unwrap_or(""),
        ])
        .output()
        .map_err(|e| SdlcError::AgeDecryptFailed(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SdlcError::AgeDecryptFailed(stderr.trim().to_string()));
    }
    String::from_utf8(output.stdout).map_err(|e| SdlcError::AgeDecryptFailed(e.to_string()))
}

/// Encrypt `content` (KEY=VALUE text) to all current recipients and write the env file.
/// Also writes the `.meta.yaml` sidecar with key names (no values).
pub fn write_env(root: &Path, env_name: &str, content: &str, keys: &[SecretsKey]) -> Result<()> {
    if keys.is_empty() {
        return Err(SdlcError::AgeEncryptFailed(
            "no recipients configured — add a key with `sdlc secrets keys add`".to_string(),
        ));
    }
    let age_bin = age_bin()?;

    // Write recipient public keys to a temporary file accepted by `age -R`.
    let recipients_content = keys
        .iter()
        .map(|k| k.public_key.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let tmp =
        tempfile::NamedTempFile::new().map_err(|e| SdlcError::AgeEncryptFailed(e.to_string()))?;
    std::fs::write(tmp.path(), recipients_content.as_bytes())
        .map_err(|e| SdlcError::AgeEncryptFailed(e.to_string()))?;

    let env_path = paths::secrets_env_path(root, env_name);
    io::ensure_dir(&paths::secrets_envs_dir(root))?;

    let mut cmd = std::process::Command::new(&age_bin);
    cmd.args([
        "--encrypt",
        "--recipients-file",
        tmp.path().to_str().unwrap_or(""),
        "--output",
        env_path.to_str().unwrap_or(""),
    ]);
    cmd.stdin(std::process::Stdio::piped());
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());

    let mut child = cmd
        .spawn()
        .map_err(|e| SdlcError::AgeEncryptFailed(e.to_string()))?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write as _;
        stdin
            .write_all(content.as_bytes())
            .map_err(|e| SdlcError::AgeEncryptFailed(e.to_string()))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| SdlcError::AgeEncryptFailed(e.to_string()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SdlcError::AgeEncryptFailed(stderr.trim().to_string()));
    }

    // Update the sidecar with key names (not values).
    save_env_meta(
        root,
        &SecretsEnvMeta {
            env: env_name.to_string(),
            key_names: parse_env_key_names(content),
            updated_at: Utc::now(),
        },
    )
}

/// Set KEY=VALUE pairs in an env, merging with any existing content.
///
/// - If the env file already exists, decrypts it (requires `identity`), merges in
///   the new pairs (new values win), and re-encrypts.
/// - If the env file does not exist, encrypts the new pairs as the initial content
///   (`identity` is not required in this case).
pub fn set_env_pairs(
    root: &Path,
    env_name: &str,
    pairs: &[(String, String)],
    identity: Option<&Path>,
) -> Result<()> {
    let config = load_config(root)?;
    let env_path = paths::secrets_env_path(root, env_name);
    let existing = if env_path.exists() {
        let id = identity.ok_or_else(|| {
            SdlcError::AgeDecryptFailed(
                "identity key is required to update an existing env — \
                 use --identity <path> or set SDLC_AGE_IDENTITY"
                    .to_string(),
            )
        })?;
        Some(export_env(root, env_name, id)?)
    } else {
        None
    };
    let merged = merge_env_pairs(existing.as_deref().unwrap_or(""), pairs);
    write_env(root, env_name, &merged, &config.keys)
}

/// Remove specific keys from an env file, re-encrypting the result.
///
/// Returns `SecretEnvKeyNotFound` if any requested key is absent.
pub fn unset_env_keys(
    root: &Path,
    env_name: &str,
    keys_to_remove: &[String],
    identity: &Path,
) -> Result<()> {
    let content = export_env(root, env_name, identity)?;
    // Validate all requested keys exist before modifying anything.
    for key in keys_to_remove {
        let prefix = format!("{key}=");
        let found = content
            .lines()
            .any(|l| !l.starts_with('#') && l.starts_with(&prefix));
        if !found {
            return Err(SdlcError::SecretEnvKeyNotFound(
                key.clone(),
                env_name.to_string(),
            ));
        }
    }
    let updated = remove_env_keys(&content, keys_to_remove);
    let config = load_config(root)?;
    write_env(root, env_name, &updated, &config.keys)
}

/// Delete an env file and its meta sidecar.
pub fn delete_env(root: &Path, env_name: &str) -> Result<()> {
    let env_path = paths::secrets_env_path(root, env_name);
    if !env_path.exists() {
        return Err(SdlcError::SecretEnvNotFound(env_name.to_string()));
    }
    std::fs::remove_file(&env_path)?;
    let meta_path = paths::secrets_env_meta_path(root, env_name);
    if meta_path.exists() {
        std::fs::remove_file(&meta_path)?;
    }
    Ok(())
}

/// Re-encrypt all env files with the current recipient list.
///
/// Required after adding or removing a key. Returns the list of rekeyed env names.
///
/// Uses a two-phase approach to avoid leaving envs in a mixed encryption state:
/// Phase 1 decrypts all envs first (fails fast before any writes if any decryption fails).
/// Phase 2 re-encrypts with the current key set only after all decryptions succeed.
pub fn rekey(root: &Path, identity: &Path) -> Result<Vec<String>> {
    let config = load_config(root)?;
    let envs = list_envs(root)?;
    // Phase 1: decrypt all — no filesystem writes yet.
    let decrypted: Vec<(String, String)> = envs
        .iter()
        .map(|meta| export_env(root, &meta.env, identity).map(|c| (meta.env.clone(), c)))
        .collect::<Result<Vec<_>>>()?;
    // Phase 2: re-encrypt all — only reached when every decryption succeeded.
    for (env_name, content) in &decrypted {
        write_env(root, env_name, content, &config.keys)?;
    }
    Ok(decrypted.into_iter().map(|(name, _)| name).collect())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Parse KEY=VALUE env content and return only the key names.
fn parse_env_key_names(content: &str) -> Vec<String> {
    content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .filter_map(|l| l.split_once('=').map(|(k, _)| k.trim().to_string()))
        .collect()
}

/// Remove specified keys from env content. Order of remaining lines is preserved.
fn remove_env_keys(existing: &str, keys_to_remove: &[String]) -> String {
    let lines: Vec<&str> = existing
        .lines()
        .filter(|line| {
            if line.starts_with('#') || line.trim().is_empty() {
                return true;
            }
            !keys_to_remove.iter().any(|k| {
                let prefix = format!("{k}=");
                line.starts_with(&prefix)
            })
        })
        .collect();
    let mut result = lines.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

/// Merge new KEY=VALUE pairs into existing env content.
/// Existing key order is preserved; new keys are appended; new values override existing.
fn merge_env_pairs(existing: &str, new_pairs: &[(String, String)]) -> String {
    let mut lines: Vec<String> = existing.lines().map(|l| l.to_string()).collect();
    for (key, value) in new_pairs {
        let prefix = format!("{key}=");
        let found = lines.iter_mut().any(|line| {
            if line.starts_with(&prefix) {
                *line = format!("{key}={value}");
                true
            } else {
                false
            }
        });
        if !found {
            lines.push(format!("{key}={value}"));
        }
    }
    let mut result = lines.join("\n");
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn parse_key_names_skips_comments_and_blank() {
        let content = "# comment\nFOO=bar\n\nBAZ=qux\n";
        assert_eq!(parse_env_key_names(content), vec!["FOO", "BAZ"]);
    }

    #[test]
    fn remove_env_keys_drops_specified() {
        let content = "# header\nFOO=1\nBAR=2\nBAZ=3\n";
        let result = remove_env_keys(content, &["BAR".to_string()]);
        assert!(result.contains("FOO=1"));
        assert!(!result.contains("BAR=2"));
        assert!(result.contains("BAZ=3"));
        assert!(result.contains("# header"));
    }

    #[test]
    fn remove_env_keys_multiple() {
        let content = "A=1\nB=2\nC=3\n";
        let result = remove_env_keys(content, &["A".to_string(), "C".to_string()]);
        assert!(!result.contains("A=1"));
        assert!(result.contains("B=2"));
        assert!(!result.contains("C=3"));
    }

    #[test]
    fn merge_adds_new_keys() {
        let existing = "A=1\nB=2\n";
        let pairs = vec![("C".to_string(), "3".to_string())];
        let result = merge_env_pairs(existing, &pairs);
        assert!(result.contains("A=1"));
        assert!(result.contains("B=2"));
        assert!(result.contains("C=3"));
    }

    #[test]
    fn merge_overrides_existing_keys() {
        let existing = "A=1\nB=2\n";
        let pairs = vec![("A".to_string(), "99".to_string())];
        let result = merge_env_pairs(existing, &pairs);
        assert!(result.contains("A=99"));
        assert!(!result.contains("A=1"));
        assert!(result.contains("B=2"));
    }

    #[test]
    fn merge_empty_existing() {
        let pairs = vec![("X".to_string(), "1".to_string())];
        let result = merge_env_pairs("", &pairs);
        assert!(result.contains("X=1"));
    }

    #[test]
    fn key_short_id_trims_ssh_key() {
        let key = SecretsKey {
            name: "jordan".into(),
            key_type: KeyType::Ssh,
            public_key: "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5ABCD1234 jordan@example.com".into(),
            added_at: Utc::now(),
        };
        let id = key.short_id();
        assert!(id.starts_with('\u{2026}'));
        assert_eq!(id.chars().count(), 9); // '…' + 8 chars
    }

    #[test]
    fn add_and_list_keys() {
        let dir = TempDir::new().unwrap();
        add_key(dir.path(), "alice", KeyType::Ssh, "ssh-ed25519 AAAA...").unwrap();
        let keys = list_keys(dir.path()).unwrap();
        assert_eq!(keys.len(), 1);
        assert_eq!(keys[0].name, "alice");
    }

    #[test]
    fn add_duplicate_key_fails() {
        let dir = TempDir::new().unwrap();
        add_key(dir.path(), "bob", KeyType::Ssh, "ssh-ed25519 AAAA...").unwrap();
        let result = add_key(dir.path(), "bob", KeyType::Ssh, "ssh-ed25519 BBBB...");
        assert!(matches!(result, Err(SdlcError::SecretKeyExists(_))));
    }

    #[test]
    fn remove_key_works() {
        let dir = TempDir::new().unwrap();
        add_key(dir.path(), "charlie", KeyType::Ssh, "ssh-ed25519 CCCC...").unwrap();
        remove_key(dir.path(), "charlie").unwrap();
        assert!(list_keys(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn remove_missing_key_fails() {
        let dir = TempDir::new().unwrap();
        let result = remove_key(dir.path(), "nobody");
        assert!(matches!(result, Err(SdlcError::SecretKeyNotFound(_))));
    }

    #[test]
    fn list_envs_empty_when_no_dir() {
        let dir = TempDir::new().unwrap();
        assert!(list_envs(dir.path()).unwrap().is_empty());
    }

    #[test]
    fn delete_env_missing_fails() {
        let dir = TempDir::new().unwrap();
        let result = delete_env(dir.path(), "production");
        assert!(matches!(result, Err(SdlcError::SecretEnvNotFound(_))));
    }

    #[test]
    fn export_env_missing_fails() {
        let dir = TempDir::new().unwrap();
        let identity = dir.path().join("id_ed25519");
        std::fs::write(&identity, b"fake-key").unwrap();
        let result = export_env(dir.path(), "production", &identity);
        assert!(matches!(result, Err(SdlcError::SecretEnvNotFound(_))));
    }
}
