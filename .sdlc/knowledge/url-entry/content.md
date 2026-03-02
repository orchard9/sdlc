# URL Entry — Web-Origin Knowledge Entries

## What This Entry Is

`url-entry` is a UAT fixture entry created during the v10-knowledge-capture milestone to validate the `--from-url` flag of `sdlc knowledge add`. It was created via:

```bash
sdlc knowledge add --title "URL Entry" --from-url https://example.com
```

The entry demonstrates the **web origin** (`OriginKind::Web`) pattern in the knowledge system.

---

## The `--from-url` Feature

When `sdlc knowledge add --from-url <url>` is used, the CLI does the following:

1. **Creates** the entry directory at `.sdlc/knowledge/<slug>/`
2. **Fetches** the URL via HTTP (using `ureq`, 10-second timeout, best-effort)
3. **Extracts** the `<title>` HTML tag from the response body
4. **Writes** `content.md` with:
   ```
   Fetched from: <url>

   Page title: <extracted title>   ← omitted if extraction fails
   ```
5. **Records** a `Source` provenance record: `{ type: web, url: <url>, captured_at: ... }`
6. **Sets** `origin: web` on the entry manifest

### Mutual exclusivity

`--from-url`, `--from-file`, and `--content` are mutually exclusive. Only one may be provided per `add` call.

---

## Data Model

### `SourceType` enum (where content came from)

| Variant | YAML value | Usage |
|---|---|---|
| `Web` | `web` | URL fetched via HTTP |
| `LocalFile` | `local_file` | File read from disk |
| `Manual` | `manual` | Inline `--content` text |
| `Harvested` | `harvested` | Extracted from a ponder/investigation workspace |
| `Guideline` | `guideline` | Published from a guideline workspace |

### `OriginKind` enum (how the entry was first created)

| Variant | YAML value | Usage |
|---|---|---|
| `Web` | `web` | Created with `--from-url` |
| `Manual` | `manual` | Created with `--content` or no content flag |
| `Research` | `research` | Created by an agent research run |
| `Harvested` | `harvested` | Harvested from a workspace |
| `Guideline` | `guideline` | Harvested from a published guideline |

### `Source` struct

```yaml
sources:
- type: web
  url: https://example.com
  captured_at: 2026-03-02T09:48:03.658971Z
```

Fields `path` and `workspace` are omitted for web sources.

---

## Implementation Location

| File | Role |
|---|---|
| `crates/sdlc-cli/src/cmd/knowledge.rs` | CLI handler; `add()` fn and `fetch_page_title()` fn |
| `crates/sdlc-core/src/knowledge.rs` | `SourceType`, `OriginKind`, `Source`, `KnowledgeEntry` structs |
| `crates/sdlc-cli/Cargo.toml` | `ureq = "2"` dependency for HTTP fetching |

The `fetch_page_title` function (line ~894 in knowledge.rs) is a best-effort HTML parser:
- Uses `ureq::get` with a 10-second timeout
- Finds `<title>...</title>` in the lowercased response body
- Returns `None` on any failure (network error, missing tag, etc.)

---

## Orphan Status

This specific entry (`url-entry`) is an orphan fixture — it has placeholder content only, no analysis, and was created solely to exercise the `--from-url` code path during UAT. It was identified and marked as an orphan in the knowledge librarian maintenance session (2026-03-02).

Other orphan entries from the same UAT run: `no-code-entry`, `uat-test-entry`, `api-created-entry`, `uat-test-inv`.

---

## Usage Pattern

To capture a web resource as a knowledge entry:

```bash
# Capture a documentation page
sdlc knowledge add --title "Rust Async Book" \
  --code 200.10 \
  --from-url https://rust-lang.github.io/async-book/

# Then enrich with analysis
sdlc knowledge update rust-async-book --summary "Core reference for async/await patterns in Rust"

# Log a research session
sdlc knowledge session log rust-async-book --content "Reviewed chapters 1-3. Key insight: ..."
```

The `--from-url` flag is a *capture* shortcut — it records provenance automatically. The agent or human is still expected to enrich the entry with a proper summary and session notes.
