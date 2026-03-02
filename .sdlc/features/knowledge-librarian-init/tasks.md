# Tasks: sdlc knowledge librarian init

## Status

The core implementation in `crates/sdlc-core/src/knowledge.rs` is complete:
- `librarian_init()` — 9-step orchestrator
- `harvest_investigations()`, `harvest_ponders()`, `harvest_guidelines()`
- `seed_catalog()` — ARCHITECTURE.md heading extraction + defaults
- `write_librarian_agent_file()` — template substitution + atomic write
- `cross_ref_pass()` — tag-overlap cross-referencing
- `upsert_knowledge_entry()` — idempotent create-or-append
- CLI dispatch in `crates/sdlc-cli/src/cmd/knowledge.rs :: run_librarian()`
- Existing tests: empty project, agent file creation, idempotency (basic), per-harvester unit tests, catalog seeding, cross-ref pass

One gap remains: the idempotency test that covers multiple entity types simultaneously (T1 from manifest).

---

## T1 — Add idempotency integration test covering all harvest types

**File:** `crates/sdlc-core/src/knowledge.rs` (test module)

Add a test `librarian_init_idempotent_full` that:
1. Seeds one completed investigation (root-cause kind)
2. Seeds one committed ponder
3. Seeds one published guideline (with a real file at `publish_path`)
4. Calls `librarian_init(&root)` — asserts `created: true` for all 3 harvest results
5. Calls `librarian_init(&root)` again — asserts `created: false` for all 3
6. Asserts `list(&root).len() == 3` (no duplicates)
7. Asserts no duplicate slugs in `related[]` of any entry

This directly addresses the `[user-gap]` task in the manifest: proving the idempotency guarantee holds when all three harvest types are present simultaneously.

**Acceptance:** `SDLC_NO_NPM=1 cargo test -p sdlc-core librarian_init_idempotent_full` passes.

---

## T2 — Verify CLI integration test covers librarian init

**File:** `crates/sdlc-cli/tests/integration.rs`

Check whether an integration test for `sdlc knowledge librarian init` exists. If absent, add a test that:
1. Invokes the binary with `sdlc knowledge librarian init`
2. Asserts exit code 0
3. Asserts output contains "Knowledge base initialized"
4. Invokes again (idempotency)
5. Asserts exit code 0 on second run

**Acceptance:** `SDLC_NO_NPM=1 cargo test -p sdlc-cli -- librarian` passes.

---

## T3 — Verify clippy clean on knowledge.rs librarian section

**Scope:** `crates/sdlc-core/src/knowledge.rs` lines 655–1100 (harvest + init functions)

Run:
```bash
SDLC_NO_NPM=1 cargo clippy -p sdlc-core -- -D warnings
```

Fix any warnings in the librarian init code path. Common issues: unused `allow` attrs, redundant clones, or shadowed variables.

**Acceptance:** `cargo clippy -p sdlc-core -- -D warnings` exits 0.
