use crate::output::print_table;
use clap::Subcommand;
use sdlc_core::escalation::{self, EscalationKind};
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand tree
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum EscalateSubcommand {
    /// Create a new escalation (optionally linked to a feature)
    Create {
        /// Kind: secret_request | question | vision | manual_test
        #[arg(long)]
        kind: String,

        /// Short, descriptive title
        #[arg(long)]
        title: String,

        /// Context explaining why human action is needed
        #[arg(long)]
        context: String,

        /// Feature slug to link (adds a blocking comment automatically)
        #[arg(long)]
        feature: Option<String>,
    },

    /// List escalations (default: open only)
    List {
        /// Filter by status: open | resolved | all  [default: open]
        #[arg(long, default_value = "open")]
        status: String,
    },

    /// Show details of a single escalation
    Show {
        /// Escalation ID (e.g. E1)
        id: String,
    },

    /// Resolve an escalation with a note
    Resolve {
        /// Escalation ID (e.g. E1)
        id: String,

        /// Human-readable resolution notes
        resolution: String,
    },
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcommand: EscalateSubcommand, json: bool) -> anyhow::Result<()> {
    match subcommand {
        EscalateSubcommand::Create {
            kind,
            title,
            context,
            feature,
        } => {
            let kind: EscalationKind = kind.parse()?;
            let item = escalation::create(root, kind, &title, &context, feature.as_deref())?;
            if json {
                crate::output::print_json(&item)?;
            } else {
                println!("created escalation {}", item.id);
                if let Some(slug) = &item.source_feature {
                    println!(
                        "  feature '{}' is now gated by blocker comment {}",
                        slug,
                        item.linked_comment_id.as_deref().unwrap_or("?")
                    );
                }
                println!();
                println!(
                    "The escalation will appear in the Dashboard under \"Needs Your Attention\"."
                );
                println!("After creating an escalation, stop the current run — the human must act first.");
            }
            Ok(())
        }

        EscalateSubcommand::List { status } => {
            let items = escalation::list(root, Some(status.as_str()))?;
            if json {
                crate::output::print_json(&items)?;
                return Ok(());
            }
            if items.is_empty() {
                println!("no escalations (status: {status})");
                return Ok(());
            }
            print_table(
                &["ID", "KIND", "STATUS", "TITLE"],
                items
                    .iter()
                    .map(|e| {
                        vec![
                            e.id.clone(),
                            e.kind.to_string(),
                            e.status.to_string(),
                            truncate(&e.title, 60),
                        ]
                    })
                    .collect(),
            );
            Ok(())
        }

        EscalateSubcommand::Show { id } => {
            let item = escalation::get(root, &id)?;
            if json {
                crate::output::print_json(&item)?;
            } else {
                println!("ID:      {}", item.id);
                println!("Kind:    {}", item.kind);
                println!("Status:  {}", item.status);
                println!("Title:   {}", item.title);
                println!("Context: {}", item.context);
                if let Some(slug) = &item.source_feature {
                    println!("Feature: {slug}");
                }
                if let Some(res) = &item.resolution {
                    println!("Resolved: {res}");
                }
                println!("Created: {}", item.created_at.format("%Y-%m-%d %H:%M UTC"));
            }
            Ok(())
        }

        EscalateSubcommand::Resolve { id, resolution } => {
            let item = escalation::resolve(root, &id, &resolution)?;
            if json {
                crate::output::print_json(&item)?;
            } else {
                println!("resolved escalation {}", item.id);
                if let Some(slug) = &item.source_feature {
                    println!("  blocker comment removed from '{slug}'");
                }
            }
            Ok(())
        }
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max.saturating_sub(1)])
    }
}
