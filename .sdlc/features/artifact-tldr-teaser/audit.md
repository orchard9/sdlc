## Summary

This feature makes two categories of changes: (1) a pure client-side TypeScript addition to `ArtifactViewer.tsx` that extracts and displays a text teaser from already-loaded artifact content, and (2) additions to Rust `const &str` template strings embedded as instruction text in installed command files. Neither change introduces new API endpoints, network calls, authentication surface, data stores, or user input paths. The security attack surface is negligible.

## Scope

- `frontend/src/components/features/ArtifactViewer.tsx` — client-side only; reads from props
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_run.rs` — const string only
- `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — const string only

## Findings

### A1: extractTeaser processes artifact content client-side (ACCEPTED)

`extractTeaser` splits `artifact.content` (a string already delivered to the browser from the existing `/api/features/:slug` endpoint) and extracts a substring. No new data is fetched. The teaser is rendered as a plain text `<span>` — not via `dangerouslySetInnerHTML`. There is no XSS vector: the span is escaped by React's JSX string rendering.

Action: Accept. No vulnerability.

### A2: formatRelativeTime uses `Date.now()` and `new Date(iso)` (ACCEPTED)

If the server returns a malformed date string, `new Date(iso).getTime()` returns `NaN`, which propagates to `diff = NaN`, and all comparisons evaluate to false — the function returns `'over a month ago'` as a safe fallback. No crash, no exploit.

Action: Accept. Graceful degradation on malformed input.

### A3: Agent instruction text changes in sdlc_run.rs / sdlc_next.rs (ACCEPTED)

These are `const &str` values embedded in the compiled binary and written to `~/.claude/commands/sdlc-*.md` by `sdlc init`/`sdlc update`. The instruction text is not executed by Rust; it is read by AI agents as natural language. No injection surface, no privilege escalation.

Action: Accept. No security surface.

### A4: No new dependencies introduced (ACCEPTED)

`formatRelativeTime` is implemented without `date-fns` or any external library. `extractTeaser` is pure string manipulation. No new npm packages, no new Cargo crates.

Action: Accept.

## Verdict

No security findings requiring action. All surface is read-only client-side rendering of already-loaded string data. Feature may proceed to QA.
