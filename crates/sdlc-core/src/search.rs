use crate::{
    error::{Result, SdlcError},
    feature::Feature,
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

#[derive(Debug, Clone)]
pub struct SearchResult {
    pub slug: String,
    pub title: String,
    pub phase: String,
    pub score: f32,
}

// ---------------------------------------------------------------------------
// FeatureIndex
// ---------------------------------------------------------------------------

struct Fields {
    slug: Field,
    title: Field,
    description: Field,
    phase: Field,
    body: Field,
}

pub struct FeatureIndex {
    index: Index,
    reader: tantivy::IndexReader,
    fields: Fields,
}

impl FeatureIndex {
    /// Build an ephemeral in-RAM index from the given feature slice.
    ///
    /// Indexed fields:
    /// - `slug`        — STRING (exact-match, stored) — used for field-scoped queries like `slug:auth`
    /// - `title`       — TEXT (tokenized, stored) — primary full-text field
    /// - `description` — TEXT (tokenized, not stored) — full-text
    /// - `phase`       — STRING (exact-match, stored) — allows `phase:ready` scoping
    /// - `body`        — TEXT (tokenized, not stored) — comment bodies + blocker strings + artifact content
    pub fn build(features: &[Feature], root: &Path) -> Result<Self> {
        let (schema, fields) = build_schema();

        let index = Index::create_in_ram(schema);

        // 15 MB heap is more than enough for dozens to hundreds of features
        let mut writer: IndexWriter = index
            .writer(15_000_000)
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        for f in features {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.slug, &f.slug);
            doc.add_text(fields.title, &f.title);
            if let Some(desc) = &f.description {
                doc.add_text(fields.description, desc);
            }
            doc.add_text(fields.phase, f.phase.to_string());

            // Body: slug (tokenized so "auth" finds "auth-google-oauth"), comment text,
            // task titles, and blockers — all concatenated for broad full-text coverage.
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

            // Read artifact files and append their content (capped at 8 KB each).
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
                let path = root.join("features").join(&f.slug).join(filename);
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

        writer
            .commit()
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        // Manual reload — we only ever read after the single commit above
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
    /// - Field scope: `phase:ready`, `slug:auth`, `title:oauth`
    /// - Prefix:     `auth*`
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let searcher = self.reader.searcher();

        // Default search fields: title, description, body
        // Phase and slug are reachable via field: prefix syntax
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
            let phase = doc
                .get_first(self.fields.phase)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            results.push(SearchResult {
                slug,
                title,
                phase,
                score,
            });
        }

        Ok(results)
    }
}

// ---------------------------------------------------------------------------
// TaskIndex
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
    ///
    /// Indexed fields:
    /// - `feature_slug` — STRING (exact-match, stored) + tokenized via `body`
    /// - `task_id`      — STRING (exact-match, stored)
    /// - `title`        — TEXT (tokenized, stored) — primary search field
    /// - `status`       — STRING (exact-match, stored) — `status:blocked` scoping
    /// - `description`  — TEXT (tokenized, not stored)
    /// - `body`         — TEXT — feature_slug tokens + blocker text + depends_on IDs
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
                // body: feature_slug (tokenized) + blocker text + depends_on IDs
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
    ///
    /// Supported query syntax:
    /// - Bare terms: `login form`            (AND by default)
    /// - Phrase:     `"implement login"`
    /// - Boolean:    `login OR signup`, `auth NOT legacy`
    /// - Field scope: `status:blocked`, `status:pending`
    /// - Prefix:     `login*`
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<TaskSearchResult>> {
        let searcher = self.reader.searcher();

        // Default search fields: title, description, body
        // status and feature_slug are reachable via field: prefix syntax
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
// PonderIndex
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct PonderSearchResult {
    pub slug: String,
    pub title: String,
    pub status: String,
    pub score: f32,
}

struct PonderFields {
    slug: Field,
    title: Field,
    status: Field,
    body: Field,
}

pub struct PonderIndex {
    index: Index,
    reader: tantivy::IndexReader,
    fields: PonderFields,
}

impl PonderIndex {
    /// Build an ephemeral in-RAM index from the given ponder entries.
    ///
    /// Indexed fields:
    /// - `slug`   — STRING (exact-match, stored)
    /// - `title`  — TEXT (tokenized, stored)
    /// - `status` — STRING (exact-match, stored) — `status:exploring` scoping
    /// - `body`   — TEXT — tags + artifact content
    pub fn build(
        entries: &[(
            crate::ponder::PonderEntry,
            Vec<crate::ponder::PonderArtifactMeta>,
        )],
        root: &Path,
    ) -> Result<Self> {
        let (schema, fields) = build_ponder_schema();

        let index = Index::create_in_ram(schema);

        let mut writer: IndexWriter = index
            .writer(15_000_000)
            .map_err(|e| SdlcError::Search(e.to_string()))?;

        for (entry, artifacts) in entries {
            let mut doc = TantivyDocument::default();
            doc.add_text(fields.slug, &entry.slug);
            doc.add_text(fields.title, &entry.title);
            doc.add_text(fields.status, entry.status.to_string());

            let mut body_parts: Vec<String> = Vec::new();
            body_parts.push(entry.tags.join(" "));

            // Read artifact content (capped at 8 KB each)
            body_parts.push(entry.slug.replace('-', " "));

            for artifact in artifacts {
                let path = crate::paths::ponder_dir(root, &entry.slug).join(&artifact.filename);
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
    pub fn search(&self, query_str: &str, limit: usize) -> Result<Vec<PonderSearchResult>> {
        let searcher = self.reader.searcher();

        let default_fields = vec![self.fields.title, self.fields.body];
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

            results.push(PonderSearchResult {
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
// Schema construction
// ---------------------------------------------------------------------------

fn build_schema() -> (Schema, Fields) {
    let mut builder = Schema::builder();

    let slug = builder.add_text_field("slug", STRING | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let description = builder.add_text_field("description", TEXT);
    let phase = builder.add_text_field("phase", STRING | STORED);
    let body = builder.add_text_field("body", TEXT);

    let schema = builder.build();
    let fields = Fields {
        slug,
        title,
        description,
        phase,
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

fn build_ponder_schema() -> (Schema, PonderFields) {
    let mut builder = Schema::builder();

    let slug = builder.add_text_field("slug", STRING | STORED);
    let title = builder.add_text_field("title", TEXT | STORED);
    let status = builder.add_text_field("status", STRING | STORED);
    let body = builder.add_text_field("body", TEXT);

    let schema = builder.build();
    let fields = PonderFields {
        slug,
        title,
        status,
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

    #[test]
    fn search_by_title_word() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        let results = index.search("authentication", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "auth-login");
    }

    #[test]
    fn search_by_description_word() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        let results = index.search("stripe", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "payment-flow");
    }

    #[test]
    fn search_no_match_returns_empty() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        let results = index.search("kubernetes", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_respects_limit() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        // "interface" matches "Search Interface"
        // broader query that matches multiple
        let results = index.search("search OR auth OR payment", 2).unwrap();
        assert!(results.len() <= 2);
    }

    #[test]
    fn search_phase_field_scoped() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        // All features are Draft by default
        let results = index.search("phase:draft", 10).unwrap();
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn search_empty_index() {
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&[], dir.path()).unwrap();
        let results = index.search("anything", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_scores_descending() {
        let features = make_features();
        let dir = tempfile::TempDir::new().unwrap();
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
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

    // -----------------------------------------------------------------------
    // TaskIndex tests
    // -----------------------------------------------------------------------

    fn make_feature_with_tasks() -> Feature {
        use crate::task::add_task;
        // Use a slug without "login" so body field tokens don't pollute title searches
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
        // Broad OR query to match multiple tasks
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
        let index = FeatureIndex::build(&features, dir.path()).unwrap();
        // Unclosed bracket is a Tantivy parse error — should return Ok([]) not Err
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

        // Write a spec.md for auth-login with content not in its title/description
        let feature_dir = dir.path().join("features").join("auth-login");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("spec.md"),
            "The system uses JWT tokens for stateless authentication with refresh token rotation",
        )
        .unwrap();

        let index = FeatureIndex::build(&features, dir.path()).unwrap();

        // "JWT" only appears in the artifact file, not in title/description
        let results = index.search("JWT", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].slug, "auth-login");
    }
}
