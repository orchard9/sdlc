use crate::error::{Result, SdlcError};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Directory constants
// ---------------------------------------------------------------------------

pub const SDLC_DIR: &str = ".sdlc";
pub const FEATURES_DIR: &str = ".sdlc/features";
pub const MILESTONES_DIR: &str = ".sdlc/milestones";
pub const PATTERNS_DIR: &str = ".sdlc/patterns";
pub const AUDITS_DIR: &str = ".sdlc/audits";
pub const BRANCHES_DIR: &str = ".sdlc/branches";
pub const ARCHIVES_DIR: &str = ".sdlc/archives";
pub const ROADMAP_DIR: &str = ".sdlc/roadmap";

pub const CONFIG_FILE: &str = ".sdlc/config.yaml";
pub const STATE_FILE: &str = ".sdlc/state.yaml";

pub const AI_LOOKUP_DIR: &str = ".ai";
pub const AI_LOOKUP_INDEX: &str = ".ai/index.md";

pub const CLAUDE_DIR: &str = ".claude";
pub const CLAUDE_COMMANDS_DIR: &str = ".claude/commands";

pub const VISION_MD: &str = "VISION.md";
pub const AGENTS_MD: &str = "AGENTS.md";
pub const MANIFEST_FILE: &str = "manifest.yaml";

// ---------------------------------------------------------------------------
// Path helpers
// ---------------------------------------------------------------------------

pub fn feature_dir(root: &Path, slug: &str) -> PathBuf {
    root.join(FEATURES_DIR).join(slug)
}

pub fn feature_manifest(root: &Path, slug: &str) -> PathBuf {
    feature_dir(root, slug).join(MANIFEST_FILE)
}

pub fn milestone_dir(root: &Path, slug: &str) -> PathBuf {
    root.join(MILESTONES_DIR).join(slug)
}

pub fn milestone_manifest(root: &Path, slug: &str) -> PathBuf {
    milestone_dir(root, slug).join(MANIFEST_FILE)
}

pub fn artifact_path(root: &Path, slug: &str, filename: &str) -> PathBuf {
    feature_dir(root, slug).join(filename)
}

pub fn config_path(root: &Path) -> PathBuf {
    root.join(CONFIG_FILE)
}

pub fn state_path(root: &Path) -> PathBuf {
    root.join(STATE_FILE)
}

pub fn sdlc_dir(root: &Path) -> PathBuf {
    root.join(SDLC_DIR)
}

pub fn ai_lookup_dir(root: &Path) -> PathBuf {
    root.join(AI_LOOKUP_DIR)
}

pub fn vision_md_path(root: &Path) -> PathBuf {
    root.join(VISION_MD)
}

pub fn agents_md_path(root: &Path) -> PathBuf {
    root.join(AGENTS_MD)
}

pub fn claude_commands_dir(root: &Path) -> PathBuf {
    root.join(CLAUDE_COMMANDS_DIR)
}

// ---------------------------------------------------------------------------
// Slug validation
// ---------------------------------------------------------------------------

static SLUG_RE: OnceLock<Regex> = OnceLock::new();

fn slug_re() -> &'static Regex {
    SLUG_RE.get_or_init(|| Regex::new(r"^[a-z0-9][a-z0-9\-]*[a-z0-9]$|^[a-z0-9]$").unwrap())
}

pub fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() || slug.len() > 64 || !slug_re().is_match(slug) {
        return Err(SdlcError::InvalidSlug(slug.to_string()));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_slugs() {
        for slug in ["auth-login", "a", "my-feature-123", "x1"] {
            validate_slug(slug).unwrap_or_else(|_| panic!("expected valid: {slug}"));
        }
    }

    #[test]
    fn invalid_slugs() {
        for slug in [
            "",
            "-starts-with-dash",
            "ends-with-dash-",
            "has spaces",
            "UPPER",
            "a_b",
        ] {
            assert!(validate_slug(slug).is_err(), "expected invalid: {slug}");
        }
    }

    #[test]
    fn path_helpers() {
        let root = Path::new("/tmp/proj");
        assert_eq!(
            config_path(root),
            PathBuf::from("/tmp/proj/.sdlc/config.yaml")
        );
        assert_eq!(
            feature_manifest(root, "auth"),
            PathBuf::from("/tmp/proj/.sdlc/features/auth/manifest.yaml")
        );
        assert_eq!(
            milestone_manifest(root, "v2"),
            PathBuf::from("/tmp/proj/.sdlc/milestones/v2/manifest.yaml")
        );
    }
}
