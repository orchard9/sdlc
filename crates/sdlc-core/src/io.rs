use crate::error::Result;
use std::io::Write;
use std::path::Path;
use tempfile::NamedTempFile;

/// Atomically write `data` to `path` using a tempfile in the same directory.
/// Prevents partial writes from corrupting state files.
pub fn atomic_write(path: &Path, data: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let dir = path.parent().unwrap_or(Path::new("."));
    let mut tmp = NamedTempFile::new_in(dir)?;
    tmp.write_all(data)?;
    tmp.persist(path).map_err(|e| e.error)?;
    Ok(())
}

/// Create a directory and all parents, idempotent.
pub fn ensure_dir(path: &Path) -> Result<()> {
    std::fs::create_dir_all(path)?;
    Ok(())
}

/// Write a file only if it does not already exist. Returns true if written.
pub fn write_if_missing(path: &Path, data: &[u8]) -> Result<bool> {
    if path.exists() {
        return Ok(false);
    }
    atomic_write(path, data)?;
    Ok(true)
}

/// Replace content between `start_marker` and `end_marker` (inclusive) in a file.
///
/// Replaces everything from the first character of `start_marker` through the last
/// character of `end_marker` with `replacement`. Returns `true` if both markers were
/// found and the file was updated, `false` if the markers were not found (file unchanged).
pub fn replace_between_markers(
    path: &Path,
    start_marker: &str,
    end_marker: &str,
    replacement: &str,
) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }
    let content = std::fs::read_to_string(path)?;
    let Some(start_pos) = content.find(start_marker) else {
        return Ok(false);
    };
    let search_from = start_pos + start_marker.len();
    let Some(end_offset) = content[search_from..].find(end_marker) else {
        return Ok(false);
    };
    let end_pos = search_from + end_offset + end_marker.len();

    let mut updated = String::with_capacity(content.len());
    updated.push_str(&content[..start_pos]);
    updated.push_str(replacement);
    updated.push_str(&content[end_pos..]);

    atomic_write(path, updated.as_bytes())?;
    Ok(true)
}

/// Append text to a file, creating it if it doesn't exist.
pub fn append_text(path: &Path, text: &str) -> Result<()> {
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    f.write_all(text.as_bytes())?;
    Ok(())
}

/// Add `entry` to `root/.gitignore` if it isn't already present.
///
/// Checks for an exact line match. Appends with a leading newline separator
/// if the file doesn't already end with one.
pub fn ensure_gitignore_entry(root: &Path, entry: &str) -> Result<()> {
    let gitignore = root.join(".gitignore");
    let existing = if gitignore.exists() {
        std::fs::read_to_string(&gitignore)?
    } else {
        String::new()
    };
    // Exact line match — avoids false positives from substring checks.
    if existing.lines().any(|l| l == entry) {
        return Ok(());
    }
    let sep = if existing.is_empty() || existing.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    use std::io::Write as _;
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&gitignore)?;
    writeln!(f, "{sep}{entry}")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn atomic_write_creates_file() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("test.yaml");
        atomic_write(&path, b"hello: world").unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "hello: world");
    }

    #[test]
    fn atomic_write_creates_parents() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("a/b/c/test.yaml");
        atomic_write(&path, b"data").unwrap();
        assert!(path.exists());
    }

    #[test]
    fn ensure_gitignore_entry_adds_when_missing() {
        let dir = TempDir::new().unwrap();
        ensure_gitignore_entry(dir.path(), ".sdlc/secrets/envs/staging.meta.yaml").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains(".sdlc/secrets/envs/staging.meta.yaml"));
    }

    #[test]
    fn ensure_gitignore_entry_idempotent() {
        let dir = TempDir::new().unwrap();
        ensure_gitignore_entry(dir.path(), ".sdlc/secrets/envs/prod.meta.yaml").unwrap();
        ensure_gitignore_entry(dir.path(), ".sdlc/secrets/envs/prod.meta.yaml").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert_eq!(
            content
                .lines()
                .filter(|l| *l == ".sdlc/secrets/envs/prod.meta.yaml")
                .count(),
            1
        );
    }

    #[test]
    fn ensure_gitignore_entry_appends_to_existing() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join(".gitignore"), "node_modules\n").unwrap();
        ensure_gitignore_entry(dir.path(), ".sdlc/secrets/envs/staging.meta.yaml").unwrap();
        let content = std::fs::read_to_string(dir.path().join(".gitignore")).unwrap();
        assert!(content.contains("node_modules"));
        assert!(content.contains(".sdlc/secrets/envs/staging.meta.yaml"));
    }

    #[test]
    fn write_if_missing_skips_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("existing.txt");
        std::fs::write(&path, b"original").unwrap();
        let written = write_if_missing(&path, b"new").unwrap();
        assert!(!written);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
    }
}
