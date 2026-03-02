# Tasks: knowledge-core-data

- [x] Add KnowledgeNotFound/Exists/InvalidKnowledgeStatus/InvalidKnowledgeCode error variants to error.rs
- [x] Add KNOWLEDGE_DIR/KNOWLEDGE_CATALOG/KNOWLEDGE_MAINTENANCE_LOG constants to paths.rs
- [x] Add 5 knowledge path helpers to paths.rs (knowledge_dir, knowledge_manifest, knowledge_content_path, knowledge_catalog_path, knowledge_maintenance_log_path)
- [x] Add path helper tests for all 5 new functions
- [x] Register `pub mod knowledge;` in lib.rs
- [x] Create knowledge.rs: KnowledgeStatus/SourceType/OriginKind enums with Display + serde
- [x] Create knowledge.rs: KnowledgeEntry, Source, Catalog*, MaintenanceLog, SearchResult structs
- [x] Implement validate_code() with OnceLock<Regex>
- [x] Implement create/load/save/list/list_by_code_prefix/update CRUD
- [x] Implement append_content/read_content
- [x] Implement full_text_search (metadata hits rank first)
- [x] Implement session wrappers (log_session/list_sessions/read_session → workspace.rs)
- [x] Implement artifact wrappers (capture_named_artifact/list/read → workspace.rs)
- [x] Implement load_catalog/save_catalog/add_class/add_division
- [x] Implement load_maintenance_log/append_maintenance_action
- [x] Write 18 unit tests (19 with paths.rs test)
- [x] Verify: SDLC_NO_NPM=1 cargo test --all passes
- [x] Verify: cargo clippy --all -- -D warnings passes clean
