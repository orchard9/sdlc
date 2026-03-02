# QA Plan: sdlc knowledge librarian init

## Test Strategy

All QA is automated via Rust unit + integration tests. No Playwright or server-level tests are required — this feature is CLI-only with no frontend changes.

Run command:
```bash
SDLC_NO_NPM=1 cargo test --all -- --test-output immediate 2>&1 | grep -E "test |FAILED|ok|error"
cargo clippy --all -- -D warnings
```

---

## Unit Tests (crates/sdlc-core)

### 1. Empty project — no crash, correct defaults

**Test:** `librarian_init_on_empty_project`
- Input: empty tempdir (no investigations, ponders, guidelines, no ARCHITECTURE.md)
- Expected: report with 0 investigation/ponder/guideline results, `catalog_created: true`, `catalog_class_count >= 1`, agent file exists at `.claude/agents/knowledge-librarian.md`

### 2. Agent file written with correct content

**Test:** `librarian_init_creates_agent_file`
- Input: empty project
- Expected: `.claude/agents/knowledge-librarian.md` exists, contains "Knowledge Librarian"

### 3. Basic idempotency (investigation only)

**Test:** `librarian_init_idempotent`
- Input: one completed investigation
- Run 1: `created: true` for that investigation
- Run 2: `created: false` — no new entry created
- Entry count stays at 1

### 4. Full idempotency — all three harvest types (T1)

**Test:** `librarian_init_idempotent_full` (to be added by T1)
- Input: one completed investigation + one committed ponder + one published guideline
- Run 1: all three `created: true`; `list().len() == 3`
- Run 2: all three `created: false`; `list().len() == 3` (no duplicates)
- No duplicate slugs in `related[]` of any entry

### 5. Investigation harvest — completed investigation creates entry

**Test:** `harvest_investigation_creates_entry`
- Input: one completed root-cause investigation with context
- Expected: entry slug `investigation-<slug>`, `origin: harvested`, `harvested_from: "investigation/<slug>"`

### 6. Investigation harvest — in-progress investigation skipped

**Test:** `harvest_investigation_in_progress_skipped`
- Input: one in-progress investigation
- Expected: empty results

### 7. Ponder harvest — committed ponder creates entry

**Test:** `harvest_ponder_committed_creates_entry`
- Input: one committed ponder
- Expected: entry slug `ponder-<slug>`, `origin: harvested`, tags include `"ponder"`

### 8. Ponder harvest — exploring ponder skipped

**Test:** `harvest_ponder_exploring_skipped`
- Input: one exploring ponder (default status)
- Expected: empty results

### 9. Guideline harvest — published guideline creates entry

**Test:** `harvest_guideline_creates_entry`
- Input: one investigation with `kind: guideline` and a real `publish_path`
- Expected: entry slug `guideline-<slug>`, `origin: guideline`, tags include `"guideline"`

### 10. Catalog seeding — uses defaults when no ARCHITECTURE.md

**Test:** `seed_catalog_uses_defaults`
- Expected: 5 classes (100–500), first is "Architecture & Design"

### 11. Catalog seeding — uses ARCHITECTURE.md H2 headings

**Test:** `seed_catalog_uses_architecture_headings`
- Input: ARCHITECTURE.md with 3 `## Heading` lines
- Expected: 3 classes named after the headings

### 12. Cross-reference pass — links entries with ≥2 shared tags

**Test:** `cross_ref_pass_links_entries`
- Input: two entries with 2+ overlapping tags
- Expected: each appears in the other's `related[]`, count > 0

### 13. Cross-reference pass — no link for <2 shared tags

- Input: two entries with only 1 shared tag
- Expected: `related[]` unchanged, count = 0

---

## CLI Integration Tests (crates/sdlc-cli)

### 14. `sdlc knowledge librarian init` exits 0 (T2)

- Run the binary with `knowledge librarian init`
- Expected: exit code 0, stdout contains "Knowledge base initialized"

### 15. `sdlc knowledge librarian init` idempotent at CLI level (T2)

- Run twice on same project root
- Both runs exit 0

### 16. `sdlc knowledge librarian init --json` outputs valid JSON

- Expected: parseable JSON with keys `investigations_new`, `catalog_created`, `agent_file`

---

## Clippy / Code Quality (T3)

### 17. Zero clippy warnings in librarian code path

```bash
SDLC_NO_NPM=1 cargo clippy -p sdlc-core -- -D warnings
SDLC_NO_NPM=1 cargo clippy -p sdlc-cli -- -D warnings
```

Expected: exit 0, no warnings.

---

## Pass Criteria

All 17 checks must pass. Failures in T1 or T2 are blockers. Clippy warnings in the librarian code path are blockers.

```bash
# Full verification command
SDLC_NO_NPM=1 cargo test --all && cargo clippy --all -- -D warnings
```
