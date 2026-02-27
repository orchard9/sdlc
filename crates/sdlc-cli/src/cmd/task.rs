use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{feature::Feature, task as task_ops};
use std::path::Path;

#[derive(Subcommand)]
pub enum TaskSubcommand {
    /// Add a task to a feature
    Add {
        slug: String,
        #[arg(required = true)]
        title: Vec<String>,
    },
    /// Start a task
    Start { slug: String, task_id: String },
    /// Complete a task
    Complete { slug: String, task_id: String },
    /// Mark a task as blocked
    Block {
        slug: String,
        task_id: String,
        #[arg(required = true)]
        reason: Vec<String>,
    },
    /// List tasks for a feature, or all tasks across every feature when no slug is given
    List { slug: Option<String> },
    /// Edit task fields (title, description, dependencies)
    Edit {
        slug: String,
        task_id: String,
        /// Update task title
        #[arg(long)]
        title: Option<String>,
        /// Update task description
        #[arg(long)]
        description: Option<String>,
        /// Set task dependencies as comma-separated IDs (e.g. T1,T2)
        #[arg(long)]
        depends: Option<String>,
    },
    /// Show full details for a single task
    Get { slug: String, task_id: String },
    /// Search tasks by title or description
    Search {
        query: String,
        /// Scope search to a single feature
        #[arg(long)]
        slug: Option<String>,
        /// Maximum results (default: 10)
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
}

pub fn run(root: &Path, subcmd: TaskSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        TaskSubcommand::Add { slug, title } => add(root, &slug, &title.join(" "), json),
        TaskSubcommand::Start { slug, task_id } => start(root, &slug, &task_id, json),
        TaskSubcommand::Complete { slug, task_id } => complete(root, &slug, &task_id, json),
        TaskSubcommand::Block {
            slug,
            task_id,
            reason,
        } => block(root, &slug, &task_id, &reason.join(" "), json),
        TaskSubcommand::List { slug } => list(root, slug.as_deref(), json),
        TaskSubcommand::Edit {
            slug,
            task_id,
            title,
            description,
            depends,
        } => edit(
            root,
            &slug,
            &task_id,
            title.as_deref(),
            description.as_deref(),
            depends.as_deref(),
            json,
        ),
        TaskSubcommand::Get { slug, task_id } => get(root, &slug, &task_id, json),
        TaskSubcommand::Search { query, slug, limit } => {
            search(root, &query, slug.as_deref(), limit, json)
        }
    }
}

fn add(root: &Path, slug: &str, title: &str, json: bool) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    let id = task_ops::add_task(&mut feature.tasks, title);
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({ "slug": slug, "task_id": id, "title": title }))?;
    } else {
        println!("Added task [{id}]: {title}");
    }
    Ok(())
}

fn start(root: &Path, slug: &str, task_id: &str, json: bool) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    task_ops::start_task(&mut feature.tasks, task_id)
        .with_context(|| format!("task '{task_id}' not found"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(
            &serde_json::json!({ "slug": slug, "task_id": task_id, "status": "in_progress" }),
        )?;
    } else {
        println!("Started task [{task_id}]");
    }
    Ok(())
}

fn complete(root: &Path, slug: &str, task_id: &str, json: bool) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    task_ops::complete_task(&mut feature.tasks, task_id)
        .with_context(|| format!("task '{task_id}' not found"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(
            &serde_json::json!({ "slug": slug, "task_id": task_id, "status": "completed" }),
        )?;
    } else {
        println!("Completed task [{task_id}]");
    }
    Ok(())
}

fn block(root: &Path, slug: &str, task_id: &str, reason: &str, json: bool) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    task_ops::block_task(&mut feature.tasks, task_id, reason)
        .with_context(|| format!("task '{task_id}' not found"))?;
    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "task_id": task_id,
            "status": "blocked",
            "reason": reason,
        }))?;
    } else {
        println!("Blocked task [{task_id}]: {reason}");
    }
    Ok(())
}

fn edit(
    root: &Path,
    slug: &str,
    task_id: &str,
    title: Option<&str>,
    description: Option<&str>,
    depends: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let mut feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    let task = feature
        .tasks
        .iter_mut()
        .find(|t| t.id == task_id)
        .with_context(|| format!("task '{task_id}' not found in feature '{slug}'"))?;

    if let Some(t) = title {
        task.title = t.to_string();
    }
    if let Some(d) = description {
        task.description = Some(d.to_string());
    }
    if let Some(deps) = depends {
        task.depends_on = deps
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    feature.save(root).context("failed to save feature")?;

    if json {
        print_json(&serde_json::json!({
            "slug": slug,
            "task_id": task_id,
            "updated": true,
        }))?;
    } else {
        println!("Updated task [{task_id}]");
    }
    Ok(())
}

fn get(root: &Path, slug: &str, task_id: &str, json: bool) -> anyhow::Result<()> {
    let feature =
        Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;
    let task = feature
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .with_context(|| format!("task '{task_id}' not found in feature '{slug}'"))?;

    if json {
        print_json(task)?;
        return Ok(());
    }

    println!("Task: {}", task.id);
    println!("Status:      {}", task.status);
    println!("Title:       {}", task.title);
    if let Some(desc) = &task.description {
        println!("Description: {}", desc);
    }
    if let Some(started) = task.started_at {
        println!("Started:     {}", started.format("%Y-%m-%d %H:%M"));
    }
    if !task.depends_on.is_empty() {
        println!("Depends:     {}", task.depends_on.join(", "));
    }
    println!(
        "Blocker:     {}",
        task.blocker.as_deref().unwrap_or("(none)")
    );

    Ok(())
}

fn search(
    root: &Path,
    query: &str,
    slug: Option<&str>,
    limit: usize,
    json: bool,
) -> anyhow::Result<()> {
    use sdlc_core::search::TaskIndex;

    let features: Vec<_> = if let Some(s) = slug {
        vec![Feature::load(root, s).with_context(|| format!("feature '{s}' not found"))?]
    } else {
        Feature::list(root).context("failed to list features")?
    };

    let index = TaskIndex::build(&features).context("failed to build task index")?;
    let results = index.search(query, limit).context("search failed")?;

    if json {
        let items: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                serde_json::json!({
                    "feature": r.feature_slug,
                    "task_id": r.task_id,
                    "status": r.status,
                    "title": r.title,
                    "score": r.score,
                })
            })
            .collect();
        print_json(&items)?;
        return Ok(());
    }

    if results.is_empty() {
        println!("No tasks matching '{}'.", query);
        return Ok(());
    }

    let count = results.len();
    println!(
        "{count} result{} for \"{}\":",
        if count == 1 { "" } else { "s" },
        query
    );
    println!();
    let rows: Vec<Vec<String>> = results
        .into_iter()
        .map(|r| {
            vec![
                format!("{:.2}", r.score),
                r.feature_slug,
                r.task_id,
                r.status,
                r.title,
            ]
        })
        .collect();
    print_table(&["SCORE", "FEATURE", "TASK ID", "STATUS", "TITLE"], rows);
    Ok(())
}

fn list(root: &Path, slug: Option<&str>, json: bool) -> anyhow::Result<()> {
    if let Some(slug) = slug {
        // Single-feature mode — preserve existing compact output
        let feature =
            Feature::load(root, slug).with_context(|| format!("feature '{slug}' not found"))?;

        if json {
            print_json(&feature.tasks)?;
            return Ok(());
        }

        if feature.tasks.is_empty() {
            println!("No tasks for '{slug}'.");
            return Ok(());
        }

        println!("{}", task_ops::summarize(&feature.tasks));
        println!();

        let rows: Vec<Vec<String>> = feature
            .tasks
            .iter()
            .map(|t| {
                vec![
                    t.id.clone(),
                    t.status.to_string(),
                    t.title.clone(),
                    t.blocker.clone().unwrap_or_default(),
                ]
            })
            .collect();
        print_table(&["ID", "STATUS", "TITLE", "BLOCKER"], rows);
    } else {
        // All-features mode — scan every feature and aggregate
        let features = Feature::list(root).context("failed to list features")?;

        // Collect (feature_slug, task) pairs, skip features with no tasks
        let rows: Vec<Vec<String>> = features
            .iter()
            .flat_map(|f| {
                f.tasks.iter().map(move |t| {
                    vec![
                        f.slug.clone(),
                        t.id.clone(),
                        t.status.to_string(),
                        t.title.clone(),
                        t.blocker.clone().unwrap_or_default(),
                    ]
                })
            })
            .collect();

        if json {
            let items: Vec<serde_json::Value> = features
                .iter()
                .flat_map(|f| {
                    f.tasks.iter().map(move |t| {
                        serde_json::json!({
                            "feature": f.slug,
                            "task_id": t.id,
                            "status": t.status,
                            "title": t.title,
                            "blocker": t.blocker,
                        })
                    })
                })
                .collect();
            print_json(&items)?;
            return Ok(());
        }

        if rows.is_empty() {
            println!("No tasks found.");
            return Ok(());
        }

        let total: usize = rows.len();
        let done = rows.iter().filter(|r| r[2] == "completed").count();
        println!(
            "{done}/{total} tasks completed across {} feature(s)",
            features.iter().filter(|f| !f.tasks.is_empty()).count()
        );
        println!();
        print_table(&["FEATURE", "ID", "STATUS", "TITLE", "BLOCKER"], rows);
    }
    Ok(())
}
