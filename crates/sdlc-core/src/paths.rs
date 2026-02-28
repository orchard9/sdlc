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
pub const INVESTIGATIONS_DIR: &str = ".sdlc/investigations";

pub const CONFIG_FILE: &str = ".sdlc/config.yaml";
pub const STATE_FILE: &str = ".sdlc/state.yaml";
pub const GUIDANCE_MD: &str = ".sdlc/guidance.md";

pub const AI_LOOKUP_DIR: &str = ".ai";
pub const AI_LOOKUP_INDEX: &str = ".ai/index.md";

pub const CLAUDE_DIR: &str = ".claude";
pub const CLAUDE_COMMANDS_DIR: &str = ".claude/commands";
pub const GEMINI_DIR: &str = ".gemini";
pub const GEMINI_COMMANDS_DIR: &str = ".gemini/commands";
pub const OPENCODE_DIR: &str = ".opencode";
pub const OPENCODE_COMMANDS_DIR: &str = ".opencode/command";
pub const AGENTS_DIR: &str = ".agents";
pub const AGENTS_SKILLS_DIR: &str = ".agents/skills";

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

pub fn milestone_acceptance_test_path(root: &Path, slug: &str) -> PathBuf {
    milestone_dir(root, slug).join("acceptance_test.md")
}

pub fn milestone_uat_results_path(root: &Path, slug: &str) -> PathBuf {
    milestone_dir(root, slug).join("uat_results.md")
}

pub fn investigation_dir(root: &Path, slug: &str) -> PathBuf {
    root.join(INVESTIGATIONS_DIR).join(slug)
}

pub fn investigation_manifest(root: &Path, slug: &str) -> PathBuf {
    investigation_dir(root, slug).join(MANIFEST_FILE)
}

pub fn ponder_dir(root: &Path, slug: &str) -> PathBuf {
    root.join(ROADMAP_DIR).join(slug)
}

pub fn ponder_manifest(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join(MANIFEST_FILE)
}

pub fn ponder_team_path(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join("team.yaml")
}

pub fn ponder_sessions_dir(root: &Path, slug: &str) -> PathBuf {
    ponder_dir(root, slug).join("sessions")
}

pub fn ponder_session_path(root: &Path, slug: &str, n: u32) -> PathBuf {
    ponder_sessions_dir(root, slug).join(format!("session-{n:03}.md"))
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

pub fn guidance_md_path(root: &Path) -> PathBuf {
    root.join(GUIDANCE_MD)
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

pub fn directive_md_path(root: &Path, slug: &str) -> PathBuf {
    feature_dir(root, slug).join("directive.md")
}

pub fn claude_commands_dir(root: &Path) -> PathBuf {
    root.join(CLAUDE_COMMANDS_DIR)
}

pub fn gemini_commands_dir(root: &Path) -> PathBuf {
    root.join(GEMINI_COMMANDS_DIR)
}

pub fn opencode_commands_dir(root: &Path) -> PathBuf {
    root.join(OPENCODE_COMMANDS_DIR)
}

pub fn codex_skills_dir(root: &Path) -> PathBuf {
    root.join(AGENTS_SKILLS_DIR)
}

// ---------------------------------------------------------------------------
// User-level (home dir) path helpers
// ---------------------------------------------------------------------------

pub fn user_home() -> Result<PathBuf> {
    home::home_dir().ok_or(SdlcError::HomeNotFound)
}

pub fn user_claude_commands_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".claude").join("commands"))
}

pub fn user_gemini_commands_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".gemini").join("commands"))
}

pub fn user_opencode_commands_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".opencode").join("command"))
}

pub fn user_agents_skills_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".agents").join("skills"))
}

pub fn user_sdlc_dir() -> Result<PathBuf> {
    Ok(user_home()?.join(".sdlc"))
}

pub fn user_ui_record_path(name: &str) -> Result<PathBuf> {
    Ok(user_sdlc_dir()?.join(format!("{name}.yaml")))
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
        assert_eq!(
            opencode_commands_dir(root),
            PathBuf::from("/tmp/proj/.opencode/command")
        );
        assert_eq!(
            codex_skills_dir(root),
            PathBuf::from("/tmp/proj/.agents/skills")
        );
    }

    #[test]
    fn user_home_from_env() {
        std::env::set_var("HOME", "/tmp/fakehome");
        let home = user_home().expect("should resolve home");
        assert_eq!(home, PathBuf::from("/tmp/fakehome"));
    }

    #[test]
    fn user_level_path_helpers() {
        std::env::set_var("HOME", "/tmp/fakehome");
        assert_eq!(
            user_claude_commands_dir().unwrap(),
            PathBuf::from("/tmp/fakehome/.claude/commands")
        );
        assert_eq!(
            user_gemini_commands_dir().unwrap(),
            PathBuf::from("/tmp/fakehome/.gemini/commands")
        );
        assert_eq!(
            user_opencode_commands_dir().unwrap(),
            PathBuf::from("/tmp/fakehome/.opencode/command")
        );
        assert_eq!(
            user_agents_skills_dir().unwrap(),
            PathBuf::from("/tmp/fakehome/.agents/skills")
        );
    }
}
