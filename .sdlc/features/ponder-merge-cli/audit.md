# Audit: sdlc ponder merge — CLI command and core data model

## Three-Lens Audit

### 1. Product Fit

**Finding:** Feature aligns with milestone vision (consolidate fragmented ponder entries). The CLI interface (`sdlc ponder merge <source> --into <target>`) is consistent with existing ponder subcommand patterns.

**Action:** Accepted — no issues.

### 2. Research Grounding

**Finding:** The merge model follows established patterns in the codebase:
- Session copying reuses `workspace::write_session` (auto-numbering)
- Team merge uses same dedup-by-name pattern as `add_team_member`
- Source parking mirrors the existing `archive` command behavior
- `merged_into`/`merged_from` fields use `serde(default)` for backward compat, matching the pattern used by `parked_at` on milestones

**Action:** Accepted — patterns are consistent.

### 3. Implementation Quality

**Finding 1 — Path safety:** All file paths are constructed from validated slugs (`paths::validate_slug`). Artifact filenames from source are filesystem names read via `read_dir`, not user input. No path traversal risk.
**Action:** Accepted.

**Finding 2 — No unwrap in library code:** Verified — all error paths use `?` propagation with `SdlcError` variants.
**Action:** Accepted.

**Finding 3 — Atomic writes:** All file writes use `crate::io::atomic_write`. Session writes delegate to `workspace::write_session` which also uses atomic_write.
**Action:** Accepted.

**Finding 4 — Partial failure behavior:** If merge fails partway (e.g., filesystem error during artifact copy), some sessions/artifacts may already be copied to the target, but the source will NOT be parked (parking is the final step). This means the user can re-run the merge after fixing the issue. However, already-copied sessions will be duplicated on retry.
**Action:** Accepted with rationale — this is documented in the design. Sessions are append-only and duplicates in a scrapbook are harmless. The alternative (transactional rollback) would add significant complexity for minimal benefit in a YAML-file-based system.

**Finding 5 — CLI reference in AGENTS.md:** The `ponder` subcommand table in AGENTS.md lists: `create · list · show · capture · team add · team list · update · archive · artifacts`. The new `merge` subcommand should be added.
**Action:** Fixed — adding `merge` to the CLI reference.

## Audit Score: 92/100

All findings addressed. One documentation update applied.
