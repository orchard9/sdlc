use crate::error::{Result, SdlcError};
use crate::paths;
use crate::types::{ArtifactType, Phase};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// ConfigWarning / WarnLevel
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigWarning {
    pub level: WarnLevel,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WarnLevel {
    Warning,
    Error,
}

// ---------------------------------------------------------------------------
// QualityConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    #[serde(default = "default_min_advance")]
    pub min_score_to_advance: u32,
    #[serde(default = "default_min_release")]
    pub min_score_to_release: u32,
    #[serde(default = "default_require_all")]
    pub require_all_lenses: bool,
}

fn default_min_advance() -> u32 {
    70
}

fn default_min_release() -> u32 {
    80
}

fn default_require_all() -> bool {
    true
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            min_score_to_advance: default_min_advance(),
            min_score_to_release: default_min_release(),
            require_all_lenses: default_require_all(),
        }
    }
}

// ---------------------------------------------------------------------------
// PhaseConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseConfig {
    #[serde(default = "default_enabled_phases")]
    pub enabled: Vec<Phase>,
    #[serde(default = "default_required_artifacts")]
    pub required_artifacts: HashMap<String, Vec<ArtifactType>>,
}

fn default_enabled_phases() -> Vec<Phase> {
    Phase::all().to_vec()
}

fn default_required_artifacts() -> HashMap<String, Vec<ArtifactType>> {
    let mut m = HashMap::new();
    m.insert("specified".to_string(), vec![ArtifactType::Spec]);
    m.insert(
        "planned".to_string(),
        vec![
            ArtifactType::Spec,
            ArtifactType::Design,
            ArtifactType::Tasks,
            ArtifactType::QaPlan,
        ],
    );
    m.insert("review".to_string(), vec![ArtifactType::Review]);
    // audit requires approved review to enter
    m.insert("audit".to_string(), vec![ArtifactType::Review]);
    // qa requires approved audit to enter
    m.insert("qa".to_string(), vec![ArtifactType::Audit]);
    // merge requires approved qa_results to enter
    m.insert("merge".to_string(), vec![ArtifactType::QaResults]);
    m
}

impl Default for PhaseConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled_phases(),
            required_artifacts: default_required_artifacts(),
        }
    }
}

impl PhaseConfig {
    pub fn is_enabled(&self, phase: Phase) -> bool {
        self.enabled.contains(&phase)
    }

    pub fn required_for(&self, phase: Phase) -> &[ArtifactType] {
        self.required_artifacts
            .get(phase.as_str())
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

// ---------------------------------------------------------------------------
// PlatformConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformArg {
    pub name: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub choices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformCommand {
    pub description: String,
    #[serde(default)]
    pub script: String,
    #[serde(default)]
    pub args: Vec<PlatformArg>,
    /// Subcommands: name → script path (e.g. "start" → ".sdlc/platform/dev-start.sh")
    #[serde(default)]
    pub subcommands: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlatformConfig {
    #[serde(default)]
    pub commands: HashMap<String, PlatformCommand>,
}

// ---------------------------------------------------------------------------
// ProjectConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
}

// ---------------------------------------------------------------------------
// Config (top-level)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_version")]
    pub version: u32,
    pub project: ProjectConfig,
    #[serde(default)]
    pub phases: PhaseConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityConfig>,
    /// Version of the `sdlc` binary that last ran `sdlc init` or `sdlc update` on this project.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sdlc_version: Option<String>,
    /// Preferred port for the app tunnel (project dev server). Persisted so the
    /// UI can pre-populate the port input across restarts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_port: Option<u16>,
}

fn default_version() -> u32 {
    1
}

impl Config {
    pub fn new(project_name: impl Into<String>) -> Self {
        Self {
            version: 1,
            project: ProjectConfig {
                name: project_name.into(),
                description: None,
            },
            phases: PhaseConfig::default(),
            platform: None,
            quality: None,
            sdlc_version: None,
            app_port: None,
        }
    }

    pub fn load(root: &Path) -> Result<Self> {
        let path = paths::config_path(root);
        if !path.exists() {
            return Err(SdlcError::NotInitialized);
        }
        let data = std::fs::read_to_string(&path)?;
        let cfg: Config = serde_yaml::from_str(&data)?;
        Ok(cfg)
    }

    pub fn save(&self, root: &Path) -> Result<()> {
        let path = paths::config_path(root);
        let data = serde_yaml::to_string(self)?;
        crate::io::atomic_write(&path, data.as_bytes())
    }

    // -----------------------------------------------------------------------
    // Validation
    // -----------------------------------------------------------------------

    pub fn validate(&self) -> Vec<ConfigWarning> {
        Vec::new()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_roundtrip() {
        let cfg = Config::new("test-project");
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        let parsed: Config = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.project.name, "test-project");
        assert_eq!(parsed.version, 1);
    }

    #[test]
    fn platform_config_roundtrip() {
        let mut cmds = HashMap::new();
        cmds.insert(
            "deploy".to_string(),
            PlatformCommand {
                description: "Deploy a service".to_string(),
                script: ".sdlc/platform/deploy.sh".to_string(),
                args: vec![PlatformArg {
                    name: "service".to_string(),
                    required: true,
                    choices: vec!["auth-service".to_string()],
                }],
                subcommands: HashMap::new(),
            },
        );
        let platform = PlatformConfig { commands: cmds };
        let yaml = serde_yaml::to_string(&platform).unwrap();
        let parsed: PlatformConfig = serde_yaml::from_str(&yaml).unwrap();
        assert!(parsed.commands.contains_key("deploy"));
        assert_eq!(parsed.commands["deploy"].args[0].name, "service");
    }

    #[test]
    fn config_without_platform_backward_compat() {
        // A config.yaml without a 'platform:' key must still deserialize
        let yaml = "version: 1\nproject:\n  name: my-project\n";
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.platform.is_none());

        // And re-serializing must NOT emit a 'platform:' key
        let out = serde_yaml::to_string(&cfg).unwrap();
        assert!(!out.contains("platform"));
    }

    #[test]
    fn required_artifacts_defaults() {
        let cfg = PhaseConfig::default();
        let spec_reqs = cfg.required_for(Phase::Specified);
        assert_eq!(spec_reqs, &[ArtifactType::Spec]);
        let planned_reqs = cfg.required_for(Phase::Planned);
        assert!(planned_reqs.contains(&ArtifactType::Design));
        assert!(planned_reqs.contains(&ArtifactType::Tasks));
    }

    #[test]
    fn validate_valid_config_no_warnings() {
        let cfg = Config::new("test-project");
        let warnings = cfg.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn quality_config_defaults() {
        let qc = QualityConfig::default();
        assert_eq!(qc.min_score_to_advance, 70);
        assert_eq!(qc.min_score_to_release, 80);
        assert!(qc.require_all_lenses);
    }

    #[test]
    fn quality_config_roundtrip() {
        let qc = QualityConfig {
            min_score_to_advance: 60,
            min_score_to_release: 90,
            require_all_lenses: false,
        };
        let yaml = serde_yaml::to_string(&qc).unwrap();
        let parsed: QualityConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.min_score_to_advance, 60);
        assert_eq!(parsed.min_score_to_release, 90);
        assert!(!parsed.require_all_lenses);
    }

    #[test]
    fn config_with_quality_roundtrip() {
        let yaml = r#"
version: 1
project:
  name: my-project
quality:
  min_score_to_advance: 75
  min_score_to_release: 85
  require_all_lenses: true
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.quality.is_some());
        let q = cfg.quality.unwrap();
        assert_eq!(q.min_score_to_advance, 75);
        assert_eq!(q.min_score_to_release, 85);
        assert!(q.require_all_lenses);
    }

    #[test]
    fn config_without_quality_backward_compat() {
        let yaml = "version: 1\nproject:\n  name: my-project\n";
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.quality.is_none());

        let out = serde_yaml::to_string(&cfg).unwrap();
        assert!(!out.contains("quality"));
    }
}
