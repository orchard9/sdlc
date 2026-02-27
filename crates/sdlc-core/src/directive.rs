use crate::{classifier::Classification, paths, types::ActionType};
use std::path::Path;

// ---------------------------------------------------------------------------
// completion_steps
// ---------------------------------------------------------------------------

pub fn completion_steps(action: ActionType, slug: &str, task_id: Option<&str>) -> Vec<String> {
    use ActionType::*;
    match action {
        CreateSpec => vec![
            format!("sdlc artifact draft {slug} spec"),
            format!("sdlc next --for {slug} --json"),
        ],
        CreateDesign => vec![
            format!("sdlc artifact draft {slug} design"),
            format!("sdlc next --for {slug} --json"),
        ],
        CreateTasks => vec![
            format!("sdlc artifact draft {slug} tasks"),
            format!("sdlc next --for {slug} --json"),
        ],
        CreateQaPlan => vec![
            format!("sdlc artifact draft {slug} qa_plan"),
            format!("sdlc next --for {slug} --json"),
        ],
        CreateReview | FixReviewIssues => vec![
            format!("sdlc artifact draft {slug} review"),
            format!("sdlc next --for {slug} --json"),
        ],
        CreateAudit => vec![
            format!("sdlc artifact draft {slug} audit"),
            format!("sdlc next --for {slug} --json"),
        ],
        RunQa => vec![
            format!("sdlc artifact draft {slug} qa_results"),
            format!("sdlc next --for {slug} --json"),
        ],
        ImplementTask => match task_id {
            Some(id) => vec![
                format!("sdlc task complete {slug} {id}"),
                format!("sdlc next --for {slug} --json"),
            ],
            None => vec![format!("sdlc next --for {slug} --json")],
        },
        Merge => vec![format!("sdlc merge {slug}")],
        _ => vec![format!("sdlc next --for {slug} --json")],
    }
}

// ---------------------------------------------------------------------------
// build_directive
// ---------------------------------------------------------------------------

pub fn build_directive(c: &Classification, slug: &str, root: &Path) -> String {
    let mut doc = String::new();

    doc.push_str(&format!("# Directive: {slug}\n\n"));
    doc.push_str(&format!("**Action:** {}\n", c.action));
    doc.push_str(&format!("**Phase:** {}\n", c.current_phase));
    let output_display = c.output_path.as_deref().unwrap_or("—");
    doc.push_str(&format!("**Output:** {output_display}\n"));
    doc.push_str(
        "**Standard:** Steve Jobs bar — right solution over expedient. No known debt shipped.\n",
    );
    doc.push_str("**Approach:** Structural before detail. Scale: understand → plan → implement; direct fix only if trivial.\n\n");

    doc.push_str("## Task\n\n");
    doc.push_str(&c.message);
    doc.push('\n');

    if let Some(ref task_id) = c.task_id {
        doc.push('\n');
        doc.push_str(&format!("**Task ID:** {task_id}\n"));
    }

    doc.push_str("\n## Feature\n\n");
    doc.push_str(&format!("- **Slug:** {slug}\n"));
    doc.push_str(&format!("- **Title:** {}\n", c.title));
    if let Some(ref description) = c.description {
        doc.push_str(&format!("- **Description:** {description}\n"));
    }
    doc.push_str(&format!("- **Phase:** {}\n", c.current_phase));

    doc.push_str("\n## Context Files\n\n");
    doc.push_str("Read these before starting:\n");
    doc.push_str("- `VISION.md`\n");
    doc.push_str("- `AGENTS.md`\n");
    doc.push_str(&format!("- `.sdlc/features/{slug}/manifest.yaml`\n"));

    // Embed task detail for ImplementTask
    if c.action == ActionType::ImplementTask {
        if let Some(ref id) = c.task_id {
            let task_path = root.join(format!(".sdlc/features/{slug}/tasks/{id}.yaml"));
            if let Ok(content) = std::fs::read_to_string(&task_path) {
                doc.push_str("\n## Task Detail\n\n");
                doc.push_str("```yaml\n");
                doc.push_str(&content);
                doc.push_str("```\n");
            }
        }
    }

    let steps = completion_steps(c.action, slug, c.task_id.as_deref());
    doc.push_str("\n## On Completion\n\n");
    doc.push_str("Run these commands in order:\n");
    for (i, step) in steps.iter().enumerate() {
        doc.push_str(&format!("{}. `{step}`\n", i + 1));
    }

    // Embed VISION.md if present
    let vision_path = paths::vision_md_path(root);
    if let Ok(vision) = std::fs::read_to_string(&vision_path) {
        if !vision.trim().is_empty() {
            doc.push_str("\n---\n\n");
            doc.push_str("## VISION.md\n\n");
            doc.push_str(&vision);
        }
    }

    // Ensure the document ends with a newline
    if !doc.ends_with('\n') {
        doc.push('\n');
    }

    doc
}
