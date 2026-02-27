use super::SdlcTool;
use sdlc_core::{
    classifier::{Classifier, EvalContext},
    config::Config,
    feature::Feature,
    rules::default_rules,
    state::State,
};
use std::path::Path;

pub struct GetDirectiveTool;

impl SdlcTool for GetDirectiveTool {
    fn name(&self) -> &str {
        "sdlc_get_directive"
    }

    fn description(&self) -> &str {
        "Get the next directive for a feature — what the agent should do next"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug"
                }
            },
            "required": ["slug"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;

        let config = Config::load(root).map_err(|e| e.to_string())?;
        let state = State::load(root).map_err(|e| e.to_string())?;
        let feature = Feature::load(root, slug).map_err(|e| e.to_string())?;

        let classifier = Classifier::new(default_rules());
        let ctx = EvalContext {
            feature: &feature,
            state: &state,
            config: &config,
            root,
        };
        let classification = classifier.classify(&ctx);

        serde_json::to_value(&classification).map_err(|e| e.to_string())
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
        let config_path = dir.path().join(".sdlc/config.yaml");
        std::fs::write(config_path, serde_yaml::to_string(&config).unwrap()).unwrap();
        let state = State::new("test");
        let state_path = dir.path().join(".sdlc/state.yaml");
        std::fs::write(state_path, serde_yaml::to_string(&state).unwrap()).unwrap();
    }

    #[test]
    fn get_directive_returns_classification() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = GetDirectiveTool;
        let result = tool
            .call(serde_json::json!({"slug": "my-feat"}), dir.path())
            .unwrap();

        assert_eq!(result["feature"], "my-feat");
        assert!(result["action"].is_string());
    }

    #[test]
    fn get_directive_missing_slug_errors() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let tool = GetDirectiveTool;
        let err = tool.call(serde_json::json!({}), dir.path()).unwrap_err();
        assert!(err.contains("missing required argument: slug"));
    }

    #[test]
    fn get_directive_unknown_slug_errors() {
        let dir = TempDir::new().unwrap();
        setup(&dir);

        let tool = GetDirectiveTool;
        let err = tool
            .call(serde_json::json!({"slug": "does-not-exist"}), dir.path())
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
