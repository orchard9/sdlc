# Tasks: ponder-session-card-preview

## T1 — Add `extract_session_preview` to `workspace.rs`

Add a `pub fn extract_session_preview(content: &str) -> Option<String>` function to `crates/sdlc-core/src/workspace.rs`.

Algorithm:
1. Strip YAML frontmatter
2. Scan lines, skipping: empty lines, `<!--...-->` comment tags, `---` fences, lines starting with `#`, lines starting with `Recruited:`
3. From the first remaining line, strip leading `⚑ `, `? `, and surrounding `**...**` bold markers
4. Trim to 140 chars, appending `…` if truncated
5. Return `None` if no suitable line found

Add unit tests covering: empty content, frontmatter-only, narrative line extraction, truncation, tool block skipping, partner message stripping.

**Files:** `crates/sdlc-core/src/workspace.rs`

---

## T2 — Enrich `list_ponders` with `last_session_preview`

In `crates/sdlc-server/src/routes/roadmap.rs`, update the `list_ponders` blocking task to:
1. Call `sdlc_core::ponder::list_sessions(&root, &e.slug)` for each entry
2. If sessions exist, call `sdlc_core::ponder::read_session(&root, &e.slug, last.session)` for the last session
3. Call `sdlc_core::workspace::extract_session_preview(&content)` to get the preview
4. Add `"last_session_preview": preview` to the JSON object (null if no sessions or extraction fails)

Error handling: any I/O error in reading sessions or the session file is silently absorbed — `last_session_preview` is `null`.

**Files:** `crates/sdlc-server/src/routes/roadmap.rs`

---

## T3 — Update `PonderSummary` TypeScript type

In `frontend/src/lib/types.ts`, add optional field to `PonderSummary`:
```typescript
last_session_preview?: string | null
```

**Files:** `frontend/src/lib/types.ts`

---

## T4 — Render preview in `EntryRow`

In `frontend/src/pages/PonderPage.tsx`, update `EntryRow` to render the preview:
```tsx
{entry.last_session_preview && (
  <p className="text-xs text-muted-foreground/50 line-clamp-1 mt-0.5 italic">
    {entry.last_session_preview}
  </p>
)}
```

Add this after the existing session/team metadata row (the `<div className="flex items-center gap-2.5 ...">` block).

**Files:** `frontend/src/pages/PonderPage.tsx`

---

## T5 — Verify build passes

Run:
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Confirm all tests pass and no clippy warnings.
