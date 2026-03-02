# Design: knowledge-cli-ingest

## CLI module structure

Follows `investigate.rs` exactly:
- `pub enum KnowledgeSubcommand` (clap Subcommand)
- `pub fn run(root, subcmd, json) -> anyhow::Result<()>`
- Private handler functions per subcommand
- `fn entry_to_json_summary` / `fn entry_to_json_full` helpers
- `fn slugify_title(title: &str) -> String` for slug derivation

## Server routes structure

Follows `investigations.rs` exactly:
- Each handler is `pub async fn` taking `State<AppState>` + optional `Path` / `Json`
- Blocking I/O wrapped in `tokio::task::spawn_blocking`
- Errors return `AppError`

## URL fetch for `--from-url`

Use `ureq` (sync HTTP) with a 10-second timeout. Extract `<title>` tag via string search.
If fetch fails: log a warning, continue without page title (don't fail the command).
Extracted page title stored in content.md as: `Fetched from: <url>\n\nPage title: <title>`

## Slug derivation

```rust
fn slugify_title(title: &str) -> String {
    let s = title.to_lowercase();
    let s = regex_replace_all(non-alnum, "-");
    let s = trim dashes;
    truncate to 40;
}
```
Uses a simple loop (no regex dep) since we control the input.

## Catalog add routing

`catalog add` determines class vs division by counting dots in code:
- No dot → `knowledge::add_class`
- One dot → `knowledge::add_division`

## Query params for list

Server `list_knowledge` handler reads `?code=` and `?tag=` from `axum::extract::Query`.
