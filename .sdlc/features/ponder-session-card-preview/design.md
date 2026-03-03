# Design: ponder-session-card-preview

## Overview

This feature adds a short text preview of the latest ponder session to each entry card in the ponder list view. The change touches three layers:

1. **sdlc-core** — a pure function `extract_session_preview` in `workspace.rs`
2. **sdlc-server** — `routes/roadmap.rs` list handler reads the last session and includes `last_session_preview`
3. **Frontend** — `PonderSummary` type gains the field; `EntryRow` renders it

---

## Layer 1: Core — `workspace.rs`

### Function: `extract_session_preview(content: &str) -> Option<String>`

Location: `crates/sdlc-core/src/workspace.rs`

**Algorithm:**
1. Strip YAML frontmatter (already handled by `extract_frontmatter`)
2. Split remaining content into lines
3. Scan lines top-to-bottom; skip:
   - Empty or whitespace-only lines
   - HTML comment tags: lines starting with `<!--` or `-->`
   - Tool/artifact block tags (same markers as frontend: `<!-- tool:`, `<!-- /tool`, `<!-- artifact:`, `<!-- /artifact`)
   - Lines that are only `---` (fence)
   - Lines starting with `#` (headings — often just structural)
   - Lines matching `Recruited:` prefix (metadata)
4. Take the first remaining line after stripping leading `⚑ `, `? `, and `**...**` bold markers
5. Trim to 140 characters, appending `…` if truncated
6. Return `None` if no suitable line is found

**Signature:**
```rust
pub fn extract_session_preview(content: &str) -> Option<String>
```

**Constants:**
```rust
const PREVIEW_MAX_CHARS: usize = 140;
```

**Why workspace.rs (not ponder.rs):** The extraction logic is purely about session file format — the same session structure is shared by ponder and investigation. Placing it in `workspace.rs` keeps the function available for future reuse.

---

## Layer 2: Server — `routes/roadmap.rs`

### Change: `list_ponders` handler

The `list_ponders` handler already runs in `spawn_blocking`. We extend it to also:
1. Call `ponder::list_sessions(root, slug)` to get session metadata sorted ascending
2. If sessions exist, read the last session: `ponder::read_session(root, slug, last_session.session)`
3. Call `workspace::extract_session_preview(&content)` to get the preview
4. Include `"last_session_preview": preview` in the JSON object (null if no sessions or no preview)

The additional I/O is bounded: only one file read per entry, and only when sessions exist. For large lists this may add latency, but since the existing handler already reads team data per entry, this is acceptable.

**JSON shape addition:**
```json
{
  "slug": "my-idea",
  "title": "My Idea",
  "status": "exploring",
  "sessions": 3,
  "last_session_preview": "We are exploring whether memory is the right frame for this problem...",
  ...
}
```

---

## Layer 3: Frontend

### Type change: `PonderSummary` in `frontend/src/lib/types.ts`

Add optional field:
```typescript
export interface PonderSummary {
  // ... existing fields ...
  last_session_preview?: string | null
}
```

### Component change: `EntryRow` in `frontend/src/pages/PonderPage.tsx`

Add a third row to the card beneath the existing session/team metadata row:

```tsx
{entry.last_session_preview && (
  <p className="text-xs text-muted-foreground/50 line-clamp-1 mt-0.5 italic">
    {entry.last_session_preview}
  </p>
)}
```

**Visual design:**
- Font: `text-xs` (12px), same as session count row
- Color: `text-muted-foreground/50` — lighter than the session count to establish visual hierarchy (count is `/60`)
- Style: `italic` to differentiate preview from structural metadata
- Truncation: `line-clamp-1` — single line, CSS overflow ellipsis
- Margin: `mt-0.5` — tight spacing below the metadata row
- Shown only when present; layout is unchanged for entries without sessions

### ASCII Wireframe

```
┌─────────────────────────────────────────────┐
│ My Ponder Idea                  [exploring] │
│ 3 sessions · 2 team members                 │
│ We are exploring whether memory is the...   │  ← new line (italic, muted)
└─────────────────────────────────────────────┘

┌─────────────────────────────────────────────┐
│ New Idea Without Sessions       [exploring] │
│ no sessions yet                             │
│ (nothing here — layout unchanged)           │
└─────────────────────────────────────────────┘
```

---

## Data Flow

```
GET /api/roadmap
       │
       ▼
list_ponders (spawn_blocking)
  ├── PonderEntry::list(root)         — load manifests
  ├── list_artifacts(root, slug)      — count artifacts
  ├── load_team(root, slug)           — count team members
  ├── list_sessions(root, slug)       — check if sessions exist
  │     └── [if sessions exist]
  │           read_session(root, slug, last_n)   — read last session file
  │           extract_session_preview(content)   — extract preview text
  └── serialize JSON with last_session_preview
```

---

## Error Handling

- `list_sessions` failure for a single entry: log the error (or silently omit), do not fail the entire list response. The field is omitted or null.
- `read_session` failure: same — omit preview, do not propagate.
- These are best-effort enrichments. The list endpoint must always succeed even if session files are corrupt or missing.

---

## Testing

- **Unit test** for `extract_session_preview` in `workspace.rs`:
  - Returns `None` for empty content
  - Returns `None` for content with only frontmatter/headings
  - Returns first narrative line when present
  - Truncates at 140 chars with ellipsis
  - Skips tool/artifact comment blocks
  - Handles partner messages (strips bold markers)
- **Integration test** for `list_ponders` route: verify `last_session_preview` is present when a session exists.

---

## Risk

Low. This is additive — new optional field in API, new optional render in UI. No existing behavior changes. Worst-case regression: preview shows unexpected content, which is cosmetic only.
