use crate::config::Config;
use crate::error::Result;

// ---------------------------------------------------------------------------
// Config migration (no-op, kept for forward-compatibility)
// ---------------------------------------------------------------------------

/// Run any pending schema migrations on a loaded [`Config`].
pub fn migrate_config(cfg: Config) -> Result<Config> {
    Ok(cfg)
}

// ---------------------------------------------------------------------------
// Feature schema migrations
// ---------------------------------------------------------------------------

/// The current schema version for feature manifests.
///
/// Increment this constant and add a migration arm to `migrate_feature` when
/// the Feature YAML schema changes in a backward-incompatible way.
///
/// Version history:
///   0 – unversioned (original schema, no `schema_version` field)
///   1 – reserved (no distinct v1 was ever explicitly stamped)
///   2 – added `blockers`, `dependencies`, `tasks`, `archived`, `phase_history`;
///       normalised `artifacts: {}` → `artifacts: []`
///   3 – backfill missing artifact entries (review, audit, qa_results) with
///       status `missing` so mark_artifact_draft can find them
pub const FEATURE_SCHEMA_VERSION: u32 = 3;

/// Migrate a raw `serde_yaml::Value` representing a feature manifest to
/// [`FEATURE_SCHEMA_VERSION`].
///
/// Returns `Ok(true)` if any migration was applied (caller should rewrite the
/// file), `Ok(false)` if the value was already at the current version.
///
/// Returns `Err(String)` only when the value is structurally broken in a way
/// that prevents safe migration (e.g. it is not a YAML mapping at all).
pub fn migrate_feature(value: &mut serde_yaml::Value) -> std::result::Result<bool, String> {
    let version = schema_version(value);
    if version >= FEATURE_SCHEMA_VERSION {
        return Ok(false);
    }

    let map = value
        .as_mapping_mut()
        .ok_or_else(|| "feature manifest is not a YAML mapping".to_string())?;

    if version < 2 {
        // v0/v1 → v2 -----------------------------------------------------------
        // Fields that were added after the initial schema but have safe empty defaults.
        insert_seq_if_missing(map, "blockers");
        insert_seq_if_missing(map, "dependencies");
        insert_seq_if_missing(map, "tasks");
        insert_bool_if_missing(map, "archived", false);

        // phase_history: synthesise a single entry from the existing `phase` and
        // `created_at` fields so the struct constraint is satisfied.
        let ph_key = serde_yaml::Value::String("phase_history".to_owned());
        if !map.contains_key(&ph_key) {
            let phase = map
                .get("phase")
                .cloned()
                .unwrap_or_else(|| serde_yaml::Value::String("draft".to_owned()));
            let entered = map
                .get("created_at")
                .cloned()
                .unwrap_or_else(|| serde_yaml::Value::String(chrono::Utc::now().to_rfc3339()));

            let mut transition = serde_yaml::Mapping::new();
            transition.insert("phase".into(), phase);
            transition.insert("entered".into(), entered);
            transition.insert("exited".into(), serde_yaml::Value::Null);

            map.insert(
                ph_key,
                serde_yaml::Value::Sequence(vec![serde_yaml::Value::Mapping(transition)]),
            );
        }

        // artifacts: {} → artifacts: []  (also handled by the custom deserialiser
        // on the struct, but normalise here so the rewritten file is canonical)
        let ak = serde_yaml::Value::String("artifacts".to_owned());
        if map.get(&ak).map(|v| v.is_mapping()).unwrap_or(false) {
            map.insert(ak, serde_yaml::Value::Sequence(vec![]));
        }
    }

    // v2 → v3 -----------------------------------------------------------------
    // Backfill any artifact entries that are missing from the artifacts list.
    // Older features may have been created before all 7 artifact types were
    // tracked in default_artifacts(), leaving the state machine unable to call
    // mark_artifact_draft() on the missing types. Add them with status `missing`.
    {
        // Read slug for path construction (best-effort; skip if absent).
        let slug = map
            .get("slug")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let ak = serde_yaml::Value::String("artifacts".to_owned());
        // Ensure artifacts is a sequence (v2 migration above handles {}→[], but
        // we may arrive here with version == 2 where it was already a sequence).
        if map.get(&ak).map(|v| v.is_mapping()).unwrap_or(false) {
            map.insert(ak.clone(), serde_yaml::Value::Sequence(vec![]));
        }

        if let Some(serde_yaml::Value::Sequence(artifacts)) = map.get_mut(&ak) {
            // Collect already-present artifact types.
            let present: std::collections::HashSet<String> = artifacts
                .iter()
                .filter_map(|a| {
                    a.get("artifact_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect();

            // The canonical set of artifact types and their filenames.
            let all_artifact_types: &[(&str, &str)] = &[
                ("spec", "spec.md"),
                ("design", "design.md"),
                ("tasks", "tasks.md"),
                ("qa_plan", "qa-plan.md"),
                ("review", "review.md"),
                ("audit", "audit.md"),
                ("qa_results", "qa-results.md"),
            ];

            for (type_str, filename) in all_artifact_types {
                if !present.contains(*type_str) {
                    let path = if slug.is_empty() {
                        format!(".sdlc/features/<slug>/{filename}")
                    } else {
                        format!(".sdlc/features/{slug}/{filename}")
                    };
                    let mut entry = serde_yaml::Mapping::new();
                    entry.insert(
                        "artifact_type".into(),
                        serde_yaml::Value::String(type_str.to_string()),
                    );
                    entry.insert(
                        "status".into(),
                        serde_yaml::Value::String("missing".to_string()),
                    );
                    entry.insert("path".into(), serde_yaml::Value::String(path));
                    entry.insert("created_at".into(), serde_yaml::Value::Null);
                    entry.insert("approved_at".into(), serde_yaml::Value::Null);
                    entry.insert("rejected_at".into(), serde_yaml::Value::Null);
                    entry.insert("rejection_reason".into(), serde_yaml::Value::Null);
                    entry.insert("approved_by".into(), serde_yaml::Value::Null);
                    artifacts.push(serde_yaml::Value::Mapping(entry));
                }
            }
        }
    }

    // Stamp the version so subsequent loads skip migration entirely.
    map.insert(
        "schema_version".into(),
        serde_yaml::Value::Number(serde_yaml::Number::from(FEATURE_SCHEMA_VERSION)),
    );

    Ok(true)
}

/// Parse a `serde_yaml` deserialisation error and return a human-readable,
/// actionable fix hint for the caller to surface to the user.
pub fn feature_fix_hint(err: &serde_yaml::Error) -> String {
    let msg = err.to_string();

    if let Some(field) = extract_missing_field(&msg) {
        let yaml_fix = match field {
            "blockers" | "dependencies" | "tasks" => format!("{field}: []"),
            "phase_history" => "phase_history:\n  - phase: draft\n    entered: <created_at timestamp>\n    exited: null".to_owned(),
            "archived" => "archived: false".to_owned(),
            "slug" | "title" | "phase" | "created_at" | "updated_at" => {
                return format!(
                    "required field `{field}` is missing — the file may be corrupted. \
                     Restore from git or recreate the feature."
                );
            }
            _ => format!("{field}: <value>"),
        };
        format!(
            "add the following to .sdlc/features/<slug>/manifest.yaml:\n  {yaml_fix}\n\
             Or run `sdlc doctor --fix` to auto-repair."
        )
    } else {
        "Run `sdlc doctor --fix` to attempt auto-repair, or inspect the file manually.".to_owned()
    }
}

// ---------------------------------------------------------------------------
// Milestone schema migrations
// ---------------------------------------------------------------------------

/// The current schema version for milestone manifests.
///
/// Version history:
///   0 – unversioned (original schema, no `schema_version` field)
///   1 – guaranteed `features: []` present; `schema_version` stamped
pub const MILESTONE_SCHEMA_VERSION: u32 = 1;

/// Migrate a raw `serde_yaml::Value` representing a milestone manifest to
/// [`MILESTONE_SCHEMA_VERSION`].
///
/// Returns `Ok(true)` if migration ran (caller should rewrite the file),
/// `Ok(false)` if already current.
pub fn migrate_milestone(value: &mut serde_yaml::Value) -> std::result::Result<bool, String> {
    let version = schema_version(value);
    if version >= MILESTONE_SCHEMA_VERSION {
        return Ok(false);
    }

    let map = value
        .as_mapping_mut()
        .ok_or_else(|| "milestone manifest is not a YAML mapping".to_string())?;

    // v0 → v1: ensure features list is present.
    insert_seq_if_missing(map, "features");

    map.insert(
        "schema_version".into(),
        serde_yaml::Value::Number(serde_yaml::Number::from(MILESTONE_SCHEMA_VERSION)),
    );

    Ok(true)
}

/// Actionable fix hint for a milestone manifest deserialization error.
pub fn milestone_fix_hint(err: &serde_yaml::Error) -> String {
    let msg = err.to_string();
    if let Some(field) = extract_missing_field(&msg) {
        match field {
            "features" => {
                "add the following to .sdlc/milestones/<slug>/manifest.yaml:\n  features: []\n\
                 Or run `sdlc doctor --fix` to auto-repair."
                    .to_owned()
            }
            "slug" | "title" | "created_at" | "updated_at" => format!(
                "required field `{field}` is missing — the file may be corrupted. \
                 Restore from git or recreate the milestone."
            ),
            _ => format!(
                "add the following to .sdlc/milestones/<slug>/manifest.yaml:\n  {field}: <value>\n\
                 Or run `sdlc doctor --fix` to auto-repair."
            ),
        }
    } else {
        "Run `sdlc doctor --fix` to attempt auto-repair, or inspect the file manually.".to_owned()
    }
}

// ---------------------------------------------------------------------------
// State schema helpers
// ---------------------------------------------------------------------------

/// Actionable fix hint for a state.yaml deserialization error.
///
/// State is a project singleton managed entirely by the tool, so migration
/// is not needed — just helpful error messages for hand-edited or corrupted files.
pub fn state_fix_hint(err: &serde_yaml::Error) -> String {
    let msg = err.to_string();
    if let Some(field) = extract_missing_field(&msg) {
        match field {
            "active_features" | "active_directives" | "history" | "blocked" | "milestones"
            | "active_ponders" => format!(
                "add the following to .sdlc/state.yaml:\n  {field}: []\n\
                 Or run `sdlc init` to reinitialize the project state."
            ),
            "project" | "last_updated" => format!(
                "required field `{field}` is missing from .sdlc/state.yaml — \
                 run `sdlc init` to reinitialize the project state."
            ),
            _ => "Run `sdlc init` to reinitialize the project state, \
                  or inspect .sdlc/state.yaml manually."
                .to_owned(),
        }
    } else {
        "Run `sdlc init` to reinitialize the project state, \
         or inspect .sdlc/state.yaml manually."
            .to_owned()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Read `schema_version` from a raw Value, defaulting to 0 (unversioned).
fn schema_version(value: &serde_yaml::Value) -> u32 {
    value
        .get("schema_version")
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as u32
}

fn insert_seq_if_missing(map: &mut serde_yaml::Mapping, key: &str) {
    let k = serde_yaml::Value::String(key.to_owned());
    if !map.contains_key(&k) {
        map.insert(k, serde_yaml::Value::Sequence(vec![]));
    }
}

fn insert_bool_if_missing(map: &mut serde_yaml::Mapping, key: &str, val: bool) {
    let k = serde_yaml::Value::String(key.to_owned());
    if !map.contains_key(&k) {
        map.insert(k, serde_yaml::Value::Bool(val));
    }
}

/// Extract the field name from a serde "missing field `foo`" error message.
fn extract_missing_field(msg: &str) -> Option<&str> {
    let prefix = "missing field `";
    let start = msg.find(prefix)? + prefix.len();
    let end = msg[start..].find('`')?;
    Some(&msg[start..start + end])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_minimal_v0() -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
slug: test-feature
title: Test Feature
phase: draft
created_at: "2026-01-01T00:00:00Z"
updated_at: "2026-01-01T00:00:00Z"
artifacts: []
comments: []
next_comment_seq: 0
scores: []
"#,
        )
        .unwrap()
    }

    #[test]
    fn migrate_adds_missing_fields() {
        let mut v = make_minimal_v0();
        let changed = migrate_feature(&mut v).unwrap();
        assert!(changed, "should report migration ran");

        let map = v.as_mapping().unwrap();
        assert!(map.contains_key("blockers"));
        assert!(map.contains_key("dependencies"));
        assert!(map.contains_key("tasks"));
        assert!(map.contains_key("archived"));
        assert!(map.contains_key("phase_history"));
        assert_eq!(
            map.get("schema_version").and_then(|v| v.as_u64()),
            Some(FEATURE_SCHEMA_VERSION as u64)
        );
    }

    #[test]
    fn migrate_normalises_artifacts_map() {
        let mut v: serde_yaml::Value = serde_yaml::from_str(
            r#"
slug: test-feature
title: Test Feature
phase: draft
created_at: "2026-01-01T00:00:00Z"
updated_at: "2026-01-01T00:00:00Z"
artifacts: {}
"#,
        )
        .unwrap();
        migrate_feature(&mut v).unwrap();
        let artifacts = &v["artifacts"];
        assert!(
            artifacts.is_sequence(),
            "artifacts should be a sequence after migration"
        );
    }

    #[test]
    fn migrate_is_noop_at_current_version() {
        let mut v = make_minimal_v0();
        // Stamp current version manually.
        v["schema_version"] =
            serde_yaml::Value::Number(serde_yaml::Number::from(FEATURE_SCHEMA_VERSION));
        let changed = migrate_feature(&mut v).unwrap();
        assert!(
            !changed,
            "should skip migration when already at current version"
        );
    }

    #[test]
    fn migrate_v2_to_v3_backfills_missing_artifacts() {
        // Build a v2 manifest that has only spec and design — simulating a feature
        // created before all 7 artifact types were tracked.
        let mut v: serde_yaml::Value = serde_yaml::from_str(
            r#"
slug: my-feature
title: My Feature
phase: implementation
created_at: "2026-01-01T00:00:00Z"
updated_at: "2026-01-01T00:00:00Z"
artifacts:
  - artifact_type: spec
    status: approved
    path: .sdlc/features/my-feature/spec.md
    created_at: null
    approved_at: null
    rejected_at: null
    rejection_reason: null
    approved_by: null
  - artifact_type: design
    status: approved
    path: .sdlc/features/my-feature/design.md
    created_at: null
    approved_at: null
    rejected_at: null
    rejection_reason: null
    approved_by: null
tasks: []
comments: []
next_comment_seq: 0
blockers: []
dependencies: []
archived: false
phase_history:
  - phase: draft
    entered: "2026-01-01T00:00:00Z"
    exited: null
schema_version: 2
"#,
        )
        .unwrap();

        let changed = migrate_feature(&mut v).unwrap();
        assert!(changed, "migration should report changes");
        assert_eq!(
            v["schema_version"].as_u64(),
            Some(FEATURE_SCHEMA_VERSION as u64)
        );

        let artifacts = v["artifacts"].as_sequence().unwrap();
        // All 7 artifact types must be present after migration.
        let all_types = [
            "spec",
            "design",
            "tasks",
            "qa_plan",
            "review",
            "audit",
            "qa_results",
        ];
        for expected_type in all_types {
            let found = artifacts.iter().any(|a| {
                a.get("artifact_type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == expected_type)
                    .unwrap_or(false)
            });
            assert!(
                found,
                "artifact type '{}' should be present after v2→v3 migration",
                expected_type
            );
        }

        // Pre-existing artifacts must not be duplicated.
        let spec_count = artifacts
            .iter()
            .filter(|a| {
                a.get("artifact_type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "spec")
                    .unwrap_or(false)
            })
            .count();
        assert_eq!(spec_count, 1, "spec must not be duplicated");

        // Backfilled entries must have status 'missing'.
        let review = artifacts
            .iter()
            .find(|a| {
                a.get("artifact_type")
                    .and_then(|v| v.as_str())
                    .map(|t| t == "review")
                    .unwrap_or(false)
            })
            .expect("review artifact must be present");
        assert_eq!(
            review.get("status").and_then(|v| v.as_str()),
            Some("missing"),
            "backfilled review must have status 'missing'"
        );
        assert_eq!(
            review.get("path").and_then(|v| v.as_str()),
            Some(".sdlc/features/my-feature/review.md"),
            "backfilled review must have correct path"
        );
    }

    #[test]
    fn migrate_synthesises_phase_history() {
        let mut v = make_minimal_v0();
        migrate_feature(&mut v).unwrap();
        let ph = &v["phase_history"];
        assert!(ph.is_sequence());
        let entries = ph.as_sequence().unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0]["phase"].as_str(), Some("draft"));
    }

    #[test]
    fn extract_missing_field_parses_serde_message() {
        let msg = "missing field `blockers` at line 1 column 1";
        assert_eq!(extract_missing_field(msg), Some("blockers"));
    }

    // ---------------------------------------------------------------------------
    // Milestone migration tests
    // ---------------------------------------------------------------------------

    fn make_minimal_milestone_v0() -> serde_yaml::Value {
        serde_yaml::from_str(
            r#"
slug: v1-launch
title: v1.0 Launch
created_at: "2026-01-01T00:00:00Z"
updated_at: "2026-01-01T00:00:00Z"
"#,
        )
        .unwrap()
    }

    #[test]
    fn milestone_migrate_inserts_features() {
        let mut v = make_minimal_milestone_v0();
        let changed = migrate_milestone(&mut v).unwrap();
        assert!(changed, "should report migration ran");

        let map = v.as_mapping().unwrap();
        assert!(map.contains_key("features"));
        assert_eq!(
            map.get("schema_version").and_then(|v| v.as_u64()),
            Some(MILESTONE_SCHEMA_VERSION as u64)
        );
    }

    #[test]
    fn milestone_migrate_is_noop_at_current_version() {
        let mut v = make_minimal_milestone_v0();
        v["schema_version"] =
            serde_yaml::Value::Number(serde_yaml::Number::from(MILESTONE_SCHEMA_VERSION));
        let changed = migrate_milestone(&mut v).unwrap();
        assert!(
            !changed,
            "should skip migration when already at current version"
        );
    }

    #[test]
    fn milestone_migrate_preserves_existing_features() {
        let mut v: serde_yaml::Value = serde_yaml::from_str(
            r#"
slug: v1-launch
title: v1.0 Launch
created_at: "2026-01-01T00:00:00Z"
updated_at: "2026-01-01T00:00:00Z"
features:
  - auth-login
  - user-profile
"#,
        )
        .unwrap();
        migrate_milestone(&mut v).unwrap();
        let features = &v["features"];
        assert!(features.is_sequence());
        assert_eq!(features.as_sequence().unwrap().len(), 2);
    }
}
