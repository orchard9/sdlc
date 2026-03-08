use crate::output::print_json;
use anyhow::{bail, Context};
use chrono::{DateTime, Duration, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Internal run record (minimal — only fields we need from .sdlc/.runs/*.json)
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RawRun {
    id: String,
    key: String,
    run_type: String,
    label: String,
    status: String,
    started_at: String,
    error: Option<String>,
    cost_usd: Option<f64>,
}

// ---------------------------------------------------------------------------
// Category
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
enum Category {
    RunFailed,
    FeatureMerged,
    Approval,
    PhaseAdvanced,
    AgentRun,
    RunStopped,
}

impl Category {
    fn icon(&self) -> &'static str {
        match self {
            Category::RunFailed => "⚠️",
            Category::FeatureMerged => "🚀",
            Category::Approval => "✅",
            Category::PhaseAdvanced => "🔄",
            Category::AgentRun => "▶",
            Category::RunStopped => "⏹",
        }
    }
}

fn classify(run: &RawRun) -> Category {
    let status = run.status.as_str();
    let run_type = run.run_type.as_str();
    let key = run.key.as_str();

    if status == "failed" || (status == "stopped" && run.error.is_some()) {
        Category::RunFailed
    } else if run_type == "merge" || key.contains("merge") {
        Category::FeatureMerged
    } else if run_type.contains("approve") || key.contains("approve") {
        Category::Approval
    } else if run_type.contains("transition") || key.contains("transition") {
        Category::PhaseAdvanced
    } else if status == "stopped" {
        Category::RunStopped
    } else {
        Category::AgentRun
    }
}

// ---------------------------------------------------------------------------
// SinceSpec
// ---------------------------------------------------------------------------

enum SinceSpec {
    Iso(DateTime<Utc>),
    Relative(Duration),
    LastMerge,
}

fn parse_since(s: &str) -> anyhow::Result<SinceSpec> {
    if s == "last-merge" {
        return Ok(SinceSpec::LastMerge);
    }
    // Relative: Nd or Nw
    if let Some(days_str) = s.strip_suffix('d') {
        let n: i64 = days_str
            .parse()
            .with_context(|| format!("Invalid --since value: '{s}'"))?;
        return Ok(SinceSpec::Relative(Duration::days(n)));
    }
    if let Some(weeks_str) = s.strip_suffix('w') {
        let n: i64 = weeks_str
            .parse()
            .with_context(|| format!("Invalid --since value: '{s}'"))?;
        return Ok(SinceSpec::Relative(Duration::weeks(n)));
    }
    // ISO date: YYYY-MM-DD
    if let Ok(date) = NaiveDate::parse_from_str(s, "%Y-%m-%d") {
        let dt = date
            .and_hms_opt(0, 0, 0)
            .expect("midnight always valid")
            .and_utc();
        return Ok(SinceSpec::Iso(dt));
    }
    bail!("Invalid --since value: '{}'. Expected ISO date (2026-03-01), relative (7d, 1w), or last-merge", s)
}

// ---------------------------------------------------------------------------
// Load
// ---------------------------------------------------------------------------

fn load_runs(root: &Path) -> Vec<RawRun> {
    let runs_dir = root.join(".sdlc").join(".runs");
    let entries = match std::fs::read_dir(&runs_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    let mut runs: Vec<RawRun> = entries
        .filter_map(|e| e.ok())
        .filter(|e| {
            let name = e.file_name();
            let s = name.to_string_lossy();
            s.ends_with(".json") && !s.ends_with(".events.json")
        })
        .filter_map(|e| {
            let data = std::fs::read_to_string(e.path()).ok()?;
            serde_json::from_str::<RawRun>(&data).ok()
        })
        .collect();
    // newest first
    runs.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    runs
}

fn parse_started_at(s: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

// ---------------------------------------------------------------------------
// Output structs
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct ChangelogEvent {
    id: String,
    category: Category,
    icon: String,
    label: String,
    run_type: String,
    status: String,
    started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cost_usd: Option<f64>,
}

#[derive(Serialize)]
struct ChangelogOutput {
    since: String,
    limit: usize,
    total: usize,
    events: Vec<ChangelogEvent>,
}

// ---------------------------------------------------------------------------
// Formatting
// ---------------------------------------------------------------------------

fn format_relative(delta: Duration) -> String {
    let secs = delta.num_seconds();
    if secs < 60 {
        format!("{secs} sec ago")
    } else if secs < 3600 {
        format!("{} min ago", secs / 60)
    } else if secs < 86400 {
        format!("{} hr ago", secs / 3600)
    } else {
        format!("{} days ago", secs / 86400)
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

pub fn run(root: &Path, since_str: &str, limit: usize, json: bool) -> anyhow::Result<()> {
    let since_spec = parse_since(since_str)?;
    let all_runs = load_runs(root);
    let now = Utc::now();

    // Resolve cutoff
    let cutoff: DateTime<Utc> = match &since_spec {
        SinceSpec::Iso(dt) => *dt,
        SinceSpec::Relative(dur) => now - *dur,
        SinceSpec::LastMerge => {
            // Find the most recent merge run
            let merge_run = all_runs
                .iter()
                .find(|r| r.run_type == "merge" || r.key.contains("merge"));
            match merge_run {
                Some(r) => match parse_started_at(&r.started_at) {
                    Some(dt) => dt,
                    None => {
                        eprintln!("Warning: could not parse merge timestamp, defaulting to 7d");
                        now - Duration::days(7)
                    }
                },
                None => {
                    eprintln!("Warning: no merge found, defaulting to 7d");
                    now - Duration::days(7)
                }
            }
        }
    };

    // Filter by cutoff
    let filtered: Vec<&RawRun> = all_runs
        .iter()
        .filter(|r| {
            parse_started_at(&r.started_at)
                .map(|dt| dt >= cutoff)
                .unwrap_or(false)
        })
        .take(limit)
        .collect();

    if json {
        let events: Vec<ChangelogEvent> = filtered
            .iter()
            .map(|r| {
                let cat = classify(r);
                let icon = cat.icon().to_string();
                ChangelogEvent {
                    id: r.id.clone(),
                    icon,
                    category: cat,
                    label: r.label.clone(),
                    run_type: r.run_type.clone(),
                    status: r.status.clone(),
                    started_at: r.started_at.clone(),
                    cost_usd: r.cost_usd,
                }
            })
            .collect();
        let total = events.len();
        let output = ChangelogOutput {
            since: cutoff.to_rfc3339(),
            limit,
            total,
            events,
        };
        print_json(&output)?;
    } else {
        if filtered.is_empty() {
            println!("No activity in the selected window.");
            return Ok(());
        }
        for r in &filtered {
            let cat = classify(r);
            let icon = cat.icon();
            let label = &r.label;
            let rel = parse_started_at(&r.started_at)
                .map(|dt| format_relative(now - dt))
                .unwrap_or_else(|| "unknown".to_string());
            // Left-pad label to 45 chars
            println!("{icon}  {label:<45}  {rel}");
        }
    }

    Ok(())
}

/// Reassign a range of sequential event IDs in the changelog.
///
/// Called by `sdlc changelog reassign --from ev-0623 --suffix x --count 7`.
pub fn reassign(root: &Path, from: &str, suffix: &str, count: usize) -> anyhow::Result<()> {
    let n = sdlc_core::event_log::reassign_ids(root, from, suffix, count)
        .context("failed to reassign changelog IDs")?;
    if n == 0 {
        eprintln!("No events matched the specified range.");
    } else {
        eprintln!("✓ Reassigned {n} event(s): {from}..+{count} → suffix '{suffix}'");
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_run(run_type: &str, status: &str, key: &str, error: Option<&str>) -> RawRun {
        RawRun {
            id: "test-id".into(),
            key: key.into(),
            run_type: run_type.into(),
            label: "Test label".into(),
            status: status.into(),
            started_at: "2026-03-02T10:00:00+00:00".into(),
            error: error.map(str::to_string),
            cost_usd: None,
        }
    }

    #[test]
    fn test_classify_run_failed() {
        let r = make_run("ponder", "failed", "ponder:test", None);
        assert!(matches!(classify(&r), Category::RunFailed));
    }

    #[test]
    fn test_classify_run_stopped_with_error() {
        let r = make_run("ponder", "stopped", "ponder:test", Some("timeout"));
        assert!(matches!(classify(&r), Category::RunFailed));
    }

    #[test]
    fn test_classify_merge() {
        let r = make_run("merge", "completed", "merge:my-feature", None);
        assert!(matches!(classify(&r), Category::FeatureMerged));
    }

    #[test]
    fn test_classify_approval_by_run_type() {
        let r = make_run("approve_spec", "completed", "feature:foo", None);
        assert!(matches!(classify(&r), Category::Approval));
    }

    #[test]
    fn test_classify_approval_by_key() {
        let r = make_run("agent", "completed", "approve:audit:foo", None);
        assert!(matches!(classify(&r), Category::Approval));
    }

    #[test]
    fn test_classify_agent_run() {
        let r = make_run(
            "knowledge_harvest",
            "completed",
            "knowledge:harvest:foo",
            None,
        );
        assert!(matches!(classify(&r), Category::AgentRun));
    }

    #[test]
    fn test_classify_run_stopped_no_error() {
        let r = make_run("ponder", "stopped", "ponder:foo", None);
        assert!(matches!(classify(&r), Category::RunStopped));
    }

    #[test]
    fn test_parse_since_relative_days() {
        let spec = parse_since("7d").unwrap();
        assert!(matches!(spec, SinceSpec::Relative(_)));
        if let SinceSpec::Relative(d) = spec {
            assert_eq!(d.num_days(), 7);
        }
    }

    #[test]
    fn test_parse_since_relative_weeks() {
        let spec = parse_since("1w").unwrap();
        assert!(matches!(spec, SinceSpec::Relative(_)));
        if let SinceSpec::Relative(d) = spec {
            assert_eq!(d.num_days(), 7);
        }
    }

    #[test]
    fn test_parse_since_iso() {
        let spec = parse_since("2026-03-01").unwrap();
        assert!(matches!(spec, SinceSpec::Iso(_)));
        if let SinceSpec::Iso(dt) = spec {
            assert_eq!(dt.format("%Y-%m-%d").to_string(), "2026-03-01");
        }
    }

    #[test]
    fn test_parse_since_last_merge() {
        let spec = parse_since("last-merge").unwrap();
        assert!(matches!(spec, SinceSpec::LastMerge));
    }

    #[test]
    fn test_parse_since_invalid() {
        assert!(parse_since("badvalue").is_err());
    }

    #[test]
    fn test_relative_time_format_seconds() {
        assert_eq!(format_relative(Duration::seconds(45)), "45 sec ago");
    }

    #[test]
    fn test_relative_time_format_minutes() {
        assert_eq!(format_relative(Duration::seconds(90)), "1 min ago");
    }

    #[test]
    fn test_relative_time_format_hours() {
        assert_eq!(format_relative(Duration::seconds(3700)), "1 hr ago");
    }

    #[test]
    fn test_relative_time_format_days() {
        assert_eq!(format_relative(Duration::seconds(90000)), "1 days ago");
    }
}
