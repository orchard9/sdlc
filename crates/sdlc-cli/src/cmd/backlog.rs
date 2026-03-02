use crate::output::{print_json, print_table};
use anyhow::Context;
use clap::Subcommand;
use sdlc_core::{
    backlog::{BacklogKind, BacklogStatus, BacklogStore},
    feature::Feature,
    milestone::Milestone,
    state::State,
};
use std::path::Path;

// ---------------------------------------------------------------------------
// Subcommand enum
// ---------------------------------------------------------------------------

#[derive(Subcommand)]
pub enum BacklogSubcommand {
    /// Capture a new backlog item (concern, idea, or debt)
    Add {
        /// Title of the backlog item (variadic — no quoting needed)
        #[arg(required = true)]
        title: Vec<String>,

        /// Multi-line context explaining the concern in detail
        #[arg(long)]
        description: Option<String>,

        /// Classification: concern (default), idea, or debt
        #[arg(long, default_value = "concern", value_parser = parse_kind)]
        kind: BacklogKind,

        /// Grounding reference: file path, function name, or failing test
        #[arg(long)]
        evidence: Option<String>,

        /// Feature slug that was active when this was discovered
        /// (auto-inferred from state.yaml if omitted)
        #[arg(long)]
        source_feature: Option<String>,
    },

    /// List backlog items (default: open only)
    List {
        /// Show all statuses (open, parked, promoted)
        #[arg(long, conflicts_with = "status")]
        all: bool,

        /// Filter by status: open, parked, or promoted
        #[arg(long)]
        status: Option<String>,

        /// Filter by feature origin
        #[arg(long)]
        source_feature: Option<String>,
    },

    /// Park an item (requires --reason)
    Park {
        /// Backlog item ID (e.g. B1)
        id: String,

        /// Why this item is being de-prioritized (required)
        #[arg(long, required = true)]
        reason: Vec<String>,
    },

    /// Promote a backlog item to a tracked feature
    Promote {
        /// Backlog item ID (e.g. B1)
        id: String,

        /// Feature slug to create (auto-generated from title if omitted)
        #[arg(long)]
        slug: Option<String>,

        /// Milestone slug to link the new feature to
        #[arg(long)]
        milestone: Option<String>,
    },

    /// Show full details for a single backlog item
    Show {
        /// Backlog item ID (e.g. B1)
        id: String,
    },
}

// ---------------------------------------------------------------------------
// Dispatch
// ---------------------------------------------------------------------------

pub fn run(root: &Path, subcmd: BacklogSubcommand, json: bool) -> anyhow::Result<()> {
    match subcmd {
        BacklogSubcommand::Add {
            title,
            description,
            kind,
            evidence,
            source_feature,
        } => add(
            root,
            title,
            description,
            kind,
            evidence,
            source_feature,
            json,
        ),

        BacklogSubcommand::List {
            all,
            status,
            source_feature,
        } => list(
            root,
            all,
            status.as_deref(),
            source_feature.as_deref(),
            json,
        ),

        BacklogSubcommand::Park { id, reason } => park(root, &id, &reason.join(" "), json),

        BacklogSubcommand::Promote {
            id,
            slug,
            milestone,
        } => promote(root, &id, slug.as_deref(), milestone.as_deref(), json),

        BacklogSubcommand::Show { id } => show(root, &id, json),
    }
}

// ---------------------------------------------------------------------------
// Subcommand implementations
// ---------------------------------------------------------------------------

fn add(
    root: &Path,
    title_words: Vec<String>,
    description: Option<String>,
    kind: BacklogKind,
    evidence: Option<String>,
    source_feature: Option<String>,
    json: bool,
) -> anyhow::Result<()> {
    let title = title_words.join(" ");

    // Auto-infer source_feature when not explicitly provided
    let source_feature = if source_feature.is_some() {
        source_feature
    } else {
        infer_source_feature(root)
    };

    let item = BacklogStore::add(
        root,
        title,
        kind,
        description,
        evidence,
        source_feature.clone(),
    )
    .context("failed to add backlog item")?;

    if json {
        print_json(&item)?;
    } else {
        let source_display = source_feature.as_deref().unwrap_or("none");
        println!(
            "Backlog item {} recorded: \"{}\" [{}]",
            item.id, item.title, source_display
        );
    }
    Ok(())
}

fn list(
    root: &Path,
    all: bool,
    status: Option<&str>,
    source_feature: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let status_filter = if all {
        None
    } else if let Some(s) = status {
        Some(parse_status(s)?)
    } else {
        // Default: show open items only
        Some(BacklogStatus::Open)
    };

    let items = BacklogStore::list(root, status_filter, source_feature)
        .context("failed to list backlog items")?;

    if json {
        print_json(&items)?;
        return Ok(());
    }

    if items.is_empty() {
        println!("No backlog items found.");
        return Ok(());
    }

    let rows: Vec<Vec<String>> = items
        .iter()
        .map(|item| {
            vec![
                item.id.clone(),
                item.kind.to_string(),
                item.status.to_string(),
                item.source_feature.as_deref().unwrap_or("-").to_string(),
                item.title.clone(),
            ]
        })
        .collect();

    print_table(&["ID", "KIND", "STATUS", "SOURCE", "TITLE"], rows);
    Ok(())
}

fn park(root: &Path, id: &str, reason: &str, json: bool) -> anyhow::Result<()> {
    let item = BacklogStore::park(root, id, reason.to_string())
        .with_context(|| format!("failed to park backlog item '{id}'"))?;

    if json {
        print_json(&item)?;
    } else {
        println!("Parked {}: {}", item.id, reason);
    }
    Ok(())
}

fn promote(
    root: &Path,
    id: &str,
    slug_override: Option<&str>,
    milestone_slug: Option<&str>,
    json: bool,
) -> anyhow::Result<()> {
    let item =
        BacklogStore::get(root, id).with_context(|| format!("backlog item '{id}' not found"))?;

    let feature_slug = if let Some(s) = slug_override {
        s.to_string()
    } else {
        slugify(&item.title)
    };

    // Create the feature
    Feature::create_with_description(root, &feature_slug, &item.title, item.description.clone())
        .with_context(|| format!("failed to create feature '{feature_slug}'"))?;

    // Add to active_features in state
    let mut state = State::load(root).context("failed to load state")?;
    state.add_active_feature(&feature_slug);
    state.save(root).context("failed to save state")?;

    // Mark the backlog item as promoted
    let promoted_item = BacklogStore::mark_promoted(root, id, &feature_slug)
        .with_context(|| format!("failed to mark backlog item '{id}' as promoted"))?;

    // Optionally link to a milestone
    if let Some(ms) = milestone_slug {
        let mut milestone =
            Milestone::load(root, ms).with_context(|| format!("milestone '{ms}' not found"))?;
        milestone.add_feature(&feature_slug);
        milestone
            .save(root)
            .with_context(|| format!("failed to save milestone '{ms}'"))?;
    }

    if json {
        print_json(&promoted_item)?;
    } else {
        println!("Promoted {} → feature: {}", id, feature_slug);
        if let Some(ms) = milestone_slug {
            println!("Added to milestone: {}", ms);
        }
    }
    Ok(())
}

fn show(root: &Path, id: &str, json: bool) -> anyhow::Result<()> {
    let item =
        BacklogStore::get(root, id).with_context(|| format!("backlog item '{id}' not found"))?;

    if json {
        print_json(&item)?;
        return Ok(());
    }

    println!("ID:      {}", item.id);
    println!("Kind:    {}", item.kind);
    println!("Status:  {}", item.status);
    println!("Title:   {}", item.title);
    if let Some(desc) = &item.description {
        println!("Description: {}", desc);
    }
    if let Some(ev) = &item.evidence {
        println!("Evidence:    {}", ev);
    }
    if let Some(sf) = &item.source_feature {
        println!("Source:      {}", sf);
    }
    if let Some(reason) = &item.park_reason {
        println!("Park Reason: {}", reason);
    }
    if let Some(promoted_to) = &item.promoted_to {
        println!("Promoted To: {}", promoted_to);
    }
    println!(
        "Created:     {}",
        item.created_at.format("%Y-%m-%d %H:%M UTC")
    );
    println!(
        "Updated:     {}",
        item.updated_at.format("%Y-%m-%d %H:%M UTC")
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Auto-infer source_feature from state.yaml active_features (last entry).
/// Prints a warning to stderr if no active feature found. Never blocks.
fn infer_source_feature(root: &Path) -> Option<String> {
    match State::load(root) {
        Ok(state) => {
            if let Some(slug) = state.active_features.last() {
                Some(slug.clone())
            } else {
                eprintln!(
                    "warning: no active feature found in state.yaml; source_feature not recorded"
                );
                None
            }
        }
        Err(_) => {
            eprintln!("warning: could not load state.yaml; source_feature not recorded");
            None
        }
    }
}

/// Convert a free-form title string to a kebab-case slug, max 40 chars.
pub fn slugify(title: &str) -> String {
    // Replace non-alphanumeric chars with dashes, lowercase
    let raw: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect();

    // Collapse consecutive dashes and trim leading/trailing dashes
    let collapsed: String = raw
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    if collapsed.len() <= 40 {
        collapsed
    } else {
        // Truncate at a word boundary (last dash within 40 chars)
        let truncated = &collapsed[..40];
        match truncated.rfind('-') {
            Some(pos) => collapsed[..pos].to_string(),
            None => truncated.to_string(),
        }
    }
}

/// Parse a kind string for Clap value_parser.
fn parse_kind(s: &str) -> Result<BacklogKind, String> {
    match s {
        "concern" => Ok(BacklogKind::Concern),
        "idea" => Ok(BacklogKind::Idea),
        "debt" => Ok(BacklogKind::Debt),
        other => Err(format!(
            "unknown kind '{other}'; expected concern, idea, or debt"
        )),
    }
}

/// Parse a status string for --status flag.
fn parse_status(s: &str) -> anyhow::Result<BacklogStatus> {
    match s {
        "open" => Ok(BacklogStatus::Open),
        "parked" => Ok(BacklogStatus::Parked),
        "promoted" => Ok(BacklogStatus::Promoted),
        other => anyhow::bail!("unknown status '{other}'; expected open, parked, or promoted"),
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Fix auth token race"), "fix-auth-token-race");
    }

    #[test]
    fn slugify_special_characters() {
        assert_eq!(
            slugify("Fix auth.rs: token race! (critical)"),
            "fix-auth-rs-token-race-critical"
        );
    }

    #[test]
    fn slugify_truncates_at_word_boundary() {
        let long = "a very long title that exceeds forty characters limit here";
        let result = slugify(long);
        assert!(result.len() <= 40, "slug too long: {result}");
        assert!(!result.ends_with('-'), "slug ends with dash: {result}");
    }

    #[test]
    fn slugify_short_no_dash_truncation() {
        // If there's no dash in the first 40 chars, just truncate at 40
        let no_dashes = "abcdefghijklmnopqrstuvwxyzabcdefghijklmnopqrstuvwxyz";
        let result = slugify(no_dashes);
        assert!(result.len() <= 40);
    }

    #[test]
    fn slugify_collapses_multiple_dashes() {
        assert_eq!(slugify("foo   bar"), "foo-bar");
    }

    #[test]
    fn parse_kind_valid() {
        assert!(matches!(parse_kind("concern"), Ok(BacklogKind::Concern)));
        assert!(matches!(parse_kind("idea"), Ok(BacklogKind::Idea)));
        assert!(matches!(parse_kind("debt"), Ok(BacklogKind::Debt)));
    }

    #[test]
    fn parse_kind_invalid() {
        assert!(parse_kind("badkind").is_err());
        let err = parse_kind("badkind").unwrap_err();
        assert!(err.contains("unknown kind 'badkind'"));
    }

    #[test]
    fn parse_status_valid() {
        assert!(matches!(parse_status("open"), Ok(BacklogStatus::Open)));
        assert!(matches!(parse_status("parked"), Ok(BacklogStatus::Parked)));
        assert!(matches!(
            parse_status("promoted"),
            Ok(BacklogStatus::Promoted)
        ));
    }

    #[test]
    fn parse_status_invalid() {
        assert!(parse_status("unknown").is_err());
    }
}
