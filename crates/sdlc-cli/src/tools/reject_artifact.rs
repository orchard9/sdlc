use super::SdlcTool;
use sdlc_core::{feature::Feature, types::ArtifactType};
use std::path::Path;
use std::str::FromStr;

pub struct RejectArtifactTool;

impl SdlcTool for RejectArtifactTool {
    fn name(&self) -> &str {
        "sdlc_reject_artifact"
    }

    fn description(&self) -> &str {
        "Reject a feature artifact with a reason, causing it to be rewritten"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug"
                },
                "artifact_type": {
                    "type": "string",
                    "description": "Artifact type: spec, design, tasks, qa_plan, review, audit, qa_results"
                },
                "reason": {
                    "type": "string",
                    "description": "Reason for rejection"
                }
            },
            "required": ["slug", "artifact_type", "reason"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let artifact_type_str = args["artifact_type"]
            .as_str()
            .ok_or_else(|| "missing required argument: artifact_type".to_string())?;
        let reason = args["reason"]
            .as_str()
            .ok_or_else(|| "missing required argument: reason".to_string())?;

        let artifact_type = ArtifactType::from_str(artifact_type_str).map_err(|e| e.to_string())?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;
        feature
            .reject_artifact(artifact_type, Some(reason.to_string()))
            .map_err(|e| e.to_string())?;
        feature.save(root).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "artifact_type": artifact_type_str,
            "status": "rejected",
            "reason": reason
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::{config::Config, feature::Feature, state::State, types::ArtifactStatus};
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

    #[test]
    fn reject_artifact_sets_rejected_status() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = RejectArtifactTool;
        let result = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "spec",
                    "reason": "Needs more detail"
                }),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["status"], "rejected");
        assert_eq!(result["reason"], "Needs more detail");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        let art = loaded.artifact(ArtifactType::Spec).unwrap();
        assert_eq!(art.status, ArtifactStatus::Rejected);
    }
}
