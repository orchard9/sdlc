# Spec: ponder-session-card-preview

## Feature Title
Session card preview from Product Summary (API + UI)

## Problem Statement

The ponder list (left panel in PonderPage) shows entry cards with only a title, status badge, session count, and team size. When a user scans their list of ponder entries, they have no way to recall the substance of any entry without clicking into it. This slows down navigation and forces context-switching just to remember what each idea is about.

## Goal

Surface a short text preview of the latest session content on each ponder entry card in the list view. This preview gives users immediate recall of what is being discussed in each ponder entry without requiring them to open it.

## User Story

As a user browsing my ponder entries, I want to see a one-line preview of the most recent session's content on each card so I can quickly identify which entry I want to open.

## Scope

### In scope
1. **API change** — `GET /api/roadmap` list endpoint includes a new optional field `last_session_preview: Option<String>` (max ~140 chars) extracted from the last session's raw content. The preview is the first meaningful narrative or partner-message line found after stripping YAML frontmatter, tool blocks, and artifact blocks.
2. **Core extraction function** — `sdlc-core` adds a `extract_session_preview(content: &str) -> Option<String>` function that extracts the preview text (first non-empty, non-meta text line, trimmed to 140 chars with ellipsis if needed).
3. **Type update** — `PonderSummary` TypeScript interface gains the optional field `last_session_preview?: string`.
4. **UI update** — `EntryRow` in PonderPage renders the preview text as a subtle second line below the session/team metadata, truncated with `line-clamp-1`. Shown only when present and not empty.

### Out of scope
- Configurable preview length
- Showing preview for sessions that are currently running (live state)
- Preview for investigation entries (separate concern)
- Hover tooltips or expandable preview

## Acceptance Criteria

1. `GET /api/roadmap` response includes `last_session_preview` for entries that have at least one session. The field is `null` or absent for entries with no sessions.
2. The preview text is derived from the last session file, contains meaningful content (not YAML, not tool/artifact comment tags), and is ≤ 140 characters with trailing `…` if truncated.
3. The ponder list card renders the preview text below the session/team count row when present. When absent, layout is unchanged.
4. The preview does not appear in the card for entries with no sessions (no empty/whitespace placeholder shown).
5. All existing ponder API tests pass with the new field included.
6. Frontend TypeScript compiles without errors.
7. `SDLC_NO_NPM=1 cargo test --all` and `cargo clippy --all -- -D warnings` pass.

## Technical Constraints

- The preview extraction must happen at list-time in the blocking Rust task — it reads the last session file from disk.
- No new data persisted to YAML — this is always derived at request time.
- The extraction function is pure Rust, no regex crate required; simple line-by-line scan.
- The UI change is isolated to `EntryRow` in `PonderPage.tsx`.
