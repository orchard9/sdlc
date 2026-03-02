# QA Plan: knowledge-query-mode

## Scope

Verify `sdlc knowledge ask` end-to-end: CLI subcommand → server handler → agent spawn → SSE event → CLI output. Also verify error paths, concurrent guard, and unit-tested internal helpers.

## Test Environment

- `SDLC_NO_NPM=1 cargo test --all` must pass (no UI build required).
- A temp `.sdlc/` directory with at least one seeded knowledge entry for integration paths.
- Server integration tests run against the in-process axum router (no real network or Claude API calls needed for HTTP-layer tests).

---

## QC-1: Empty question rejected

**Type:** Automated integration test (`crates/sdlc-server/tests/integration.rs`)

**Steps:**
1. `POST /api/knowledge/ask` with `{"question": ""}`.

**Expected:** `400 Bad Request`, JSON body `{"error": "..."}` containing a descriptive message.

---

## QC-2: Valid question accepted and run started

**Type:** Automated integration test

**Steps:**
1. Create temp dir with empty `.sdlc/knowledge/` (no entries).
2. `POST /api/knowledge/ask` with `{"question": "What is our deployment process?"}`.

**Expected:** `202 Accepted`, JSON body `{"run_id": "<non-empty string>", "status": "started", ...}`.

---

## QC-3: Concurrent query returns 409

**Type:** Automated integration test

**Steps:**
1. `POST /api/knowledge/ask` with a valid question (agent run starts).
2. Immediately `POST /api/knowledge/ask` with another question.

**Expected:** Second request returns `409 Conflict`, body `{"error": "..."}`.

---

## QC-4: `build_ask_prompt` unit test

**Type:** Automated unit test (`crates/sdlc-server/src/routes/knowledge.rs` `#[cfg(test)]`)

**Steps:**
1. Call `build_ask_prompt` with a mock catalog (1 class, 1 division) and 2 entries.
2. Provide question `"How does branching work?"`.

**Expected:**
- Returned string contains catalog class name.
- Contains both entry titles.
- Contains question text verbatim.
- Contains the word "librarian" (from prompt instructions).

---

## QC-5: `extract_citations` unit test

**Type:** Automated unit test

**Steps:**
1. Build answer text containing `[100.20 trunk-based-dev]` and `[999.99 nonexistent]`.
2. Build entry list with only `trunk-based-dev` (code `100.20`, title `Trunk-based dev`).
3. Call `extract_citations(text, entries)`.

**Expected:**
- Returns a `Vec` with exactly one `CitedEntry` (`slug = "trunk-based-dev"`, `code = "100.20"`).
- The nonexistent slug is silently dropped.

---

## QC-6: `extract_gap` unit test — gap detected

**Type:** Automated unit test

**Steps:**
1. Construct text ending with:
   ` ```json\n{"gap": true, "suggestion": "deployment pipeline"}\n``` `
2. Call `extract_gap(text)`.

**Expected:** Returns `(true, Some("deployment pipeline"))`.

---

## QC-7: `extract_gap` unit test — no gap

**Type:** Automated unit test

**Steps:**
1. Call `extract_gap` with plain text containing no sentinel block.

**Expected:** Returns `(false, None)`.

---

## QC-8: SSE variants serialize correctly

**Type:** Automated unit test (or snapshot test)

**Steps:**
1. Serialize `SseMessage::KnowledgeQueryStarted { question: "hello".into() }` to JSON.
2. Serialize `SseMessage::KnowledgeQueryCompleted { answer: "ans".into(), cited_entries: vec![], gap_detected: false, gap_suggestion: None }` to JSON.

**Expected:**
- First JSON contains `"type": "KnowledgeQueryStarted"` and `"question": "hello"`.
- Second JSON contains `"type": "KnowledgeQueryCompleted"`, `"answer": "ans"`, `"cited_entries": []`, `"gap_detected": false`.

---

## QC-9: CLI `--json` flag output schema

**Type:** Manual smoke test (or automated via CLI subprocess in integration test)

**Steps:**
1. Start `sdlc ui` with a seeded knowledge base.
2. Run `sdlc knowledge ask "What is trunk-based development?" --json`.
3. Capture stdout.

**Expected:**
- Valid JSON matching:
  ```json
  {
    "answer": "<string>",
    "cited_entries": [{"slug": "...", "code": "...", "title": "..."}],
    "gap_detected": <bool>,
    "gap_suggestion": <string|null>
  }
  ```

---

## QC-10: CLI server-not-running error message

**Type:** Manual smoke test

**Steps:**
1. Ensure `sdlc ui` is not running on the default port.
2. Run `sdlc knowledge ask "anything"`.

**Expected:** Output contains a human-readable message mentioning `sdlc ui` (not a panic or unintelligible error).

---

## QC-11: Cargo tests pass, clippy clean

**Type:** Automated CI gate

**Steps:**
```bash
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

**Expected:** Zero test failures, zero clippy warnings.

---

## Pass Criteria

All of QC-1 through QC-8 must be automated (run via `cargo test`). QC-9 and QC-10 are manual smoke tests sufficient for the first release. QC-11 must pass in CI before merge.
