use std::path::{Path, PathBuf};

/// Resolve the SDLC root directory.
///
/// Priority:
/// 1. `--root` flag / `SDLC_ROOT` env var (passed in as `explicit`)
/// 2. Nearest ancestor directory (including CWD) that contains a `.sdlc/` subdirectory
/// 3. Current working directory (fallback when no `.sdlc/` found)
pub fn resolve_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }

    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    find_sdlc_root(&cwd).unwrap_or(cwd)
}

/// Walk from `start` upward through ancestor directories, returning the first
/// directory that contains a `.sdlc/` subdirectory, or `None` if not found.
fn find_sdlc_root(start: &Path) -> Option<PathBuf> {
    let mut current = start;
    loop {
        if current.join(".sdlc").is_dir() {
            return Some(current.to_path_buf());
        }
        match current.parent() {
            Some(parent) => current = parent,
            None => return None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn explicit_root_wins() {
        let dir = TempDir::new().unwrap();
        let result = resolve_root(Some(dir.path()));
        assert_eq!(result, dir.path());
    }

    #[test]
    fn sdlc_dir_in_current_dir() {
        let dir = TempDir::new().unwrap();
        fs::create_dir(dir.path().join(".sdlc")).unwrap();
        let result = find_sdlc_root(dir.path());
        assert_eq!(result, Some(dir.path().to_path_buf()));
    }

    #[test]
    fn sdlc_dir_in_grandparent() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join(".sdlc")).unwrap();
        let child = root.path().join("a").join("b");
        fs::create_dir_all(&child).unwrap();
        let result = find_sdlc_root(&child);
        assert_eq!(result, Some(root.path().to_path_buf()));
    }

    #[test]
    fn no_sdlc_dir_returns_none() {
        let dir = TempDir::new().unwrap();
        // No .sdlc/ anywhere — find_sdlc_root should return None.
        let result = find_sdlc_root(dir.path());
        assert_eq!(result, None);
    }
}
