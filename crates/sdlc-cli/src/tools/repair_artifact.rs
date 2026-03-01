use super::SdlcTool;
use sdlc_core::{io::atomic_write, paths};
use std::path::Path;

/// Valid status values that serde will accept for ArtifactStatus.
const VALID_STATUSES: &[&str] = &[
    "missing",
    "draft",
    "approved",
    "rejected",
    "needs_fix",
    "passed",
    "failed",
    "waived",
];

pub struct RepairArtifactTool;

impl SdlcTool for RepairArtifactTool {
    fn name(&self) -> &str {
        "sdlc_repair_artifact"
    }

    fn description(&self) -> &str {
        "Repair a feature with corrupt artifact status in its manifest YAML. \
        Unlike other artifact tools, this operates on raw YAML and bypasses \
        Feature::load(), so it works even when the feature cannot be deserialized. \
        Use this when the API returns a 500 error like: \
        'artifacts[N].status: unknown variant `X`, expected one of ...'. \
        After repairing, call sdlc_get_directive to re-enter the normal flow."
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug (e.g. 'my-feature')"
                },
                "artifact_type": {
                    "type": "string",
                    "description": "Artifact type to repair: spec, design, tasks, qa_plan, review, audit, qa_results"
                },
                "set_status": {
                    "type": "string",
                    "description": "Status to set. Defaults to 'missing' (safest reset — triggers the full flow again). \
                    Valid values: missing, draft, approved, rejected, needs_fix, passed, failed, waived"
                }
            },
            "required": ["slug", "artifact_type"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let artifact_type_str = args["artifact_type"]
            .as_str()
            .ok_or_else(|| "missing required argument: artifact_type".to_string())?;
        let target_status = args["set_status"].as_str().unwrap_or("missing");

        if !VALID_STATUSES.contains(&target_status) {
            return Err(format!(
                "invalid set_status '{}'. Valid values: {}",
                target_status,
                VALID_STATUSES.join(", ")
            ));
        }

        let manifest_path = paths::feature_manifest(root, slug);
        if !manifest_path.exists() {
            return Err(format!("feature '{}' not found", slug));
        }

        let raw = std::fs::read_to_string(&manifest_path)
            .map_err(|e| format!("failed to read manifest: {e}"))?;

        let mut doc: serde_yaml::Value =
            serde_yaml::from_str(&raw).map_err(|e| format!("manifest is not valid YAML: {e}"))?;

        let artifacts = doc
            .get_mut("artifacts")
            .and_then(|v| v.as_sequence_mut())
            .ok_or_else(|| "manifest has no 'artifacts' sequence".to_string())?;

        let mut previous_raw_status: Option<String> = None;
        let mut repaired = false;

        for artifact in artifacts.iter_mut() {
            let type_matches = artifact
                .get("artifact_type")
                .and_then(|v| v.as_str())
                .map(|t| t == artifact_type_str)
                .unwrap_or(false);

            if type_matches {
                previous_raw_status = artifact
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if let Some(map) = artifact.as_mapping_mut() {
                    map.insert(
                        serde_yaml::Value::String("status".to_string()),
                        serde_yaml::Value::String(target_status.to_string()),
                    );
                }
                repaired = true;
                break;
            }
        }

        if !repaired {
            return Err(format!(
                "artifact_type '{}' not found in feature '{}'",
                artifact_type_str, slug
            ));
        }

        let repaired_yaml =
            serde_yaml::to_string(&doc).map_err(|e| format!("failed to serialize YAML: {e}"))?;

        atomic_write(&manifest_path, repaired_yaml.as_bytes())
            .map_err(|e| format!("failed to write manifest: {e}"))?;

        Ok(serde_json::json!({
            "slug": slug,
            "artifact_type": artifact_type_str,
            "previous_raw_status": previous_raw_status,
            "new_status": target_status,
            "hint": format!(
                "Feature manifest repaired. Run `sdlc artifact draft {} {}` to re-enter the normal flow, or call sdlc_get_directive.",
                slug, artifact_type_str
            )
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::{config::Config, feature::Feature, state::State};
    use tempfile::TempDir;

    fn setup(dir: &TempDir) {
        std::fs::create_dir_all(dir.path().join(".sdlc/features")).unwrap();
        let config = Config::new("test");
        std::fs::write(
            dir.path().join(".sdlc/config.yaml"),
            serde_yaml::to_string(&config).unwrap(),
        )
        .unwrap();
        let state = State::new("test");
        std::fs::write(
            dir.path().join(".sdlc/state.yaml"),
            serde_yaml::to_string(&state).unwrap(),
        )
        .unwrap();
    }

    /// Inject an unrecognized status value directly into the raw YAML,
    /// simulating data written by a newer binary version.
    fn inject_corrupt_status(dir: &TempDir, slug: &str, artifact_type: &str, bad_status: &str) {
        let manifest_path = paths::feature_manifest(dir.path(), slug);
        let raw = std::fs::read_to_string(&manifest_path).unwrap();
        let mut doc: serde_yaml::Value = serde_yaml::from_str(&raw).unwrap();
        let artifacts = doc
            .get_mut("artifacts")
            .and_then(|v| v.as_sequence_mut())
            .unwrap();
        for a in artifacts.iter_mut() {
            if a.get("artifact_type")
                .and_then(|v| v.as_str())
                .map(|t| t == artifact_type)
                .unwrap_or(false)
            {
                if let Some(map) = a.as_mapping_mut() {
                    map.insert(
                        serde_yaml::Value::String("status".to_string()),
                        serde_yaml::Value::String(bad_status.to_string()),
                    );
                }
            }
        }
        let out = serde_yaml::to_string(&doc).unwrap();
        std::fs::write(&manifest_path, out).unwrap();
    }

    #[test]
    fn repairs_corrupt_status() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();
        inject_corrupt_status(&dir, "my-feat", "spec", "unknown_future_variant");

        // Feature::load must fail — that's the scenario we're fixing
        assert!(Feature::load(dir.path(), "my-feat").is_err());

        let tool = RepairArtifactTool;
        let result = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "spec",
                    "set_status": "missing"
                }),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["new_status"], "missing");
        assert_eq!(result["previous_raw_status"], "unknown_future_variant");

        // Feature must now load cleanly
        Feature::load(dir.path(), "my-feat").expect("feature should load after repair");
    }

    #[test]
    fn defaults_to_missing_when_set_status_omitted() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();
        inject_corrupt_status(&dir, "my-feat", "spec", "bad_value");

        let tool = RepairArtifactTool;
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "artifact_type": "spec"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["new_status"], "missing");
    }

    #[test]
    fn rejects_invalid_target_status() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = RepairArtifactTool;
        let err = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "spec",
                    "set_status": "totally_wrong"
                }),
                dir.path(),
            )
            .unwrap_err();

        assert!(err.contains("invalid set_status"));
    }

    #[test]
    fn errors_on_unknown_artifact_type() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = RepairArtifactTool;
        let err = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "nonexistent_type"
                }),
                dir.path(),
            )
            .unwrap_err();

        assert!(err.contains("not found"));
    }
}
