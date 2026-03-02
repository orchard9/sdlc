# QA Plan: knowledge-cli-ingest

## Test Strategy

Run the full test suite with `SDLC_NO_NPM=1 cargo test --all` and confirm clippy clean with `cargo clippy --all -- -D warnings`.

## Verification Checklist

### CLI: sdlc knowledge status
- [ ] Prints "not initialized" message when `.sdlc/knowledge/` does not exist
- [ ] Prints summary table with entry count, catalog category count, last-maintained when initialized

### CLI: sdlc knowledge add
- [ ] `--title` creates an entry with code=uncategorized, status=draft
- [ ] `--code 100` creates an entry with the given code
- [ ] `--content "text"` writes inline text to content.md
- [ ] `--from-file <path>` reads file content into content.md
- [ ] `--from-url <url>` fetches page title via HTTP and writes to content.md (best-effort, no failure on bad URL)
- [ ] Slug is derived correctly from title (lowercase, non-alnum → `-`, truncated at 40)
- [ ] Duplicate slug returns an error

### CLI: sdlc knowledge list
- [ ] Shows CODE, TITLE, SUMMARY, STATUS, UPDATED columns
- [ ] Empty state prints init hint
- [ ] `--code-prefix 100` filters by code prefix
- [ ] `--tag <tag>` filters by tag
- [ ] `--status draft` filters by status
- [ ] `--json` outputs machine-readable JSON

### CLI: sdlc knowledge show <slug>
- [ ] Default output: code badge, title, summary, tags, sources, then content.md as plain text
- [ ] `--json` returns full entry fields + content field
- [ ] Unknown slug prints error

### CLI: sdlc knowledge search '<query>'
- [ ] Matches in title, summary, tags, and content.md
- [ ] Empty knowledge base prints init hint
- [ ] `--json` returns JSON array of results with excerpts

### CLI: sdlc knowledge update <slug>
- [ ] `--code` updates code in YAML only (no directory rename)
- [ ] `--status published` updates status
- [ ] `--tag t` appends to tags (does not replace)
- [ ] `--related r` appends to related list

### CLI: sdlc knowledge catalog show
- [ ] Prints taxonomy tree
- [ ] Empty catalog prints hint

### CLI: sdlc knowledge catalog add
- [ ] `--code 100 --name "Engineering"` adds a top-level class
- [ ] `--code 100.20 --name "Architecture"` adds a division under class 100

### CLI: sdlc knowledge session log/list/read
- [ ] `session log <slug>` increments session count
- [ ] `session list <slug>` lists sessions
- [ ] `session read <slug> 1` reads first session

### Server Routes
- [ ] `GET /api/knowledge/catalog` returns catalog JSON
- [ ] `GET /api/knowledge` returns entry list
- [ ] `GET /api/knowledge?code=100` filters by code prefix
- [ ] `GET /api/knowledge?tag=rust` filters by tag
- [ ] `POST /api/knowledge` creates an entry
- [ ] `GET /api/knowledge/:slug` returns entry + content
- [ ] `PUT /api/knowledge/:slug` updates entry
- [ ] `POST /api/knowledge/:slug/capture` captures artifact
- [ ] `GET /api/knowledge/:slug/sessions` lists sessions
- [ ] `GET /api/knowledge/:slug/sessions/:n` reads session

## Automated Checks

```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

Expected: all tests pass, no clippy warnings.
