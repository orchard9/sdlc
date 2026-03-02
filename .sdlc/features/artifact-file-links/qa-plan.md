# QA Plan: Auto-detect File Paths as IDE Links

## Test Scope

Covers the three-layer change: config struct, server API, and frontend rendering.

---

## Unit Tests

### `sdlc-core` — Config settings

**TC-1: `Settings` struct deserializes with defaults**
- Create a `Config` with no `settings` field in YAML
- Assert `config.ide_uri_scheme()` returns `"vscode"`

**TC-2: `Settings.ide_uri_scheme` is read from YAML**
- Config YAML includes `settings: { ide_uri_scheme: cursor }`
- Assert `config.ide_uri_scheme()` returns `"cursor"`

**TC-3: `Settings` round-trips through serde**
- Serialize a `Config` with `settings = Some(Settings { ide_uri_scheme: "zed" })`
- Deserialize and assert the field is preserved

---

## Integration Tests

### `sdlc-server` — `GET /api/config`

**TC-4: `project_root` is present in response**
- Call `GET /api/config` on a running test server
- Assert the JSON response contains `"project_root"` as a non-empty string

**TC-5: `settings.ide_uri_scheme` defaults to `"vscode"` when absent from config**
- Config file has no `settings` key
- Response contains `settings: null` or omits the key, but `project_root` is present
- (The frontend defaults to `"vscode"` when `settings` is absent)

**TC-6: Configured `ide_uri_scheme` appears in response**
- Config file has `settings: { ide_uri_scheme: cursor }`
- Response `settings.ide_uri_scheme` is `"cursor"`

---

## Frontend Manual / Playwright Tests

### File path detection in MarkdownContent

**TC-7: Path in inline code renders as link**
- Render `MarkdownContent` with `content = "See \`crates/sdlc-core/src/feature.rs\`"`
- Provide `projectRoot = "/home/user/project"`, `ideUriScheme = "vscode"`
- Assert a `<a>` element exists with `href = "vscode://file//home/user/project/crates/sdlc-core/src/feature.rs"`
- Assert link text is `crates/sdlc-core/src/feature.rs`

**TC-8: Non-path inline code renders as code span**
- Render with `content = "Use \`max-h-96\` for the class"`
- Assert no `<a>` element; a `<code>` element contains `max-h-96`

**TC-9: Option-like token not linkified**
- Render with `content = "Returns \`Some(20)\`"`
- Assert no `<a>` element

**TC-10: Fenced code block is not linkified**
- Render with a fenced block containing a path on its own line
- Assert no `<a>` element is produced for that block's content

**TC-11: Dotfile path renders as link**
- Render with `content = "Edit \`.sdlc/config.yaml\`"`
- Assert `<a>` is rendered (starts with `.`, has `/`)

**TC-12: Cursor scheme produces correct URI**
- Provide `ideUriScheme = "cursor"`
- Render path inline code
- Assert `href` starts with `cursor://file/`

**TC-13: Empty `projectRoot` skips link rendering**
- Provide `projectRoot = ""`
- Render a valid file path inline code
- Assert no `<a>` element; code span is rendered

---

## Regression

**TC-14: Existing `MarkdownContent` usage — no visual regression**
- Render a full feature spec artifact (artifact content with no file paths)
- Assert all headings, paragraphs, and code blocks render normally
- Assert no links appear for non-path inline code

**TC-15: Mermaid and syntax-highlighted blocks unaffected**
- Render a markdown block with a mermaid diagram and a fenced `rust` block
- Assert neither block is affected by the file-path logic

---

## Acceptance Criteria Mapping

| Criterion | Test Cases |
|---|---|
| Inline file path renders as IDE link | TC-7 |
| Link URI uses configured scheme | TC-7, TC-12 |
| Non-path inline code stays as code span | TC-8, TC-9 |
| Fenced code blocks unaffected | TC-10, TC-15 |
| `cursor` scheme works | TC-12 |
| `project_root` in API response | TC-4 |
| Empty `project_root` degrades gracefully | TC-13 |
| No regression on existing rendering | TC-14, TC-15 |
