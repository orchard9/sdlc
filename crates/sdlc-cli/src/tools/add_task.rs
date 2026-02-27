use super::SdlcTool;
use sdlc_core::{feature::Feature, task::add_task};
use std::path::Path;

pub struct AddTaskTool;

impl SdlcTool for AddTaskTool {
    fn name(&self) -> &str {
        "sdlc_add_task"
    }

    fn description(&self) -> &str {
        "Add a task to a feature's task list"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug"
                },
                "title": {
                    "type": "string",
                    "description": "Task title"
                }
            },
            "required": ["slug", "title"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let title = args["title"]
            .as_str()
            .ok_or_else(|| "missing required argument: title".to_string())?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;
        let task_id = add_task(&mut feature.tasks, title);
        feature.save(root).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "task_id": task_id,
            "title": title
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

    #[test]
    fn add_task_creates_task() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = AddTaskTool;
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "title": "Write tests"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["task_id"], "T1");
        assert_eq!(result["title"], "Write tests");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        assert_eq!(loaded.tasks.len(), 1);
        assert_eq!(loaded.tasks[0].title, "Write tests");
    }

    #[test]
    fn add_task_sequential_ids() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = AddTaskTool;
        tool.call(
            serde_json::json!({"slug": "my-feat", "title": "First"}),
            dir.path(),
        )
        .unwrap();
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "title": "Second"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["task_id"], "T2");
    }
}
