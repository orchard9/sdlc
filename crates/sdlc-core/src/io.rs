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
    fn write_if_missing_skips_existing() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("existing.txt");
        std::fs::write(&path, b"original").unwrap();
        let written = write_if_missing(&path, b"new").unwrap();
        assert!(!written);
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "original");
    }
}
