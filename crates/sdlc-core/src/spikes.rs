//! Spike data layer.
//!
//! Spikes are time-boxed technical investigations whose findings live at
//! `.sdlc/spikes/<slug>/findings.md` (written by the `/sdlc-spike` agent).
//!
//! This module is read-only with respect to findings.md — it never modifies
//! spike findings. It writes only `.sdlc/spikes/<slug>/state.yaml` to track
//! promotion and knowledge-filing outcomes.

use crate::error::{Result, SdlcError};
use crate::paths::{spike_findings_path, spike_state_path, SPIKES_DIR};
use serde::{Deserialize, Serialize};
use std::path::Path;

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// The verdict recorded in a spike's findings.md.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpikeVerdict {
    Adopt,
    Adapt,
    Reject,
}

impl std::fmt::Display for SpikeVerdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpikeVerdict::Adopt => write!(f, "ADOPT"),
            SpikeVerdict::Adapt => write!(f, "ADAPT"),
            SpikeVerdict::Reject => write!(f, "REJECT"),
        }
    }
}

/// Metadata and derived fields for a single spike.
#[derive(Debug, Clone)]
pub struct SpikeEntry {
    pub slug: String,
    pub title: String,
    pub verdict: Option<SpikeVerdict>,
    /// ISO date string as found in findings.md (e.g. "2026-03-04").
    pub date: Option<String>,
    /// Text of the `## The Question` section.
    pub the_question: Option<String>,
    /// Ponder slug recorded after `promote_to_ponder` was called.
    pub ponder_slug: Option<String>,
    /// Knowledge slug recorded after `store_in_knowledge` was called.
    pub knowledge_slug: Option<String>,
}

// ---------------------------------------------------------------------------
// Internal state (state.yaml)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct SpikeState {
    #[serde(skip_serializing_if = "Option::is_none")]
    verdict: Option<SpikeVerdict>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ponder_slug: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    knowledge_slug: Option<String>,
}

fn read_state(root: &Path, slug: &str) -> SpikeState {
    let path = spike_state_path(root, slug);
    if !path.exists() {
        return SpikeState::default();
    }
    std::fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_yaml::from_str(&s).ok())
        .unwrap_or_default()
}

fn write_state(root: &Path, slug: &str, state: &SpikeState) -> Result<()> {
    let path = spike_state_path(root, slug);
    let data = serde_yaml::to_string(state)
        .map_err(|e| SdlcError::Io(std::io::Error::other(e.to_string())))?;
    crate::io::atomic_write(&path, data.as_bytes())
}

// ---------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------

struct ParsedFindings {
    title: Option<String>,
    verdict: Option<SpikeVerdict>,
    date: Option<String>,
    the_question: Option<String>,
    open_questions: Option<String>,
}

fn parse_findings(content: &str) -> ParsedFindings {
    let mut title: Option<String> = None;
    let mut verdict: Option<SpikeVerdict> = None;
    let mut date: Option<String> = None;
    let mut the_question: Option<String> = None;
    let mut open_questions: Option<String> = None;

    // State machine for section collection
    enum Section {
        None,
        TheQuestion,
        OpenQuestions,
    }
    let mut section = Section::None;
    let mut q_buf: Vec<&str> = Vec::new();
    let mut oq_buf: Vec<&str> = Vec::new();

    for line in content.lines() {
        // Header fields (only scan before sections get complex)
        if title.is_none() {
            if let Some(rest) = line.strip_prefix("# Spike:") {
                title = Some(rest.trim().to_string());
                continue;
            }
        }
        if verdict.is_none() {
            if let Some(rest) = line.strip_prefix("**Verdict:**") {
                let v = rest.trim().to_uppercase();
                verdict = match v.as_str() {
                    "ADOPT" => Some(SpikeVerdict::Adopt),
                    "ADAPT" => Some(SpikeVerdict::Adapt),
                    "REJECT" => Some(SpikeVerdict::Reject),
                    _ => None,
                };
                continue;
            }
        }
        if date.is_none() {
            if let Some(rest) = line.strip_prefix("**Date:**") {
                date = Some(rest.trim().to_string());
                continue;
            }
        }

        // Section detection
        if line.starts_with("## ") {
            // Flush current section buffers
            if matches!(section, Section::TheQuestion) && !q_buf.is_empty() {
                the_question = Some(q_buf.join("\n").trim().to_string());
                q_buf.clear();
            }
            if matches!(section, Section::OpenQuestions) && !oq_buf.is_empty() {
                open_questions = Some(oq_buf.join("\n").trim().to_string());
                oq_buf.clear();
            }

            let heading = line.trim_start_matches('#').trim();
            section = if heading == "The Question" {
                Section::TheQuestion
            } else if heading == "Risks and Open Questions" {
                Section::OpenQuestions
            } else {
                Section::None
            };
            continue;
        }

        // Collect section lines
        match section {
            Section::TheQuestion => q_buf.push(line),
            Section::OpenQuestions => oq_buf.push(line),
            Section::None => {}
        }
    }

    // Flush remaining buffers
    if matches!(section, Section::TheQuestion) && !q_buf.is_empty() {
        the_question = Some(q_buf.join("\n").trim().to_string());
    }
    if matches!(section, Section::OpenQuestions) && !oq_buf.is_empty() {
        open_questions = Some(oq_buf.join("\n").trim().to_string());
    }

    ParsedFindings {
        title,
        verdict,
        date,
        the_question,
        open_questions,
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Extract the "Risks and Open Questions" section text from a findings.md string.
pub fn extract_open_questions(findings: &str) -> Option<String> {
    parse_findings(findings).open_questions
}

/// List all spikes from `.sdlc/spikes/`.
///
/// For each REJECT spike that has no `knowledge_slug` yet, auto-calls
/// `store_in_knowledge` as a background side effect. Errors from that call
/// are ignored so the list always completes.
///
/// Returns entries sorted by date descending (undated entries last).
pub fn list(root: &Path) -> Result<Vec<SpikeEntry>> {
    let spikes_dir = root.join(SPIKES_DIR);
    if !spikes_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries: Vec<SpikeEntry> = Vec::new();

    for dir_entry in std::fs::read_dir(&spikes_dir)? {
        let dir_entry = dir_entry?;
        if !dir_entry.file_type()?.is_dir() {
            continue;
        }
        let slug = dir_entry.file_name().to_string_lossy().into_owned();
        let state = read_state(root, &slug);

        let findings_path = spike_findings_path(root, &slug);
        let (parsed, _raw) = if findings_path.exists() {
            let raw = std::fs::read_to_string(&findings_path)?;
            let p = parse_findings(&raw);
            (p, Some(raw))
        } else {
            (
                ParsedFindings {
                    title: None,
                    verdict: None,
                    date: None,
                    the_question: None,
                    open_questions: None,
                },
                None,
            )
        };

        let entry = SpikeEntry {
            slug: slug.clone(),
            title: parsed.title.unwrap_or_else(|| slug.clone()),
            verdict: parsed.verdict,
            date: parsed.date,
            the_question: parsed.the_question,
            ponder_slug: state.ponder_slug.clone(),
            knowledge_slug: state.knowledge_slug.clone(),
        };

        // Auto-file REJECT spikes with no knowledge entry yet
        if matches!(entry.verdict, Some(SpikeVerdict::Reject)) && entry.knowledge_slug.is_none() {
            let _ = store_in_knowledge(root, &slug);
        }

        entries.push(entry);
    }

    // Sort by date descending; undated entries go last
    entries.sort_by(|a, b| match (&b.date, &a.date) {
        (Some(bd), Some(ad)) => bd.cmp(ad),
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.slug.cmp(&b.slug),
    });

    Ok(entries)
}

/// Load a single spike entry plus the raw findings.md content.
///
/// Returns `(SpikeEntry, findings_content)`. If findings.md is absent,
/// `findings_content` is an empty string and entry fields are `None`.
pub fn load(root: &Path, slug: &str) -> Result<(SpikeEntry, String)> {
    crate::paths::validate_slug(slug)?;
    let spikes_dir = root.join(SPIKES_DIR);
    let dir = spikes_dir.join(slug);
    if !dir.exists() {
        return Err(SdlcError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("spike '{slug}' not found"),
        )));
    }

    let state = read_state(root, slug);

    let findings_path = spike_findings_path(root, slug);
    let (parsed, raw) = if findings_path.exists() {
        let raw = std::fs::read_to_string(&findings_path)?;
        let p = parse_findings(&raw);
        (p, raw)
    } else {
        (
            ParsedFindings {
                title: None,
                verdict: None,
                date: None,
                the_question: None,
                open_questions: None,
            },
            String::new(),
        )
    };

    Ok((
        SpikeEntry {
            slug: slug.to_string(),
            title: parsed.title.unwrap_or_else(|| slug.to_string()),
            verdict: parsed.verdict,
            date: parsed.date,
            the_question: parsed.the_question,
            ponder_slug: state.ponder_slug,
            knowledge_slug: state.knowledge_slug,
        },
        raw,
    ))
}

/// Create a ponder entry pre-seeded with spike findings and open questions.
///
/// The ponder slug defaults to the spike slug unless `ponder_slug_override`
/// is provided. Records the ponder slug back into `state.yaml`.
///
/// Returns the ponder slug.
pub fn promote_to_ponder(
    root: &Path,
    slug: &str,
    ponder_slug_override: Option<&str>,
) -> Result<String> {
    crate::paths::validate_slug(slug)?;
    let (entry, findings_raw) = load(root, slug)?;

    let ponder_slug = ponder_slug_override.unwrap_or(&entry.slug).to_string();

    let title = entry
        .the_question
        .as_deref()
        .unwrap_or(&entry.title)
        .to_string();

    crate::ponder::PonderEntry::create(root, &ponder_slug, title)?;

    crate::ponder::capture_content(root, &ponder_slug, "spike-findings.md", &findings_raw)?;

    if let Some(oq) = extract_open_questions(&findings_raw) {
        if !oq.is_empty() {
            crate::ponder::capture_content(root, &ponder_slug, "open-questions.md", &oq)?;
        }
    }

    let mut state = read_state(root, slug);
    state.ponder_slug = Some(ponder_slug.clone());
    write_state(root, slug, &state)?;

    Ok(ponder_slug)
}

/// File a REJECT spike's findings into the knowledge base.
///
/// Idempotent — if `knowledge_slug` is already set in `state.yaml`, returns
/// it without creating a duplicate entry.
///
/// Returns the knowledge slug.
pub fn store_in_knowledge(root: &Path, slug: &str) -> Result<String> {
    crate::paths::validate_slug(slug)?;
    let mut state = read_state(root, slug);

    // Idempotency check
    if let Some(ref ks) = state.knowledge_slug {
        return Ok(ks.clone());
    }

    let findings_path = spike_findings_path(root, slug);
    let raw = if findings_path.exists() {
        std::fs::read_to_string(&findings_path)?
    } else {
        String::new()
    };
    let parsed = parse_findings(&raw);

    let title = parsed.title.unwrap_or_else(|| format!("Spike: {slug}"));
    let knowledge_slug = format!("spike-{slug}");

    // Create knowledge entry (code "900" = investigations/decisions)
    match crate::knowledge::create(root, &knowledge_slug, &title, "900") {
        Ok(_) => {}
        Err(SdlcError::KnowledgeExists(_)) => {
            // Already exists (e.g. from a previous partial run) — continue
        }
        Err(e) => return Err(e),
    }

    if !raw.is_empty() {
        crate::knowledge::append_content(root, &knowledge_slug, &raw)?;
    }

    crate::knowledge::update(
        root,
        &knowledge_slug,
        None,
        None,
        None,
        None,
        &["spike".to_string(), "rejected".to_string()],
        &[],
    )?;

    state.knowledge_slug = Some(knowledge_slug.clone());
    write_state(root, slug, &state)?;

    Ok(knowledge_slug)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_findings(verdict: &str, date: &str, has_question: bool, has_oq: bool) -> String {
        let mut s = format!("# Spike: Test Spike\n**Verdict:** {verdict}\n**Date:** {date}\n\n");
        if has_question {
            s.push_str("## The Question\nCan we test this?\n\n");
        }
        s.push_str("## Success Criteria\nIt works.\n\n");
        if has_oq {
            s.push_str("## Risks and Open Questions\n- First question\n- Second question\n");
        }
        s
    }

    fn write_spike(tmp: &TempDir, slug: &str, findings: &str) {
        let dir = tmp.path().join(".sdlc/spikes").join(slug);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("findings.md"), findings).unwrap();
    }

    #[test]
    fn parse_adopt_verdict() {
        let f = make_findings("ADOPT", "2026-03-04", false, false);
        let p = parse_findings(&f);
        assert_eq!(p.verdict, Some(SpikeVerdict::Adopt));
    }

    #[test]
    fn parse_adapt_verdict() {
        let f = make_findings("ADAPT", "2026-03-04", false, false);
        let p = parse_findings(&f);
        assert_eq!(p.verdict, Some(SpikeVerdict::Adapt));
    }

    #[test]
    fn parse_reject_verdict() {
        let f = make_findings("REJECT", "2026-03-04", false, false);
        let p = parse_findings(&f);
        assert_eq!(p.verdict, Some(SpikeVerdict::Reject));
    }

    #[test]
    fn parse_no_verdict() {
        let f = "# Spike: No Verdict\n\n## The Question\nSomething.\n";
        let p = parse_findings(f);
        assert_eq!(p.verdict, None);
    }

    #[test]
    fn parse_title() {
        let f = make_findings("ADOPT", "2026-03-04", false, false);
        let p = parse_findings(&f);
        assert_eq!(p.title.as_deref(), Some("Test Spike"));
    }

    #[test]
    fn parse_date() {
        let f = make_findings("ADOPT", "2026-03-04", false, false);
        let p = parse_findings(&f);
        assert_eq!(p.date.as_deref(), Some("2026-03-04"));
    }

    #[test]
    fn parse_the_question() {
        let f = make_findings("ADOPT", "2026-03-04", true, false);
        let p = parse_findings(&f);
        assert_eq!(p.the_question.as_deref(), Some("Can we test this?"));
    }

    #[test]
    fn parse_open_questions() {
        let f = make_findings("REJECT", "2026-03-04", false, true);
        let p = parse_findings(&f);
        let oq = p.open_questions.unwrap();
        assert!(oq.contains("First question"));
        assert!(oq.contains("Second question"));
    }

    #[test]
    fn parse_missing_sections() {
        let f = "# Spike: Bare\n**Verdict:** ADOPT\n**Date:** 2026-01-01\n";
        let p = parse_findings(f);
        assert!(p.the_question.is_none());
        assert!(p.open_questions.is_none());
    }

    #[test]
    fn extract_open_questions_works() {
        let f = make_findings("REJECT", "2026-03-04", false, true);
        let oq = extract_open_questions(&f).unwrap();
        assert!(oq.contains("First question"));
    }

    #[test]
    fn extract_open_questions_missing() {
        let f = make_findings("ADOPT", "2026-03-04", true, false);
        assert!(extract_open_questions(&f).is_none());
    }

    #[test]
    fn list_empty_dir() {
        let tmp = TempDir::new().unwrap();
        let entries = list(tmp.path()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn list_absent_findings() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".sdlc/spikes/no-findings");
        std::fs::create_dir_all(&dir).unwrap();
        let entries = list(tmp.path()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].verdict, None);
        assert_eq!(entries[0].title, "no-findings");
    }

    #[test]
    fn list_sorts_by_date() {
        let tmp = TempDir::new().unwrap();
        write_spike(
            &tmp,
            "older",
            &make_findings("ADOPT", "2026-01-01", false, false),
        );
        write_spike(
            &tmp,
            "newer",
            &make_findings("ADOPT", "2026-03-01", false, false),
        );
        let entries = list(tmp.path()).unwrap();
        assert_eq!(entries[0].slug, "newer");
        assert_eq!(entries[1].slug, "older");
    }

    #[test]
    fn promote_creates_ponder() {
        let tmp = TempDir::new().unwrap();
        // Need .sdlc config dir so ponder can find root
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(
            &tmp,
            "my-spike",
            &make_findings("ADAPT", "2026-03-04", true, true),
        );

        let ponder_slug = promote_to_ponder(tmp.path(), "my-spike", None).unwrap();
        assert_eq!(ponder_slug, "my-spike");

        // Ponder dir created
        assert!(tmp.path().join(".sdlc/roadmap/my-spike").exists());

        // spike-findings.md present
        assert!(tmp
            .path()
            .join(".sdlc/roadmap/my-spike/spike-findings.md")
            .exists());

        // open-questions.md present
        let oq =
            std::fs::read_to_string(tmp.path().join(".sdlc/roadmap/my-spike/open-questions.md"))
                .unwrap();
        assert!(oq.contains("First question"));

        // state.yaml records ponder_slug
        let state = read_state(tmp.path(), "my-spike");
        assert_eq!(state.ponder_slug.as_deref(), Some("my-spike"));
    }

    #[test]
    fn promote_with_override_slug() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(
            &tmp,
            "spike-a",
            &make_findings("ADAPT", "2026-03-04", false, false),
        );

        let ponder_slug = promote_to_ponder(tmp.path(), "spike-a", Some("custom-ponder")).unwrap();
        assert_eq!(ponder_slug, "custom-ponder");

        let state = read_state(tmp.path(), "spike-a");
        assert_eq!(state.ponder_slug.as_deref(), Some("custom-ponder"));
    }

    #[test]
    fn store_in_knowledge_creates_entry() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(
            &tmp,
            "rej-spike",
            &make_findings("REJECT", "2026-03-04", false, true),
        );

        let ks = store_in_knowledge(tmp.path(), "rej-spike").unwrap();
        assert_eq!(ks, "spike-rej-spike");

        // Knowledge entry directory created
        assert!(tmp.path().join(".sdlc/knowledge/spike-rej-spike").exists());

        // state.yaml records knowledge_slug
        let state = read_state(tmp.path(), "rej-spike");
        assert_eq!(state.knowledge_slug.as_deref(), Some("spike-rej-spike"));
    }

    #[test]
    fn store_in_knowledge_idempotent() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".sdlc")).unwrap();
        write_spike(
            &tmp,
            "rej2",
            &make_findings("REJECT", "2026-03-04", false, false),
        );

        let ks1 = store_in_knowledge(tmp.path(), "rej2").unwrap();
        let ks2 = store_in_knowledge(tmp.path(), "rej2").unwrap();
        assert_eq!(ks1, ks2);

        // Only one knowledge dir
        let count = std::fs::read_dir(tmp.path().join(".sdlc/knowledge"))
            .unwrap()
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn state_yaml_default_on_absent() {
        let tmp = TempDir::new().unwrap();
        let state = read_state(tmp.path(), "nonexistent");
        assert!(state.ponder_slug.is_none());
        assert!(state.knowledge_slug.is_none());
    }

    #[test]
    fn state_yaml_malformed_fallback() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join(".sdlc/spikes/bad");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("state.yaml"), "not: valid: yaml: :::").unwrap();
        // Should not panic — returns default
        let state = read_state(tmp.path(), "bad");
        assert!(state.ponder_slug.is_none());
    }
}
