use super::SdlcTool;
use sdlc_core::{feature::Feature, task::complete_task};
use std::path::Path;

pub struct CompleteTaskTool;

impl SdlcTool for CompleteTaskTool {
    fn name(&self) -> &str {
        "sdlc_complete_task"
    }

    fn description(&self) -> &str {
        "Mark a task as completed on a feature"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug"
                },
                "task_id": {
                    "type": "string",
                    "description": "Task ID (e.g. T1, T2)"
                }
            },
            "required": ["slug", "task_id"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let task_id = args["task_id"]
            .as_str()
            .ok_or_else(|| "missing required argument: task_id".to_string())?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;
        complete_task(&mut feature.tasks, task_id).map_err(|e| e.to_string())?;
        feature.save(root).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "status": "completed"
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sdlc_core::{
        config::Config, feature::Feature, state::State, task::add_task, types::TaskStatus,
    };
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
    fn complete_task_marks_done() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        let mut feat = Feature::create(dir.path(), "my-feat", "My Feature").unwrap();
        add_task(&mut feat.tasks, "Write tests");
        feat.save(dir.path()).unwrap();

        let tool = CompleteTaskTool;
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "task_id": "T1"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["status"], "completed");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        assert_eq!(loaded.tasks[0].status, TaskStatus::Completed);
    }

    #[test]
    fn complete_task_not_found_errors() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = CompleteTaskTool;
        let err = tool
            .call(
                serde_json::json!({"slug": "my-feat", "task_id": "T99"}),
                dir.path(),
            )
            .unwrap_err();
        assert!(!err.is_empty());
    }
}
