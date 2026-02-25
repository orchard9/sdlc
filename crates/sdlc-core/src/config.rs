use crate::error::{Result, SdlcError};
use crate::gate::GateDefinition;
use crate::paths;
use crate::types::{ActionType, ArtifactType, Phase};
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
// AgentBackend
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentBackend {
    ClaudeAgentSdk {
        #[serde(default = "default_claude_model")]
        model: String,
        #[serde(default)]
        allowed_tools: Vec<String>,
        #[serde(default)]
        permission_mode: Option<String>,
        #[serde(default)]
        timeout_minutes: Option<u32>,
    },
    Xadk {
        agent_id: String,
        #[serde(default)]
        read_agents_md: bool,
    },
    Human,
}

fn default_claude_model() -> String {
    "claude-opus-4-6".to_string()
}

// ---------------------------------------------------------------------------
// AgentsConfig
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentsConfig {
    #[serde(default = "default_agent_backend")]
    pub default: AgentBackend,
    #[serde(default)]
    pub actions: HashMap<String, AgentBackend>,
}

fn default_agent_backend() -> AgentBackend {
    AgentBackend::ClaudeAgentSdk {
        model: default_claude_model(),
        allowed_tools: vec![
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "Bash".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
        ],
        permission_mode: Some("acceptEdits".to_string()),
        timeout_minutes: None,
    }
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            default: default_agent_backend(),
            actions: HashMap::new(),
        }
    }
}

impl AgentsConfig {
    pub fn backend_for(&self, action: ActionType) -> &AgentBackend {
        self.actions
            .get(action.as_str())
            .unwrap_or(&self.default)
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
    #[serde(default)]
    pub agents: AgentsConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub platform: Option<PlatformConfig>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub gates: HashMap<String, Vec<GateDefinition>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quality: Option<QualityConfig>,
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
            agents: AgentsConfig::default(),
            platform: None,
            gates: HashMap::new(),
            quality: None,
        }
    }

    pub fn gates_for(&self, action: &str) -> &[GateDefinition] {
        self.gates
            .get(action)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
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
        let mut warnings = Vec::new();

        // 1. All action keys in agents.actions must be valid ActionType strings
        for action_key in self.agents.actions.keys() {
            if !ActionType::is_valid(action_key) {
                warnings.push(ConfigWarning {
                    level: WarnLevel::Warning,
                    message: format!(
                        "unknown action '{}' in agents.actions",
                        action_key
                    ),
                });
            }
        }

        // 2. Gate commands: warn if command is empty
        for (action_key, gate_defs) in &self.gates {
            // Also check that gate action keys are valid ActionType strings
            if !ActionType::is_valid(action_key) {
                warnings.push(ConfigWarning {
                    level: WarnLevel::Warning,
                    message: format!(
                        "unknown action '{}' in gates",
                        action_key
                    ),
                });
            }

            for gate in gate_defs {
                if let crate::gate::GateKind::Shell { command } = &gate.gate_type {
                    if command.trim().is_empty() {
                        warnings.push(ConfigWarning {
                            level: WarnLevel::Warning,
                            message: format!(
                                "gate '{}' on action '{}' has an empty command",
                                gate.name, action_key
                            ),
                        });
                    }
                }

                // 4. Gate max_retries > 10 → warning
                if gate.max_retries > 10 {
                    warnings.push(ConfigWarning {
                        level: WarnLevel::Warning,
                        message: format!(
                            "gate '{}' on action '{}' has max_retries={} (>10 is unusual)",
                            gate.name, action_key, gate.max_retries
                        ),
                    });
                }
            }
        }

        // 5. Cross-model review enforcement (Step 4.2):
        //    If implement_task and create_review both use ClaudeAgentSdk with
        //    the same model, emit a warning.
        if let (Some(impl_backend), Some(review_backend)) = (
            self.agents.actions.get("implement_task"),
            self.agents.actions.get("create_review"),
        ) {
            if let (
                AgentBackend::ClaudeAgentSdk { model: impl_model, .. },
                AgentBackend::ClaudeAgentSdk { model: review_model, .. },
            ) = (impl_backend, review_backend)
            {
                if impl_model == review_model {
                    warnings.push(ConfigWarning {
                        level: WarnLevel::Warning,
                        message: format!(
                            "reviewer and implementer use the same model '{}' \
                             — consider using a different model for review",
                            impl_model
                        ),
                    });
                }
            }
        }

        warnings
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
    fn agent_backend_yaml_tagged() {
        let backend = AgentBackend::ClaudeAgentSdk {
            model: "claude-opus-4-6".to_string(),
            allowed_tools: vec!["Read".to_string()],
            permission_mode: None,
            timeout_minutes: Some(45),
        };
        let yaml = serde_yaml::to_string(&backend).unwrap();
        assert!(yaml.contains("type: claude_agent_sdk"));
        let parsed: AgentBackend = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, backend);
    }

    #[test]
    fn human_backend_roundtrip() {
        let backend = AgentBackend::Human;
        let yaml = serde_yaml::to_string(&backend).unwrap();
        assert!(yaml.contains("type: human"));
        let parsed: AgentBackend = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, backend);
    }

    #[test]
    fn xadk_backend_roundtrip() {
        let backend = AgentBackend::Xadk {
            agent_id: "design_planner".to_string(),
            read_agents_md: true,
        };
        let yaml = serde_yaml::to_string(&backend).unwrap();
        assert!(yaml.contains("type: xadk"));
        let parsed: AgentBackend = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, backend);
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
    fn config_with_gates_roundtrip() {
        use crate::gate::GateKind;

        let yaml = r#"
version: 1
project:
  name: my-project
gates:
  implement_task:
    - name: build
      gate_type:
        type: shell
        command: "npm run build"
      max_retries: 2
      timeout_seconds: 120
    - name: test
      gate_type:
        type: shell
        command: "npm test"
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(cfg.gates.len(), 1);
        let gates = cfg.gates_for("implement_task");
        assert_eq!(gates.len(), 2);
        assert_eq!(gates[0].name, "build");
        assert_eq!(gates[0].max_retries, 2);
        assert_eq!(gates[0].timeout_seconds, 120);
        assert!(matches!(gates[0].gate_type, GateKind::Shell { .. }));
        assert_eq!(gates[1].name, "test");
        // Second gate should use defaults
        assert_eq!(gates[1].max_retries, 0);
        assert_eq!(gates[1].timeout_seconds, 60);
    }

    #[test]
    fn config_without_gates_backward_compat() {
        let yaml = "version: 1\nproject:\n  name: my-project\n";
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        assert!(cfg.gates.is_empty());
        assert_eq!(cfg.gates_for("implement_task").len(), 0);
    }

    #[test]
    fn config_gates_not_serialized_when_empty() {
        let cfg = Config::new("test");
        let yaml = serde_yaml::to_string(&cfg).unwrap();
        assert!(!yaml.contains("gates"));
    }

    #[test]
    fn gates_for_unknown_action_returns_empty() {
        let cfg = Config::new("test");
        assert!(cfg.gates_for("nonexistent_action").is_empty());
    }

    #[test]
    fn validate_valid_config_no_warnings() {
        let cfg = Config::new("test-project");
        let warnings = cfg.validate();
        assert!(warnings.is_empty());
    }

    #[test]
    fn validate_unknown_action_in_agents() {
        let mut cfg = Config::new("test-project");
        cfg.agents.actions.insert(
            "bogus_action".to_string(),
            AgentBackend::Human,
        );
        let warnings = cfg.validate();
        assert!(!warnings.is_empty());
        assert!(warnings.iter().any(|w| {
            w.message.contains("unknown action 'bogus_action'")
                && w.message.contains("agents.actions")
        }));
    }

    #[test]
    fn validate_gate_max_retries_warning() {
        use crate::gate::GateKind;

        let mut cfg = Config::new("test-project");
        cfg.gates.insert(
            "implement_task".to_string(),
            vec![GateDefinition {
                name: "excessive".to_string(),
                gate_type: GateKind::Shell {
                    command: "npm test".to_string(),
                },
                auto: true,
                max_retries: 15,
                timeout_seconds: 60,
            }],
        );
        let warnings = cfg.validate();
        assert!(warnings.iter().any(|w| {
            w.message.contains("max_retries=15")
                && w.message.contains(">10 is unusual")
        }));
    }

    #[test]
    fn validate_empty_gate_command_warning() {
        use crate::gate::GateKind;

        let mut cfg = Config::new("test-project");
        cfg.gates.insert(
            "create_spec".to_string(),
            vec![GateDefinition {
                name: "bad-gate".to_string(),
                gate_type: GateKind::Shell {
                    command: "".to_string(),
                },
                auto: true,
                max_retries: 0,
                timeout_seconds: 60,
            }],
        );
        let warnings = cfg.validate();
        assert!(warnings.iter().any(|w| w.message.contains("empty command")));
    }

    #[test]
    fn validate_unknown_action_in_gates() {
        use crate::gate::GateKind;

        let mut cfg = Config::new("test-project");
        cfg.gates.insert(
            "not_a_real_action".to_string(),
            vec![GateDefinition {
                name: "check".to_string(),
                gate_type: GateKind::Shell {
                    command: "echo ok".to_string(),
                },
                auto: true,
                max_retries: 0,
                timeout_seconds: 60,
            }],
        );
        let warnings = cfg.validate();
        assert!(warnings.iter().any(|w| {
            w.message.contains("unknown action 'not_a_real_action'")
                && w.message.contains("gates")
        }));
    }

    #[test]
    fn validate_same_model_review_warning() {
        let mut cfg = Config::new("test-project");
        cfg.agents.actions.insert(
            "implement_task".to_string(),
            AgentBackend::ClaudeAgentSdk {
                model: "claude-opus-4-6".to_string(),
                allowed_tools: vec![],
                permission_mode: None,
                timeout_minutes: None,
            },
        );
        cfg.agents.actions.insert(
            "create_review".to_string(),
            AgentBackend::ClaudeAgentSdk {
                model: "claude-opus-4-6".to_string(),
                allowed_tools: vec![],
                permission_mode: None,
                timeout_minutes: None,
            },
        );
        let warnings = cfg.validate();
        assert!(warnings.iter().any(|w| {
            w.message.contains("reviewer and implementer use the same model")
        }));
    }

    #[test]
    fn validate_different_model_review_no_warning() {
        let mut cfg = Config::new("test-project");
        cfg.agents.actions.insert(
            "implement_task".to_string(),
            AgentBackend::ClaudeAgentSdk {
                model: "claude-opus-4-6".to_string(),
                allowed_tools: vec![],
                permission_mode: None,
                timeout_minutes: None,
            },
        );
        cfg.agents.actions.insert(
            "create_review".to_string(),
            AgentBackend::ClaudeAgentSdk {
                model: "claude-sonnet-4-20250514".to_string(),
                allowed_tools: vec![],
                permission_mode: None,
                timeout_minutes: None,
            },
        );
        let warnings = cfg.validate();
        assert!(!warnings.iter().any(|w| {
            w.message.contains("reviewer and implementer use the same model")
        }));
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

    #[test]
    fn config_with_step_back_gate_roundtrip() {
        use crate::gate::GateKind;

        let yaml = r#"
version: 1
project:
  name: my-project
gates:
  create_design:
    - name: step-back
      gate_type:
        type: step_back
        questions:
          - "Does this align with the spec?"
          - "Are there simpler alternatives?"
      auto: false
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        let gates = cfg.gates_for("create_design");
        assert_eq!(gates.len(), 1);
        assert!(!gates[0].auto);
        assert!(matches!(
            &gates[0].gate_type,
            GateKind::StepBack { questions } if questions.len() == 2
        ));
    }

    #[test]
    fn config_with_human_gate() {
        use crate::gate::GateKind;

        let yaml = r#"
version: 1
project:
  name: my-project
gates:
  implement_task:
    - name: review
      gate_type:
        type: human
        prompt: "Review the implementation before proceeding"
      auto: false
"#;
        let cfg: Config = serde_yaml::from_str(yaml).unwrap();
        let gates = cfg.gates_for("implement_task");
        assert_eq!(gates.len(), 1);
        assert!(!gates[0].auto);
        assert!(matches!(
            &gates[0].gate_type,
            GateKind::Human { prompt } if prompt.contains("Review")
        ));
    }
}
