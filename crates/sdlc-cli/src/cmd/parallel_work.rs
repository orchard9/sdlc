use crate::output::{print_json, print_table};
use anyhow::Context;
use sdlc_core::parallel_work::{select_parallel_work_from_root, WorkItemKind};
use std::path::Path;

pub fn run(root: &Path, json: bool) -> anyhow::Result<()> {
    let items = select_parallel_work_from_root(root).context("failed to compute parallel work")?;

    if json {
        print_json(&items)?;
        return Ok(());
    }

    if items.is_empty() {
        println!("No parallel work available. All milestones are done, released, or horizon.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|item| {
            let (kind, slug, action) = match &item.kind {
                WorkItemKind::Feature { slug, next_action } => {
                    ("feature", slug.as_str(), next_action.as_str())
                }
                WorkItemKind::Uat => ("uat", item.milestone_slug.as_str(), "milestone_uat"),
            };
            vec![
                item.milestone_title.clone(),
                kind.to_string(),
                slug.to_string(),
                action.to_string(),
                item.command.clone(),
            ]
        })
        .collect();

    print_table(&["Milestone", "Type", "Slug", "Action", "Command"], rows);

    Ok(())
}
