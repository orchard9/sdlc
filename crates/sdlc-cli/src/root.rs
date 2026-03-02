use std::path::{Path, PathBuf};

/// Resolve the SDLC root directory.
///
/// Priority:
/// 1. `--root` flag / `SDLC_ROOT` env var (passed in as `explicit`)
/// 2. Current working directory
pub fn resolve_root(explicit: Option<&Path>) -> PathBuf {
    if let Some(p) = explicit {
        return p.to_path_buf();
    }

    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
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
    fn falls_back_to_cwd() {
        // No explicit root — should return current_dir (or "." fallback).
        let result = resolve_root(None);
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        assert_eq!(result, cwd);
    }
}
