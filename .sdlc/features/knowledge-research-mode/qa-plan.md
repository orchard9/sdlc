# QA Plan: knowledge-research-mode

## 1. Automated Tests

### 1.1 Unit: `OriginKind::Research` round-trip (sdlc-core)

**Location:** `crates/sdlc-core/src/knowledge.rs` — module `tests`

| Step | Expected |
|---|---|
| `"research".parse::<OriginKind>()` | `Ok(OriginKind::Research)` |
| `OriginKind::Research.to_string()` | `"research"` |
| Serialize to YAML via serde | `research` (no quotes) |
| Deserialize YAML `research` | `OriginKind::Research` |

### 1.2 Integration: research HTTP endpoint

**Location:** `crates/sdlc-server/tests/integration.rs`

**Test: `test_knowledge_research_returns_202`**

| Step | Expected |
|---|---|
| POST `/api/knowledge/test-topic/research` with `{"topic":"test"}` | HTTP 202 |
| Response body `started` field | `true` |
| Response body `slug` field | `"test-topic"` |

**Test: `test_knowledge_research_creates_entry_if_missing`**

| Step | Expected |
|---|---|
| POST to slug that does not exist | 202; entry created in temp dir |
| GET `/api/knowledge/my-new-slug` after POST | 200 with entry data |

**Test: `test_knowledge_research_409_when_already_running`**

| Step | Expected |
|---|---|
| POST `/api/knowledge/busy-slug/research` twice rapidly | Second call returns 409 |
| Response body contains `"error"` key | `"research already running for this entry"` |

### 1.3 Cargo test suite

```bash
SDLC_NO_NPM=1 cargo test --all
```

All tests must pass. Zero regressions.

### 1.4 Clippy

```bash
cargo clippy --all -- -D warnings
```

Zero warnings.

---

## 2. Manual Smoke Test (requires running `sdlc ui`)

### 2.1 CLI fires and returns

```bash
sdlc knowledge research 'rust ownership model' --code '100.10'
```

Expected output (within ~1s):
```
Research started for 'rust-ownership-model'.
Watch progress in the UI or via: sdlc knowledge session list rust-ownership-model
```

### 2.2 Entry appears in knowledge list

```bash
sdlc knowledge list
```

Expected: `rust-ownership-model` present with status `draft`.

### 2.3 Session appears after agent completes

```bash
sdlc knowledge session list rust-ownership-model
```

Expected: at least one session listed after the agent run finishes.

### 2.4 Content written

```bash
sdlc knowledge show rust-ownership-model
```

Expected: non-empty content section below the `---` separator.

### 2.5 JSON flag

```bash
sdlc knowledge research 'another topic' --json
```

Expected output:
```json
{"slug":"another-topic","started":true}
```

### 2.6 Server-not-running error

Stop `sdlc ui`, then:
```bash
sdlc knowledge research 'anything'
```

Expected: error message containing `"is \`sdlc ui\` running?"`.

---

## 3. SSE Event Verification

Using a browser devtools `EventSource` or `curl -N http://localhost:<port>/api/events`:

| Trigger | Expected SSE event |
|---|---|
| POST `/api/knowledge/:slug/research` | `event: knowledge\ndata: {"type":"KnowledgeResearchStarted","slug":"..."}` |
| Agent run completes | `event: knowledge\ndata: {"type":"KnowledgeResearchCompleted","slug":"..."}` |

---

## 4. Regression Checklist

- [ ] Existing `sdlc knowledge` subcommands (status, add, list, show, search, update, catalog, session, librarian) still work.
- [ ] Existing SSE events (ponder, investigation, advisory, run) still serialize correctly.
- [ ] Knowledge file watcher still fires `Update` events when content.md changes.
- [ ] `SDLC_NO_NPM=1 cargo test --all` passes (no npm build needed).
