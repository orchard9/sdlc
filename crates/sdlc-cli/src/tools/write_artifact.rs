use super::SdlcTool;
use sdlc_core::{
    classifier::try_auto_transition, feature::Feature, io::atomic_write, paths, types::ArtifactType,
};
use std::path::Path;
use std::str::FromStr;

pub struct WriteArtifactTool;

impl SdlcTool for WriteArtifactTool {
    fn name(&self) -> &str {
        "sdlc_write_artifact"
    }

    fn description(&self) -> &str {
        "Write content to a feature artifact and mark it as draft"
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
                "content": {
                    "type": "string",
                    "description": "Markdown content for the artifact"
                }
            },
            "required": ["slug", "artifact_type", "content"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let artifact_type_str = args["artifact_type"]
            .as_str()
            .ok_or_else(|| "missing required argument: artifact_type".to_string())?;
        let content = args["content"]
            .as_str()
            .ok_or_else(|| "missing required argument: content".to_string())?;

        let artifact_type = ArtifactType::from_str(artifact_type_str).map_err(|e| e.to_string())?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;

        let artifact_path = paths::artifact_path(root, slug, artifact_type.filename());
        atomic_write(&artifact_path, content.as_bytes()).map_err(|e| e.to_string())?;

        feature
            .mark_artifact_draft(artifact_type)
            .map_err(|e| e.to_string())?;
        feature.save(root).map_err(|e| e.to_string())?;

        let transitioned_to = try_auto_transition(root, slug);

        let mut result = serde_json::json!({
            "path": artifact_path.to_string_lossy(),
            "status": "draft"
        });
        if let Some(phase) = transitioned_to {
            result["transitioned_to"] = serde_json::Value::String(phase);
        }
        Ok(result)
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

    #[test]
    fn write_artifact_creates_file_and_marks_draft() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = WriteArtifactTool;
        let result = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "spec",
                    "content": "# Spec\n\nContent here."
                }),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["status"], "draft");
        let path = result["path"].as_str().unwrap();
        assert!(std::path::Path::new(path).exists());

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        let art = loaded.artifact(ArtifactType::Spec).unwrap();
        assert_eq!(art.status, sdlc_core::types::ArtifactStatus::Draft);
    }

    #[test]
    fn write_artifact_invalid_type_errors() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = WriteArtifactTool;
        let err = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "artifact_type": "nonexistent",
                    "content": "content"
                }),
                dir.path(),
            )
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
