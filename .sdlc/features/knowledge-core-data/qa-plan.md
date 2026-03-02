# QA Plan: knowledge-core-data

## Automated verification

```bash
SDLC_NO_NPM=1 cargo test -p sdlc-core knowledge
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All must pass with zero failures and zero warnings.

## Test coverage checklist

| Area | Test | Status |
|---|---|---|
| Entry creation | `create_returns_entry_with_correct_fields` | ✅ |
| Duplicate prevention | `duplicate_slug_rejected` | ✅ |
| Serde round-trip | `load_roundtrip` | ✅ |
| List all | `list_returns_all` | ✅ |
| Code prefix filter | `list_by_code_prefix_filters` | ✅ |
| Field updates | `update_modifies_fields` | ✅ |
| Reclassification safety | `reclassify_does_not_rename_dir` | ✅ |
| Code validation (valid) | `validate_code_accepts_valid` | ✅ |
| Code validation (invalid) | `validate_code_rejects_invalid` | ✅ |
| Content append/read | `append_and_read_content` | ✅ |
| Search title | `full_text_search_in_title` | ✅ |
| Search content.md | `full_text_search_in_content` | ✅ |
| Search empty base | `full_text_search_empty_base` | ✅ |
| Session logging | `log_session_increments_count` | ✅ |
| Catalog persistence | `catalog_add_class_and_load` | ✅ |
| Catalog missing | `catalog_load_missing_returns_empty` | ✅ |
| Maintenance log | `maintenance_log_append_and_load` | ✅ |
| Maintenance log missing | `maintenance_log_missing_returns_empty` | ✅ |
| Path helpers | `paths::tests::knowledge_path_helpers` | ✅ |

## Edge cases verified

- Empty knowledge dir: `list`, `full_text_search` return `[]`, not error
- Missing catalog.yaml: returns empty `Catalog`, not error
- Missing maintenance-log.yaml: returns empty `MaintenanceLog`, not error
- Reclassification leaves directory name unchanged
- `updated_at` changes on update; `created_at` does not
