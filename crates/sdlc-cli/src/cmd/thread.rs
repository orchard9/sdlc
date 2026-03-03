//! `sdlc thread` — manage feedback threads (create, post, list, show).

use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand enum
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum ThreadSubcommand {
    /// Create a new feedback thread anchored to a context
    Create {
        /// Context string, e.g. "feature:my-slug" or "ponder:idea"
        context: String,

        /// Thread title (defaults to "Discussion: <context>")
        #[arg(long)]
        title: Option<String>,
    },

    /// Append a post to an existing thread
    Post {
        /// Thread ID
        id: String,

        /// Author identifier: "human" or "agent:<name>"
        #[arg(long, required = true)]
        author: String,

        /// Post content
        content: String,
    },

    /// List all threads
    List {
        /// Filter by context string
        #[arg(long)]
        context: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show a thread and all its posts
    Show {
        /// Thread ID
        id: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcommand: ThreadSubcommand, _json: bool) -> anyhow::Result<()> {
    match subcommand {
        ThreadSubcommand::Create { context, title } => {
            let title_str = title.as_deref().unwrap_or("");
            let thread = sdlc_core::feedback_thread::create_thread(root, &context, title_str, None)
                .with_context(|| format!("failed to create thread for context '{context}'"))?;
            println!("{}", thread.id);
            Ok(())
        }

        ThreadSubcommand::Post {
            id,
            author,
            content,
        } => {
            if author.trim().is_empty() {
                anyhow::bail!("--author cannot be empty");
            }
            if content.trim().is_empty() {
                anyhow::bail!("content cannot be empty");
            }
            let post = sdlc_core::feedback_thread::add_post(root, &id, &author, &content)
                .with_context(|| format!("failed to add post to thread '{id}'"))?;
            println!("{}", post.seq);
            Ok(())
        }

        ThreadSubcommand::List { context, json } => {
            let threads = sdlc_core::feedback_thread::list_threads(root, context.as_deref())
                .context("failed to list threads")?;

            if json {
                return print_json(&threads);
            }

            if threads.is_empty() {
                println!("No threads found.");
                return Ok(());
            }

            let rows: Vec<Vec<String>> = threads
                .iter()
                .map(|t| {
                    vec![
                        t.id.clone(),
                        t.context.clone(),
                        t.post_count.to_string(),
                        t.updated_at.format("%Y-%m-%d %H:%M").to_string(),
                    ]
                })
                .collect();
            print_table(&["ID", "CONTEXT", "POSTS", "UPDATED"], rows);
            Ok(())
        }

        ThreadSubcommand::Show { id, json } => {
            let thread = sdlc_core::feedback_thread::load_thread(root, &id)
                .with_context(|| format!("thread '{id}' not found"))?;
            let posts = sdlc_core::feedback_thread::list_posts(root, &id)
                .with_context(|| format!("failed to load posts for thread '{id}'"))?;

            if json {
                let mut value = serde_json::to_value(&thread)?;
                value["posts"] = serde_json::to_value(&posts)?;
                return print_json(&value);
            }

            println!("ID:      {}", thread.id);
            println!("Title:   {}", thread.title);
            println!("Context: {}", thread.context);
            println!("Posts:   {}", thread.post_count);
            println!(
                "Created: {}",
                thread.created_at.format("%Y-%m-%d %H:%M UTC")
            );
            println!(
                "Updated: {}",
                thread.updated_at.format("%Y-%m-%d %H:%M UTC")
            );

            if posts.is_empty() {
                println!("\n(no posts yet)");
            } else {
                println!();
                for post in &posts {
                    println!(
                        "--- #{} [{}] {} ---",
                        post.seq,
                        post.author,
                        post.created_at.format("%Y-%m-%d %H:%M UTC")
                    );
                    println!("{}", post.content);
                    println!();
                }
            }
            Ok(())
        }
    }
}
