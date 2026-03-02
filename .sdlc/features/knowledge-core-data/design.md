# Design: knowledge-core-data

## Approach

Pure data layer following the `investigation.rs` model. No logic, no heuristics — load, save, validate. Agent decision logic lives in skill templates, not here.

## Module Structure

```
crates/sdlc-core/src/knowledge.rs
├── Enums: KnowledgeStatus, SourceType, OriginKind
├── Structs: KnowledgeEntry, Source, Catalog*, MaintenanceLog, SearchResult
├── validate_code() — OnceLock<Regex>
├── Entry CRUD: create / load / save / list / list_by_code_prefix / update
├── Content: append_content / read_content
├── Full-text search: full_text_search
├── Session wrappers: log_session / list_sessions / read_session
├── Artifact wrappers: capture_named_artifact / list_named_artifacts / read_named_artifact
├── Catalog: load_catalog / save_catalog / add_class / add_division
└── Maintenance log: load_maintenance_log / append_maintenance_action
```

## Key Design Decisions

### Slug-only directories
`.sdlc/knowledge/<slug>/` — the code lives only in `manifest.yaml`. Reclassification = YAML field update, no filesystem rename. This was the critical fix from the v10 pressure test.

### Reuse workspace.rs
Sessions and artifacts delegate to `workspace::write_session`, `workspace::list_sessions`, etc. No duplication of I/O primitives.

### Graceful empty states
`load_catalog`, `load_maintenance_log`, `list`, `full_text_search` all return empty values (not errors) when the knowledge directory or files don't exist. Critical for the `sdlc knowledge status` command to work before init.

### Full-text search ordering
Metadata hits (title/summary/tags match) sort before content-only hits, matching information retrieval convention: structured matches are more relevant.

### Atomic writes everywhere
All writes go through `crate::io::atomic_write` — consistent with the rest of sdlc-core.
