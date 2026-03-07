use crate::{
    error::{Result, SdlcError},
    feature::Feature,
    investigation::InvestigationEntry,
    milestone::{Milestone, MilestoneStatus},
    ponder::{PonderArtifactMeta, PonderEntry},
    workspace,
};
use std::path::Path;
use tantivy::{
    collector::TopDocs,
    query::QueryParser,
    schema::{Field, Schema, Value, STORED, STRING, TEXT},
    Index, IndexWriter, ReloadPolicy, TantivyDocument,
};

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Serialize)]
pub struct EntitySearchResult {
    pub kind: String,
    pub slug: String,
    pub title: String,
    pub status: String,
    pub score: f32,
}

// ---------------------------------------------------------------------------
// EntityIndex
// ---------------------------------------------------------------------------

struct EntityFields {
    kind: Field,
    slug: Field,
    title: Field,
    status: Field,
    description: Field,
    body: Field,
}

pub struct EntitySources<'a> {
    pub features: &'a [Feature],
    pub ponders: &'a [(PonderEntry, Vec<PonderArtifactMeta>)],
    pub milestones: &'a [(Milestone, MilestoneStatus)],
    pub investigations: &'a [(InvestigationEntry, Vec<workspace::ArtifactMeta>)],
    pub root: &'a Path,
}

pub struct EntityIndex {
    index: Index,
    reader: tantivy::IndexReader,
    fields: EntityFields,
}

impl EntityIndex {
    /// Build an ephemeral in-RAM index from all entity types.
    ///
    /// Indexed fields:
    /// - `kind`        — STRING (exact-match, stored) — "feature" | "ponder" | "milestone" | "investigation"
    /// - `slug`        — STRING (exact-match, stored)
    /// - `title`       — TEXT (tokenized, stored)
    /// - `status`      — STRING (exact-match, stored)
    /// - `description` — TEXT (tokenized, not stored)
    /// - `body`        — TEXT (tokenized, not stored)
    pub fn build(sources: EntitySources<'_>) -> Result<Self> {
        let (schema, fields) = build_entity_schema();
        let index = Index::create_in_ram(schema);
        let mut writer: IndexWriter = index
            .writer(15_000_000)
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        // -- Features --
        for f in sources.features {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.kind, "feature");
            doc.add_text(fields.slug, &f.slug);
            doc.add_text(fields.title, &f.title);
            doc.add_text(fields.status, f.phase.to_string());
            if let Some(desc) = &f.description {
                doc.add_text(fields.description, desc);
            }

            let task_titles: String = f
                .tasks
                .iter()
                .map(|t| t.title.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let mut body_parts: Vec<&str> = std::iter::once(f.slug.as_str())
                .chain(f.comments.iter().map(|c| c.body.as_str()))
                .chain(f.blockers.iter().map(String::as_str))
                .chain(std::iter::once(task_titles.as_str()))
                .filter(|s| !s.is_empty())
                .collect();

            const ARTIFACT_FILES: &[&str] = &[
                "spec.md",
                "design.md",
                "tasks.md",
                "qa_plan.md",
                "review.md",
                "audit.md",
                "qa_results.md",
            ];
            let mut artifact_contents: Vec<String> = Vec::new();
            for filename in ARTIFACT_FILES {
                let path = sources.root.join("features").join(&f.slug).join(filename);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let end = content.floor_char_boundary(8000);
                    artifact_contents.push(content[..end].to_string());
                }
            }
            for content in &artifact_contents {
                body_parts.push(content.as_str());
            }

            doc.add_text(fields.body, body_parts.join(" "));
            writer
                .add_document(doc)
                .map_err(|e| SdlcError::Search(e.to_string()))?;
        }

        // -- Ponders --
        for (entry, artifacts) in sources.ponders {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.kind, "ponder");
            doc.add_text(fields.slug, &entry.slug);
            doc.add_text(fields.title, &entry.title);
            doc.add_text(fields.status, entry.status.to_string());

            let mut body_parts: Vec<String> = Vec::new();
            body_parts.push(entry.tags.join(" "));
            body_parts.push(entry.slug.replace('-', " "));

            for artifact in artifacts {
                let path =
                    crate::paths::ponder_dir(sources.root, &entry.slug).join(&artifact.filename);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let end = content.floor_char_boundary(8000);
                    body_parts.push(content[..end].to_string());
                }
            }

            doc.add_text(fields.body, body_parts.join(" "));
            writer
                .add_document(doc)
                .map_err(|e| SdlcError::Search(e.to_string()))?;
        }

        // -- Milestones --
        for (m, status) in sources.milestones {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.kind, "milestone");
            doc.add_text(fields.slug, &m.slug);
            doc.add_text(fields.title, &m.title);
            doc.add_text(fields.status, status.to_string());
            if let Some(desc) = &m.description {
                doc.add_text(fields.description, desc);
            }

            let mut body_parts: Vec<String> = Vec::new();
            body_parts.push(m.slug.replace('-', " "));
            if let Some(vision) = &m.vision {
                body_parts.push(vision.clone());
            }
            body_parts.push(m.features.join(" "));

            // acceptance_test.md
            if let Ok(Some(at)) = m.load_acceptance_test(sources.root) {
                let end = at.floor_char_boundary(8000);
                body_parts.push(at[..end].to_string());
            }

            doc.add_text(fields.body, body_parts.join(" "));
            writer
                .add_document(doc)
                .map_err(|e| SdlcError::Search(e.to_string()))?;
        }

        // -- Investigations --
        for (entry, artifacts) in sources.investigations {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.kind, "investigation");
            doc.add_text(fields.slug, &entry.slug);
            doc.add_text(fields.title, &entry.title);
            doc.add_text(fields.status, entry.status.to_string());
            if let Some(ctx) = &entry.context {
                doc.add_text(fields.description, ctx);
            }

            let mut body_parts: Vec<String> = Vec::new();
            body_parts.push(entry.slug.replace('-', " "));
            body_parts.push(entry.kind.to_string());

            for artifact in artifacts {
                let path = crate::paths::investigation_dir(sources.root, &entry.slug)
                    .join(&artifact.filename);
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let end = content.floor_char_boundary(8000);
                    body_parts.push(content[..end].to_string());
                }
            }

            doc.add_text(fields.body, body_parts.join(" "));
            writer
                .add_document(doc)
                .map_err(|e| SdlcError::Search(e.to_string()))?;
        }

        writer
            .commit()
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e: tantivy::TantivyError| SdlcError::Search(e.to_string()))?;

        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    /// BM25 full-text search. Returns up to `limit` results sorted by score descending.
    ///
    /// Supported query syntax:
    /// - Bare terms: `auth login`          (AND by default)
    /// - Phrase:     `"exact phrase"`
    /// - Boolean:    `auth OR oauth`, `auth NOT legacy`
    /// - Field scope: `status:ready`, `slug:auth`, `title:oauth`, `kind:milestone`
    /// - Prefix:     `auth*`
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<EntitySearchResult>> {
        let searcher = self.reader.searcher();

        let default_fields = vec![self.fields.title, self.fields.description, self.fields.body];
        let mut parser = QueryParser::for_index(&self.index, default_fields);
        parser.set_conjunction_by_default();

        let query = match parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(vec![]),
        };

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_addr) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_addr)
                .map_err(|e| SdlcError::Search(e.to_string()))?;

            let kind = doc
                .get_first(self.fields.kind)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let slug = doc
                .get_first(self.fields.slug)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = doc
                .get_first(self.fields.status)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(EntitySearchResult {
                kind,
                slug,
                title,
                status,
                score,
            });
        }

        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// TaskIndex (unchanged — separate endpoint)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct TaskSearchResult {
    pub feature_slug: String,
    pub task_id: String,
    pub title: String,
    pub status: String,
    pub score: f32,
}

struct TaskFields {
    feature_slug: Field,
    task_id: Field,
    title: Field,
    status: Field,
    description: Field,
    body: Field,
}

pub struct TaskIndex {
    index: Index,
    reader: tantivy::IndexReader,
    fields: TaskFields,
}

impl TaskIndex {
    /// Build an ephemeral in-RAM index from the given feature slice.
    pub fn build(features: &[Feature]) -> Result<Self> {
        let (schema, fields) = build_task_schema();

        let index = Index::create_in_ram(schema);

        let mut writer: IndexWriter = index
            .writer(15_000_000)
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        for feature in features {
            for task in &feature.tasks {
                let mut doc = TantivyDocument::default();
                doc.add_text(fields.feature_slug, &feature.slug);
                doc.add_text(fields.task_id, &task.id);
                doc.add_text(fields.title, &task.title);
                doc.add_text(fields.status, task.status.to_string());
                if let Some(desc) = &task.description {
                    doc.add_text(fields.description, desc);
                }
                let depends_str = task.depends_on.join(" ");
                let body_parts: Vec<&str> = [
                    feature.slug.as_str(),
                    task.blocker.as_deref().unwrap_or(""),
                    depends_str.as_str(),
                ]
                .iter()
                .filter(|s| !s.is_empty())
                .cloned()
                .collect();
                doc.add_text(fields.body, body_parts.join(" "));

                writer
                    .add_document(doc)
                    .map_err(|e| SdlcError::Search(e.to_string()))?;
            }
        }

        writer
            .commit()
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        let reader = index
            .reader_builder()
            .reload_policy(ReloadPolicy::Manual)
            .try_into()
            .map_err(|e: tantivy::TantivyError| SdlcError::Search(e.to_string()))?;

        Ok(Self {
            index,
            reader,
            fields,
        })
    }

    /// BM25 full-text search. Returns up to `limit` results sorted by score descending.
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<TaskSearchResult>> {
        let searcher = self.reader.searcher();

        let default_fields = vec![self.fields.title, self.fields.description, self.fields.body];
        let mut parser = QueryParser::for_index(&self.index, default_fields);
        parser.set_conjunction_by_default();

        let query = match parser.parse_query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(vec![]),
        };

        let top_docs = searcher
            .search(&query, &TopDocs::with_limit(limit))
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        let mut results = Vec::with_capacity(top_docs.len());
        for (score, doc_addr) in top_docs {
            let doc: TantivyDocument = searcher
                .doc(doc_addr)
                .map_err(|e| SdlcError::Search(e.to_string()))?;

            let feature_slug = doc
                .get_first(self.fields.feature_slug)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let task_id = doc
                .get_first(self.fields.task_id)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let title = doc
                .get_first(self.fields.title)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let status = doc
                .get_first(self.fields.status)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(TaskSearchResult {
                feature_slug,
                task_id,
                title,
                status,
                score,
            });
        }

        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// Schema construction
// ---------------------------------------------------------------------------

fn build_entity_schema() -> (Schema, EntityFields) {
    let mut builder = Schema::builder();

    let kind = builder.add_text_field("kind", STRING | STORED);
    let slug = builder.add_text_field("slug", STRING | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let status = builder.add_text_field("status", STRING | STORED);
    let description = builder.add_text_field("description", TEXT);
    let body = builder.add_text_field("body", TEXT);

    let schema = builder.build();
    let fields = EntityFields {
        kind,
        slug,
        title,
        status,
        description,
        body,
    };
    (schema, fields)
}

fn build_task_schema() -> (Schema, TaskFields) {
    let mut builder = Schema::builder();

    let feature_slug = builder.add_text_field("feature_slug", STRING | STORED);
    let task_id = builder.add_text_field("task_id", STRING | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let status = builder.add_text_field("status", STRING | STORED);
    let description = builder.add_text_field("description", TEXT);
    let body = builder.add_text_field("body", TEXT);

    let schema = builder.build();
    let fields = TaskFields {
        feature_slug,
        task_id,
        title,
        status,
        description,
        body,
    };
    (schema, fields)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::Feature;

    fn make_features() -> Vec<Feature> {
        vec![
            Feature::with_description(
                "auth-login",
                "User Authentication",
                Some("OAuth login with Google and GitHub".to_string()),
            ),
            Feature::with_description(
                "payment-flow",
                "Payment Processing",
                Some("Stripe checkout integration".to_string()),
            ),
            Feature::new("search-ui", "Search Interface"),
        ]
    }

    fn empty_sources<'a>(features: &'a [Feature], root: &'a Path) -> EntitySources<'a> {
        EntitySources {
            features,
            ponders: &[],
            milestones: &[],
            investigations: &[],
            root,
        }
    }

    #[test]
    fn search_by_title_word() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("authentication", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "auth-login");
        assert_eq!(results[0].kind, "feature");
    }

    #[test]
    fn search_by_description_word() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("stripe", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "payment-flow");
    }

    #[test]
    fn search_no_match_returns_empty() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("kubernetes", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_respects_limit() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("search OR auth OR payment", 2).unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn search_status_field_scoped() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        // All features are Draft by default
        let results = index.search("status:draft", 10).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn search_kind_field_scoped() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let m = Milestone::new("v1-launch", "V1 Launch");
        let sources = EntitySources {
            features: &features,
            ponders: &[],
            milestones: &[(m, MilestoneStatus::Active)],
            investigations: &[],
            root: dir.path(),
        };
        let index = EntityIndex::build(sources).unwrap();
        let results = index.search("kind:milestone", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "v1-launch");
        assert_eq!(results[0].kind, "milestone");
    }

    #[test]
    fn search_empty_index() {
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&[], dir.path())).unwrap();
        let results = index.search("anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_scores_descending() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("search OR auth OR payment", 10).unwrap();
        let scores: Vec<f32> = results.iter().map(|r| r.score).collect();
        let sorted = {
            let mut s = scores.clone();
            s.sort_by(|a, b| b.partial_cmp(a).unwrap());
            s
        };
        assert_eq!(
            scores, sorted,
            "results should be sorted by score descending"
        );
    }

    #[test]
    fn search_finds_milestone_by_title() {
        let dir = tempfile::TempDir::new().unwrap();
        let m = Milestone::new("v46-realtime", "V46 Realtime Activity Feed");
        let sources = EntitySources {
            features: &[],
            ponders: &[],
            milestones: &[(m, MilestoneStatus::Active)],
            investigations: &[],
            root: dir.path(),
        };
        let index = EntityIndex::build(sources).unwrap();
        let results = index.search("realtime", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "v46-realtime");
        assert_eq!(results[0].kind, "milestone");
        assert_eq!(results[0].status, "active");
    }

    #[test]
    fn search_finds_milestone_by_vision() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut m = Milestone::new("v2-billing", "Billing Overhaul");
        m.vision = Some("Users can manage subscriptions self-serve".to_string());
        let sources = EntitySources {
            features: &[],
            ponders: &[],
            milestones: &[(m, MilestoneStatus::Verifying)],
            investigations: &[],
            root: dir.path(),
        };
        let index = EntityIndex::build(sources).unwrap();
        let results = index.search("subscriptions", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "v2-billing");
        assert_eq!(results[0].status, "verifying");
    }

    #[test]
    fn search_finds_investigation_by_title() {
        let dir = tempfile::TempDir::new().unwrap();
        let entry = InvestigationEntry {
            slug: "auth-leak".to_string(),
            title: "Authentication Token Leak".to_string(),
            kind: crate::investigation::InvestigationKind::RootCause,
            phase: "triage".to_string(),
            status: crate::investigation::InvestigationStatus::InProgress,
            context: Some("Tokens appearing in logs".to_string()),
            orientation: None,
            sessions: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            confidence: None,
            output_type: None,
            output_ref: None,
            scope: None,
            lens_scores: None,
            output_refs: vec![],
            guideline_scope: None,
            problem_statement: None,
            evidence_counts: None,
            principles_count: None,
            publish_path: None,
        };
        let sources = EntitySources {
            features: &[],
            ponders: &[],
            milestones: &[],
            investigations: &[(entry, vec![])],
            root: dir.path(),
        };
        let index = EntityIndex::build(sources).unwrap();
        let results = index.search("authentication token", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "auth-leak");
        assert_eq!(results[0].kind, "investigation");
    }

    #[test]
    fn search_cross_entity_ranking() {
        let dir = tempfile::TempDir::new().unwrap();
        let features = vec![Feature::new("auth-feature", "Auth Feature")];
        let m = Milestone::new("auth-milestone", "Auth Milestone");
        let sources = EntitySources {
            features: &features,
            ponders: &[],
            milestones: &[(m, MilestoneStatus::Active)],
            investigations: &[],
            root: dir.path(),
        };
        let index = EntityIndex::build(sources).unwrap();
        let results = index.search("auth", 10).unwrap();
        assert_eq!(results.len(), 2);
        // Both should be present (order depends on BM25)
        let kinds: Vec<&str> = results.iter().map(|r| r.kind.as_str()).collect();
        assert!(kinds.contains(&"feature"));
        assert!(kinds.contains(&"milestone"));
    }

    // -----------------------------------------------------------------------
    // TaskIndex tests
    // -----------------------------------------------------------------------

    fn make_feature_with_tasks() -> Feature {
        use crate::task::add_task;
        let mut f = Feature::with_description("auth-feature", "User Authentication", None);
        add_task(&mut f.tasks, "Implement OAuth flow");
        add_task(&mut f.tasks, "Write login form");
        add_task(&mut f.tasks, "Set up database schema");
        f
    }

    #[test]
    fn task_search_by_title_word() {
        let feature = make_feature_with_tasks();
        let index = TaskIndex::build(&[feature]).unwrap();
        let results = index.search("login", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Write login form");
    }

    #[test]
    fn task_search_by_description_word() {
        use crate::task::add_task;
        let mut f = Feature::new("payment", "Payment");
        let id = add_task(&mut f.tasks, "Stripe integration");
        f.tasks.iter_mut().find(|t| t.id == id).unwrap().description =
            Some("Handle checkout webhook events".to_string());
        let index = TaskIndex::build(&[f]).unwrap();
        let results = index.search("webhook", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Stripe integration");
    }

    #[test]
    fn task_search_no_match_returns_empty() {
        let feature = make_feature_with_tasks();
        let index = TaskIndex::build(&[feature]).unwrap();
        let results = index.search("kubernetes", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn task_search_respects_limit() {
        let feature = make_feature_with_tasks();
        let index = TaskIndex::build(&[feature]).unwrap();
        let results = index.search("implement OR login OR database", 2).unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn task_search_status_field_scoped() {
        use crate::task::{add_task, block_task};
        let mut f = Feature::new("api", "API Feature");
        let t1 = add_task(&mut f.tasks, "Build endpoint");
        add_task(&mut f.tasks, "Write tests");
        block_task(&mut f.tasks, &t1, "waiting for infra").unwrap();
        let index = TaskIndex::build(&[f]).unwrap();
        let results = index.search("status:blocked", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Build endpoint");
        assert_eq!(results[0].status, "blocked");
    }

    #[test]
    fn task_search_empty_index() {
        let index = TaskIndex::build(&[]).unwrap();
        let results = index.search("anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_malformed_query_returns_empty() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();
        let results = index.search("title:[unclosed", 10).unwrap();
        assert!(
            results.is_empty(),
            "malformed query should return empty results"
        );
    }

    #[test]
    fn task_search_malformed_query_returns_empty() {
        let feature = make_feature_with_tasks();
        let index = TaskIndex::build(&[feature]).unwrap();
        let results = index.search("title:[unclosed", 10).unwrap();
        assert!(
            results.is_empty(),
            "malformed query should return empty results"
        );
    }

    #[test]
    fn task_search_scores_descending() {
        let feature = make_feature_with_tasks();
        let index = TaskIndex::build(&[feature]).unwrap();
        let results = index.search("implement OR login OR database", 10).unwrap();
        let scores: Vec<f32> = results.iter().map(|r| r.score).collect();
        let sorted = {
            let mut s = scores.clone();
            s.sort_by(|a, b| b.partial_cmp(a).unwrap());
            s
        };
        assert_eq!(
            scores, sorted,
            "results should be sorted by score descending"
        );
    }

    #[test]
    fn search_indexes_artifact_content() {
        use std::fs;
        let dir = tempfile::TempDir::new().unwrap();
        let features = make_features();

        let feature_dir = dir.path().join("features").join("auth-login");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("spec.md"),
            "The system uses JWT tokens for stateless authentication with refresh token rotation",
        )
        .unwrap();

        let index = EntityIndex::build(empty_sources(&features, dir.path())).unwrap();

        let results = index.search("JWT", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "auth-login");
    }
}
