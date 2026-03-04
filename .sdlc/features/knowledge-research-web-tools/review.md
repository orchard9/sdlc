# Code Review: Knowledge research — web-capable agent with rewritten prompt

## Changes reviewed

**File:** `crates/sdlc-server/src/routes/knowledge.rs`

### T1: Tool injection (+2 lines, 1 line mutated)

```rust
-    let opts = sdlc_query_options(app.root.clone(), 20);
+    let mut opts = sdlc_query_options(app.root.clone(), 20);
+    opts.allowed_tools.push("WebSearch".into());
+    opts.allowed_tools.push("WebFetch".into());
```

- Correctly mirrors the identical pattern from `ask_knowledge` (same file, ~line 721).
- `let mut` is correct since we're mutating the struct.
- Tool names `"WebSearch"` and `"WebFetch"` match the names used by `ask_knowledge` — consistent.
- No risk of double-injection: `sdlc_query_options` does not include web tools by default.

### T2: Prompt rewrite

New prompt restructured around five numbered steps:

1. **Web search first** — instructs `WebSearch` + `WebFetch`, targeting 3–5 authoritative external sources.
2. **Local codebase context** — `Grep` + `Read` to find existing usage in the project.
3. **Synthesize and write `content.md`** — structured as `## External Findings`, `## Local Context`, `## Summary`.
4. **Update entry summary** — `sdlc knowledge update <slug> --summary "..."`.
5. **Log the session** — `sdlc knowledge session log <slug> --content "..."`.

- Function signature unchanged (`slug`, `title`, `topic`, `root` — same as before).
- Web-first ordering is correct: external knowledge is the primary improvement this feature delivers.
- URL citation requirement in External Findings section reduces hallucination risk.
- No new routes, structs, SSE variants, or data model changes.

## Automated checks

- `SDLC_NO_NPM=1 cargo test --all`: PASS — all 804 tests pass, 0 failures.
- `cargo clippy --all -- -D warnings`: PASS — zero warnings.

## Acceptance criteria verification

1. `research_knowledge` pushes `WebSearch` and `WebFetch` onto `opts.allowed_tools` — PASS.
2. `build_research_prompt` instructs web search first, local search second, synthesizes to `content.md`, updates summary, logs session — PASS.
3. Function signature unchanged — PASS.
4. All tests pass — PASS.
5. Clippy clean — PASS.

## Verdict

APPROVED — implementation matches the spec and design exactly. Minimal, targeted, and correct.
