# Tasks: human-uat-backend

## T1: Add `UatRunMode` enum and `mode` field to `UatRun`

**File:** `crates/sdlc-core/src/milestone.rs`

1. Add `UatRunMode` enum with `Agent` (default) and `Human` variants, `serde(rename_all = "snake_case")`.
2. Add `#[serde(default)] pub mode: UatRunMode` to `UatRun`.
3. Re-export `UatRunMode` from `crates/sdlc-core/src/lib.rs`.
4. Add unit test `uat_run_mode_backward_compat` — deserialize YAML without `mode` field, assert result is `Agent`.

**Acceptance:** `SDLC_NO_NPM=1 cargo test -p sdlc-core` passes. No clippy warnings.

---

## T2: Implement `POST /api/milestone/{slug}/uat/human` handler

**File:** `crates/sdlc-server/src/routes/runs.rs`

1. Add `HumanUatBody` struct (verdict, tests_total, tests_passed, tests_failed, notes).
2. Implement `submit_milestone_uat_human` handler:
   - Validate slug.
   - `spawn_blocking`: load Milestone, validate notes (non-empty for Failed/PassWithTasks), generate run_id, write summary.md via `io::atomic_write`, build `UatRun` with `mode: UatRunMode::Human`, call `save_uat_run`, if verdict==Pass call `milestone.release()` + `milestone.save()`.
   - Emit `SseMessage::MilestoneUatCompleted { slug }`.
   - Return `{ run_id, slug, status: "submitted" }`.
3. Helper `fn generate_run_id() -> String` using chrono + rand.

**Acceptance:** Handler compiles, no `unwrap()`, all I/O through `atomic_write`.

---

## T3: Implement `POST /api/features/{slug}/human-qa` handler

**File:** `crates/sdlc-server/src/routes/features.rs`

1. Add `HumanQaBody` struct (verdict: String, notes: String).
2. Implement `submit_human_qa` handler:
   - Validate slug.
   - `spawn_blocking`: load Feature (→ 404), validate verdict string and notes, format qa-results.md content, write to `.sdlc/features/{slug}/qa-results.md` via `io::atomic_write`, reload Feature and call `feature.draft_artifact(ArtifactType::QaResults)`, save.
   - Emit `SseMessage::Update`.
   - Return `{ slug, artifact: "qa_results", status: "draft" }`.

**Acceptance:** Handler compiles, no `unwrap()`.

---

## T4: Register new routes in `lib.rs`

**File:** `crates/sdlc-server/src/lib.rs`

1. Add route: `POST /api/milestone/{slug}/uat/human` → `routes::runs::submit_milestone_uat_human`.
2. Add route: `POST /api/features/{slug}/human-qa` → `routes::features::submit_human_qa`.
3. Place the human route before `/api/milestone/{slug}/uat/stop` and `/fail` to keep lexicographic order sensible.

**Acceptance:** `SDLC_NO_NPM=1 cargo build --all` succeeds.

---

## T5: Integration tests

**File:** `crates/sdlc-server/tests/integration.rs`

1. `human_uat_submit_pass` — create milestone in verifying state, POST `verdict: pass`, assert 200, `run.yaml` has `mode: human`, milestone `released_at` set.
2. `human_uat_submit_pass_with_tasks_empty_notes` — POST `verdict: pass_with_tasks`, empty notes → assert 422.
3. `human_uat_submit_failed_empty_notes` — POST `verdict: failed`, empty notes → assert 422.
4. `human_qa_submit_drafts_artifact` — create feature, POST to `/api/features/{slug}/human-qa`, assert 200 + `qa_results` artifact status is `draft`.

**Acceptance:** All 4 tests pass under `SDLC_NO_NPM=1 cargo test --all`. Zero clippy warnings.
