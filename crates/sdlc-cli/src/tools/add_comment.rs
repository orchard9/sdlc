use super::SdlcTool;
use sdlc_core::{
    comment::{add_comment, CommentFlag, CommentTarget},
    feature::Feature,
};
use std::path::Path;

pub struct AddCommentTool;

impl SdlcTool for AddCommentTool {
    fn name(&self) -> &str {
        "sdlc_add_comment"
    }

    fn description(&self) -> &str {
        "Add a comment to a feature, optionally flagged as blocker, question, decision, or fyi"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "slug": {
                    "type": "string",
                    "description": "Feature slug"
                },
                "body": {
                    "type": "string",
                    "description": "Comment text"
                },
                "flag": {
                    "type": "string",
                    "enum": ["blocker", "question", "decision", "fyi"],
                    "description": "Optional flag for the comment"
                }
            },
            "required": ["slug", "body"]
        })
    }

    fn call(&self, args: serde_json::Value, root: &Path) -> Result<serde_json::Value, String> {
        let slug = args["slug"]
            .as_str()
            .ok_or_else(|| "missing required argument: slug".to_string())?;
        let body = args["body"]
            .as_str()
            .ok_or_else(|| "missing required argument: body".to_string())?;

        let flag = args["flag"]
            .as_str()
            .map(|f| match f {
                "blocker" => Ok(CommentFlag::Blocker),
                "question" => Ok(CommentFlag::Question),
                "decision" => Ok(CommentFlag::Decision),
                "fyi" => Ok(CommentFlag::Fyi),
                other => Err(format!("unknown flag: {other}")),
            })
            .transpose()?;

        let mut feature = Feature::load(root, slug).map_err(|e| e.to_string())?;
        let comment_id = add_comment(
            &mut feature.comments,
            &mut feature.next_comment_seq,
            body,
            flag,
            CommentTarget::Feature,
            None,
        );
        feature.save(root).map_err(|e| e.to_string())?;

        Ok(serde_json::json!({
            "comment_id": comment_id
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
    fn add_comment_creates_comment() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = AddCommentTool;
        let result = tool
            .call(
                serde_json::json!({"slug": "my-feat", "body": "Looks good"}),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["comment_id"], "C1");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        assert_eq!(loaded.comments.len(), 1);
        assert_eq!(loaded.comments[0].body, "Looks good");
    }

    #[test]
    fn add_comment_with_flag() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = AddCommentTool;
        let result = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "body": "This is blocking",
                    "flag": "blocker"
                }),
                dir.path(),
            )
            .unwrap();

        assert_eq!(result["comment_id"], "C1");

        let loaded = Feature::load(dir.path(), "my-feat").unwrap();
        assert_eq!(loaded.comments[0].flag, Some(CommentFlag::Blocker));
    }

    #[test]
    fn add_comment_invalid_flag_errors() {
        let dir = TempDir::new().unwrap();
        setup(&dir);
        Feature::create(dir.path(), "my-feat", "My Feature").unwrap();

        let tool = AddCommentTool;
        let err = tool
            .call(
                serde_json::json!({
                    "slug": "my-feat",
                    "body": "test",
                    "flag": "invalid"
                }),
                dir.path(),
            )
            .unwrap_err();
        assert!(err.contains("unknown flag"));
    }
}
