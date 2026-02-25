use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// GateKind
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GateKind {
    Shell { command: String },
    Human { prompt: String },
    StepBack { questions: Vec<String> },
}

// ---------------------------------------------------------------------------
// GateDefinition
// ---------------------------------------------------------------------------

/// A verification gate that runs after an agent completes an action.
///
/// `PartialEq` compares all fields including `timeout_seconds`. If
/// `GateDefinition` is used as a map key or dedup key in the future,
/// consider comparing only `name` + `gate_type` to avoid false
/// mismatches from differing timeout values.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GateDefinition {
    pub name: String,
    pub gate_type: GateKind,
    #[serde(default = "default_auto")]
    pub auto: bool,
    /// Maximum number of retries after the first attempt.
    /// `0` means one attempt total, `2` means up to three attempts.
    /// See `GateResult.attempt` for the 1-indexed attempt counter.
    #[serde(default)]
    pub max_retries: u32,
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u32,
}

fn default_auto() -> bool {
    true
}

fn default_timeout() -> u32 {
    60
}

// ---------------------------------------------------------------------------
// GateResult
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GateResult {
    pub gate_name: String,
    pub passed: bool,
    pub output: String,
    /// 1-indexed attempt number. The first attempt is `1`, the second is `2`,
    /// etc. This pairs with `GateDefinition.max_retries` which is 0-indexed:
    /// `max_retries=0` → one attempt (`attempt=1` only),
    /// `max_retries=2` → up to three attempts (`attempt` in `1..=3`).
    pub attempt: u32,
    pub duration_ms: u64,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_gate_roundtrip() {
        let gate = GateDefinition {
            name: "build".to_string(),
            gate_type: GateKind::Shell {
                command: "npm run build".to_string(),
            },
            auto: true,
            max_retries: 2,
            timeout_seconds: 120,
        };
        let yaml = serde_yaml::to_string(&gate).unwrap();
        assert!(yaml.contains("type: shell"));
        assert!(yaml.contains("npm run build"));
        let parsed: GateDefinition = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, gate);
    }

    #[test]
    fn human_gate_roundtrip() {
        let gate = GateDefinition {
            name: "code-review".to_string(),
            gate_type: GateKind::Human {
                prompt: "Review the implementation".to_string(),
            },
            auto: false,
            max_retries: 0,
            timeout_seconds: 0,
        };
        let yaml = serde_yaml::to_string(&gate).unwrap();
        assert!(yaml.contains("type: human"));
        let parsed: GateDefinition = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, gate);
    }

    #[test]
    fn gate_defaults() {
        let yaml = "name: lint\ngate_type:\n  type: shell\n  command: npm run lint\n";
        let gate: GateDefinition = serde_yaml::from_str(yaml).unwrap();
        assert!(gate.auto);
        assert_eq!(gate.max_retries, 0);
        assert_eq!(gate.timeout_seconds, 60);
    }

    #[test]
    fn gate_result_json_roundtrip() {
        let result = GateResult {
            gate_name: "build".to_string(),
            passed: true,
            output: "Build successful".to_string(),
            attempt: 1,
            duration_ms: 3400,
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: GateResult = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.gate_name, "build");
        assert!(parsed.passed);
        assert_eq!(parsed.attempt, 1);
        assert_eq!(parsed.duration_ms, 3400);
    }

    #[test]
    fn gate_kind_json_tagged() {
        let shell = GateKind::Shell {
            command: "cargo test".to_string(),
        };
        let json = serde_json::to_string(&shell).unwrap();
        assert!(json.contains("\"type\":\"shell\""));

        let human = GateKind::Human {
            prompt: "Check output".to_string(),
        };
        let json = serde_json::to_string(&human).unwrap();
        assert!(json.contains("\"type\":\"human\""));
    }

    #[test]
    fn gate_definition_rejects_unknown_fields() {
        let yaml =
            "name: lint\ngate_type:\n  type: shell\n  command: npm run lint\ntimout_seconds: 30\n";
        let result = serde_yaml::from_str::<GateDefinition>(yaml);
        assert!(result.is_err(), "typo in field name should be rejected");
    }

    #[test]
    fn step_back_gate_yaml_roundtrip() {
        let gate = GateDefinition {
            name: "step-back-review".to_string(),
            gate_type: GateKind::StepBack {
                questions: vec![
                    "Does this change align with the spec?".to_string(),
                    "Are there any security implications?".to_string(),
                    "Will this break existing tests?".to_string(),
                ],
            },
            auto: false,
            max_retries: 0,
            timeout_seconds: 0,
        };
        let yaml = serde_yaml::to_string(&gate).unwrap();
        assert!(yaml.contains("type: step_back"));
        assert!(yaml.contains("Does this change align with the spec?"));
        let parsed: GateDefinition = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed, gate);
    }

    #[test]
    fn step_back_gate_json_roundtrip() {
        let gate = GateDefinition {
            name: "design-review".to_string(),
            gate_type: GateKind::StepBack {
                questions: vec![
                    "Is this the simplest solution?".to_string(),
                    "Have we considered alternatives?".to_string(),
                ],
            },
            auto: false,
            max_retries: 0,
            timeout_seconds: 0,
        };
        let json = serde_json::to_string(&gate).unwrap();
        assert!(json.contains("\"type\":\"step_back\""));
        let parsed: GateDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, gate);
    }

    #[test]
    fn step_back_gate_kind_json_tagged() {
        let sb = GateKind::StepBack {
            questions: vec!["Q1?".to_string()],
        };
        let json = serde_json::to_string(&sb).unwrap();
        assert!(json.contains("\"type\":\"step_back\""));
        assert!(json.contains("Q1?"));
    }

    #[test]
    fn gates_config_map_roundtrip() {
        use std::collections::HashMap;

        let mut gates: HashMap<String, Vec<GateDefinition>> = HashMap::new();
        gates.insert(
            "implement_task".to_string(),
            vec![
                GateDefinition {
                    name: "build".to_string(),
                    gate_type: GateKind::Shell {
                        command: "npm run build".to_string(),
                    },
                    auto: true,
                    max_retries: 2,
                    timeout_seconds: 120,
                },
                GateDefinition {
                    name: "test".to_string(),
                    gate_type: GateKind::Shell {
                        command: "npm test".to_string(),
                    },
                    auto: true,
                    max_retries: 1,
                    timeout_seconds: 300,
                },
            ],
        );

        let yaml = serde_yaml::to_string(&gates).unwrap();
        let parsed: HashMap<String, Vec<GateDefinition>> = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed["implement_task"].len(), 2);
        assert_eq!(parsed["implement_task"][0].name, "build");
        assert_eq!(parsed["implement_task"][1].name, "test");
    }
}
