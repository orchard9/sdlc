# Tasks: backlog-core

## Implementation Order

Tasks are sequential. Each builds on the previous.

1. **Add BACKLOG_FILE const and backlog_path() to paths.rs**
   Add `pub const BACKLOG_FILE: &str = ".sdlc/backlog.yaml";` and
   `pub fn backlog_path(root: &Path) -> PathBuf { root.join(BACKLOG_FILE) }`.

2. **Add BacklogItemNotFound variant to error.rs**
   Add `#[error("backlog item not found: {0}")] BacklogItemNotFound(String),` to `SdlcError`.

3. **Create crates/sdlc-core/src/backlog.rs**
   Full module: `BacklogKind`, `BacklogStatus`, `BacklogItem`, `BacklogStore` with all methods.
   Includes `add`, `list`, `get`, `park`, `mark_promoted`, `load`, `save`, `next_id`.

4. **Register pub mod backlog in lib.rs**
   Add `pub mod backlog;`.

5. **Write unit tests in backlog.rs**
   Cover all 14 test cases from the design doc using `tempfile::TempDir`.

## Existing Task Mapping

The pre-created tasks (T1–T9) map to these implementation items and will be
checked off as code is written. The tasks.md artifact covers the same scope —
no duplication of task creation needed.
