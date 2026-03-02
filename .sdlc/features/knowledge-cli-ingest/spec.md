# Spec: knowledge-cli-ingest

## Purpose

CLI commands and server REST routes for the knowledge base. Provides the full CRUD surface for agents and humans to add, browse, search, and update knowledge entries.

No librarian logic here (that is `knowledge-librarian-init`).

---

## CLI: `crates/sdlc-cli/src/cmd/knowledge.rs`

### `sdlc knowledge status`
Shows: initialized (yes/no), entry count, catalog category count, last-maintained date.
If not initialized (no `.sdlc/knowledge/` dir or empty), prints:
```
Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed from your project.
```
Otherwise prints a summary table.

### `sdlc knowledge add`
```
sdlc knowledge add --title <title>
  [--code <NNN.NN>]         # optional; defaults to "uncategorized"
  [--content <text>]        # inline text → content.md
  [--from-url <url>]        # v10: stores URL source, fetches page title via HTTP
  [--from-file <path>]      # reads file content → content.md
```
- `--title` is always required
- `--code` is optional; if omitted entry gets `code: uncategorized`, `status: draft`
- `--from-url` and `--from-file` and `--content` are mutually exclusive
- `--from-url` behavior (v10): fetch the URL via HTTP, extract `<title>` tag (best-effort), store a `Source { source_type: Web, url }`, write brief content.md with fetched page title
- `--from-file` behavior: read file content, store as content.md, add `Source { source_type: LocalFile, path }`
- `--content` behavior: write inline text to content.md, `Source { source_type: Manual }`
- Slug is derived from `--title` (slugify: lowercase, replace spaces and non-alnum with `-`, deduplicate `-`)
- Outputs: `Created knowledge entry '<slug>'.`

### `sdlc knowledge list`
```
sdlc knowledge list
  [--code-prefix <100>]
  [--tag <tag>]
  [--status draft|published]
  [--json]
```
Table columns: `CODE`, `TITLE` (truncated to 30 chars), `SUMMARY` (first 60 chars), `STATUS`, `UPDATED`.
Empty-state: if no entries, print the init hint.

### `sdlc knowledge show <slug>`
```
sdlc knowledge show <slug> [--json]
```
Default output: code badge `[100.20]`, title, summary, tags, sources list, then content.md as plain text.
JSON: full `KnowledgeEntry` fields + `content` field.

### `sdlc knowledge search <query>`
```
sdlc knowledge search '<query>' [--json]
```
Calls `knowledge::full_text_search`. Table columns: `SLUG`, `TITLE`, `EXCERPT`.
Empty-state: if no entries, print the init hint.

### `sdlc knowledge update <slug>`
```
sdlc knowledge update <slug>
  [--code <NNN.NN>]
  [--status published|draft]
  [--tag <tag>]              # appended to existing tags
  [--related <slug-or-code>] # appended to related list
  [--summary <text>]
```
Code update requires no filesystem rename (directory is slug-based).

### `sdlc knowledge catalog show`
```
sdlc knowledge catalog show [--json]
```
Prints the taxonomy tree. If catalog is empty: `No catalog defined. Run sdlc knowledge librarian init or sdlc knowledge catalog add.`

### `sdlc knowledge catalog add`
```
sdlc knowledge catalog add --code <NNN.NN> --name <name> [--description <desc>]
```
Adds a class or division depending on code format:
- `NNN` → top-level class
- `NNN.NN` → division under class `NNN`

### `sdlc knowledge session log/list/read <slug>`
Delegates to `knowledge::log_session / list_sessions / read_session`.

---

## Server: `crates/sdlc-server/src/routes/knowledge.rs`

| Method | Path | Handler |
|---|---|---|
| GET | /api/knowledge/catalog | `get_catalog` |
| GET | /api/knowledge | `list_knowledge` — supports `?code=100` and `?tag=<t>` query params |
| POST | /api/knowledge | `create_knowledge` |
| GET | /api/knowledge/:slug | `get_knowledge` — returns entry + content |
| PUT | /api/knowledge/:slug | `update_knowledge` |
| POST | /api/knowledge/:slug/capture | `capture_knowledge_artifact` |
| GET | /api/knowledge/:slug/sessions | `list_knowledge_sessions` |
| GET | /api/knowledge/:slug/sessions/:n | `get_knowledge_session` |

---

## Files modified/created

| File | Change |
|---|---|
| `crates/sdlc-cli/Cargo.toml` | +`ureq` (URL fetch for --from-url) |
| `crates/sdlc-cli/src/cmd/mod.rs` | +`pub mod knowledge;` |
| `crates/sdlc-cli/src/cmd/knowledge.rs` | **CREATE** CLI module |
| `crates/sdlc-cli/src/main.rs` | Add `Knowledge` variant + handler |
| `crates/sdlc-server/src/routes/mod.rs` | +`pub mod knowledge;` |
| `crates/sdlc-server/src/routes/knowledge.rs` | **CREATE** server routes |
| `crates/sdlc-server/src/lib.rs` | Register knowledge routes |

---

## Slug derivation from title

```
slugify(title: &str) -> String:
  lowercase, replace [^a-z0-9]+ with '-', strip leading/trailing '-', truncate at 40 chars
```

---

## Empty-state message

When the knowledge dir doesn't exist or is empty:
```
Knowledge base not initialized. Run `sdlc knowledge librarian init` to seed from your project.
```
