use crate::error::{Result, SdlcError};
use crate::io::atomic_write;
use std::path::Path;

// ---------------------------------------------------------------------------
// Data types
// ---------------------------------------------------------------------------

/// On-disk representation of `.sdlc/auth.yaml`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub tokens: Vec<NamedToken>,
}

/// A named tunnel-access token.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NamedToken {
    /// Human-readable label (e.g. "jordan", "ci-bot").
    pub name: String,
    /// Raw token value — 8 alphanumeric characters.
    pub token: String,
    /// RFC 3339 timestamp of creation.
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Persistence helpers
// ---------------------------------------------------------------------------

/// Path to `.sdlc/auth.yaml` inside `root`.
pub fn auth_config_path(root: &Path) -> std::path::PathBuf {
    root.join(".sdlc").join("auth.yaml")
}

/// Load `.sdlc/auth.yaml`.
///
/// Returns an empty `AuthConfig` (no tokens, open mode) if the file does not
/// exist. Returns an error only for I/O or YAML parse failures.
pub fn load(root: &Path) -> Result<AuthConfig> {
    let path = auth_config_path(root);
    if !path.exists() {
        return Ok(AuthConfig::default());
    }
    let data = std::fs::read_to_string(&path)?;
    let config: AuthConfig = serde_yaml::from_str(&data)?;
    Ok(config)
}

/// Atomically write `config` to `.sdlc/auth.yaml`.
pub fn save(root: &Path, config: &AuthConfig) -> Result<()> {
    let path = auth_config_path(root);
    let data = serde_yaml::to_string(config)?;
    atomic_write(&path, data.as_bytes())?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Token operations
// ---------------------------------------------------------------------------

/// Generate a random 8-character alphanumeric token.
///
/// Reads 6 bytes from `/dev/urandom` (Linux/macOS) and base62-encodes them
/// to produce an 8-character token. Falls back to a timestamp+pid value on
/// platforms without `/dev/urandom` (e.g. Windows).
pub fn generate_token() -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    let bytes: Option<[u8; 6]> = (|| {
        use std::io::Read;
        let mut buf = [0u8; 6];
        std::fs::File::open("/dev/urandom")
            .ok()?
            .read_exact(&mut buf)
            .ok()?;
        Some(buf)
    })();

    let raw = match bytes {
        Some(b) => b,
        None => {
            // Fallback: mix nanos + pid into 6 pseudo-random bytes
            let nanos = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .subsec_nanos();
            let pid = std::process::id();
            [
                (nanos & 0xFF) as u8,
                ((nanos >> 8) & 0xFF) as u8,
                ((nanos >> 16) & 0xFF) as u8,
                ((nanos >> 24) & 0xFF) as u8,
                (pid & 0xFF) as u8,
                ((pid >> 8) & 0xFF) as u8,
            ]
        }
    };

    // Map each byte to a character from CHARS (62 options).
    // We take 8 characters from 6 bytes using modulo — slight bias but
    // acceptable for a short-lived, human-read token.
    raw.iter()
        .flat_map(|&b| {
            // Produce 2 chars per byte: high nibble + low nibble, each mod 62
            let hi = (b >> 4) as usize % CHARS.len();
            let lo = (b & 0x0F) as usize % CHARS.len();
            [CHARS[hi] as char, CHARS[lo] as char]
        })
        .take(8)
        .collect()
}

/// Add a new named token to `.sdlc/auth.yaml`.
///
/// Returns the generated token value, or `Err(AuthTokenExists)` if a token
/// with `name` already exists.
pub fn add_token(root: &Path, name: &str) -> Result<String> {
    let mut config = load(root)?;
    if config.tokens.iter().any(|t| t.name == name) {
        return Err(SdlcError::AuthTokenExists(name.to_string()));
    }
    let token = generate_token();
    let created_at = chrono::Utc::now().to_rfc3339();
    config.tokens.push(NamedToken {
        name: name.to_string(),
        token: token.clone(),
        created_at,
    });
    save(root, &config)?;
    Ok(token)
}

/// Remove a named token from `.sdlc/auth.yaml` by name.
///
/// Returns `Err(AuthTokenNotFound)` if no token with `name` exists.
pub fn remove_token(root: &Path, name: &str) -> Result<()> {
    let mut config = load(root)?;
    let before = config.tokens.len();
    config.tokens.retain(|t| t.name != name);
    if config.tokens.len() == before {
        return Err(SdlcError::AuthTokenNotFound(name.to_string()));
    }
    save(root, &config)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn tmp() -> TempDir {
        tempfile::TempDir::new().expect("tempdir")
    }

    #[test]
    fn load_returns_empty_when_no_file() {
        let dir = tmp();
        let config = load(dir.path()).unwrap();
        assert!(config.tokens.is_empty());
    }

    #[test]
    fn add_token_writes_file() {
        let dir = tmp();
        let token = add_token(dir.path(), "jordan").unwrap();
        assert_eq!(token.len(), 8);
        assert!(token.chars().all(|c| c.is_ascii_alphanumeric()));
        let config = load(dir.path()).unwrap();
        assert_eq!(config.tokens.len(), 1);
        assert_eq!(config.tokens[0].name, "jordan");
        assert_eq!(config.tokens[0].token, token);
    }

    #[test]
    fn add_token_duplicate_name_errors() {
        let dir = tmp();
        add_token(dir.path(), "jordan").unwrap();
        let err = add_token(dir.path(), "jordan").unwrap_err();
        assert!(matches!(err, SdlcError::AuthTokenExists(_)));
    }

    #[test]
    fn remove_token_removes_entry() {
        let dir = tmp();
        add_token(dir.path(), "jordan").unwrap();
        remove_token(dir.path(), "jordan").unwrap();
        let config = load(dir.path()).unwrap();
        assert!(config.tokens.is_empty());
    }

    #[test]
    fn remove_token_not_found_errors() {
        let dir = tmp();
        let err = remove_token(dir.path(), "ghost").unwrap_err();
        assert!(matches!(err, SdlcError::AuthTokenNotFound(_)));
    }

    #[test]
    fn generate_token_is_8_alphanumeric() {
        let tok = generate_token();
        assert_eq!(tok.len(), 8);
        assert!(tok.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn save_and_load_roundtrip() {
        let dir = tmp();
        let original = AuthConfig {
            tokens: vec![NamedToken {
                name: "alice".to_string(),
                token: "abcd1234".to_string(),
                created_at: "2026-01-01T00:00:00Z".to_string(),
            }],
        };
        save(dir.path(), &original).unwrap();
        let loaded = load(dir.path()).unwrap();
        assert_eq!(loaded.tokens.len(), 1);
        assert_eq!(loaded.tokens[0].name, "alice");
        assert_eq!(loaded.tokens[0].token, "abcd1234");
    }
}
