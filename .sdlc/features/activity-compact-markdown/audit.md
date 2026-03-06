# Audit: CompactMarkdown Component

## Security surface

This feature replaces plain-text rendering with markdown-aware rendering in the frontend activity feed. The security surface is the markdown content passed through `react-markdown`.

## Findings

### 1. XSS via markdown content — LOW RISK, MITIGATED

**Concern:** Agent output rendered as markdown could contain script injection attempts.

**Mitigation:** `react-markdown` does not render raw HTML by default. The `rehype-raw` plugin (which would enable raw HTML passthrough) is not used. All output is generated through React's JSX, which auto-escapes content. Links use `target="_blank"` with `rel="noopener noreferrer"` to prevent reverse tabnapping.

**Action:** Accept — `react-markdown` default behavior is safe. No `rehype-raw` plugin is imported or configured.

### 2. Link targets — LOW RISK, MITIGATED

**Concern:** Markdown links in agent output could point to malicious URLs.

**Mitigation:** Links render with `target="_blank" rel="noopener noreferrer"`. The content source is agent output (server-generated), not user input. The risk is equivalent to the existing link rendering in `MarkdownContent`.

**Action:** Accept — same pattern as the existing `MarkdownContent` component.

### 3. No new dependencies — NO RISK

No new npm packages were added. The component uses `react-markdown` and `remark-gfm` which are already in the dependency tree.

**Action:** Accept.

### 4. No server-side changes — NO RISK

This is a frontend-only change. No API endpoints, authentication, or data storage are modified.

**Action:** Accept.

## Verdict

No actionable security issues. All findings are low risk with existing mitigations.
