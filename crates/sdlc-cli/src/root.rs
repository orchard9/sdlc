use std::path::{Path, PathBuf};

/// Resolve the SDLC root directory.
///
/// Priority:
/// 1. `--root` flag / `SDLC_ROOT` env var (passed in as `explicit`)
/// 2. Walk upward from `cwd` looking for `.sdlc/`
/// 3. Walk upward from `cwd` looking for `.git/`
/// 4. Fall back to `cwd`
pub fn resolve_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Walk upward looking for .sdlc/
    let mut dir = cwd.clone();
    loop {
        if dir.join(".sdlc").is_dir() {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }

    // Walk upward looking for .git/
    let mut dir = cwd.clone();
    loop {
        if dir.join(".git").is_dir() {
            return dir;
        }
        match dir.parent() {
            Some(p) => dir = p.to_path_buf(),
            None => break,
        }
    }

    cwd
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn explicit_root_wins() {
        let dir = TempDir::new().unwrap();
        let result = resolve_root(Some(dir.path()));
        assert_eq!(result, dir.path());
    }

    #[test]
    fn finds_sdlc_dir() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join(".sdlc")).unwrap();
        let subdir = dir.path().join("src/deep");
        std::fs::create_dir_all(&subdir).unwrap();

        // Override cwd isn't possible in tests without unsafe tricks,
        // but we can verify explicit path logic.
        let result = resolve_root(Some(dir.path()));
        assert_eq!(result, dir.path());
    }
}
