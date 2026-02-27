use super::SdlcTool;
use sdlc_core::{feature::Feature, types::ArtifactType};
use std::path::Path;
use std::str::FromStr;

pub struct ApproveArtifactTool;

impl SdlcTool for ApproveArtifactTool {
    fn name(&self) -> &str {
        "sdlc_approve_artifact"
    }

    fn description(&self) -> &str {
        "Approve a feature artifact, advancing it through the state machine"
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

        let artifact_type = ArtifactType::from_str(artifact_type_str).map_err(|e| e.to_string())?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;
        feature
            .approve_artifact(artifact_type, None)
            .map_err(|e| e.to_string())?;
        feature.save(root).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "artifact_type": artifact_type_str,
            "status": "approved"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::{config::Config, feature::Feature, io::atomic_write, paths, state::State};
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
    fn approve_artifact_marks_approved() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let mut feat = Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        // First mark as draft
        let art_path = paths::artifact_path(dir.path(), "my-feat", ArtifactType::Spec.filename());
        atomic_write(&art_path, b"# Spec").unwrap();
        feat.mark_artifact_draft(ArtifactType::Spec).unwrap();
        feat.save(dir.path()).unwrap();

        let tool = ApproveArtifactTool;
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "artifact_type": "spec"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["status"], "approved");
        assert_eq!(result["artifact_type"], "spec");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        assert!(loaded.artifact(ArtifactType::Spec).unwrap().is_approved());
    }
}
