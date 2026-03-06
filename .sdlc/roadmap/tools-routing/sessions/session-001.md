---
session: 1
timestamp: 2026-03-05T00:00:00Z
orientation:
  current: "Routing is already implemented via /tools/:name. Gaps: no not-found state, param named :name not :toolId (cosmetic). No UUID needed — name is stable."
  next: "Implement: rename route param to :toolId, add not-found state when URL param doesn't match any tool"
  commit: "When the two implementation gaps are fixed, this is complete — no further exploration needed"
---

## Context Read

Read ToolsPage.tsx, App.tsx, and Rust ToolMeta struct. Here's what exists:

- `App.tsx` routes: `/tools` and `/tools/:name` → both render `<ToolsPage />`
- `ToolsPage` reads `useParams<{ name?: string }>()`, derives `selectedTool = tools.find(t => t.name === name)`
- Selecting a tool: `navigate(\`/tools/${tool.name}\`)` → URL updates
- Load function auto-selects first tool on desktop if no name in URL
- `ToolMeta` has `name` (slug, unique, filesystem-backed) and `display_name`

## Team Perspectives

**Priya Sharma · Routing Architect (ex-Linear, Notion)**
URL slugs as IDs are fine when immutable. Tool names don't change — no rename endpoint, evolve is in-place, clone creates a new name. A UUID layer adds no value here. The real gaps are:
1. No not-found handling when URL param doesn't match any tool
2. Route param named `:name` instead of `:toolId` — cosmetic but worth fixing for readability

**Marcus Chen · Daily User / Skeptic**
Deep linking works — copy-paste `/tools/quality-check` opens the right tool. Loading state resolves correctly. The main annoyance would be if you had a bookmarked URL for a tool that was deleted — you'd get a silent empty state rather than a clear message.

**Danielle Osei · End-User Advocate**
Shareability is important. Current routing supports it. What's missing: if a shared link is stale (tool deleted or renamed, which currently can't happen), there's no helpful error. The "select a tool" empty state on the right panel is confusing if you arrived via a URL that should have pointed somewhere.

## Analysis

⚑  Decided: Tool `name` IS the stable ID. No UUID system needed. Names are immutable (no rename endpoint), filesystem-backed, and human-readable.

⚑  Decided: The two actionable gaps are:
  1. Route param rename: `:name` → `:toolId` in App.tsx (and update `useParams` extraction in ToolsPage)
  2. Not-found state: when `name` param is set but `selectedTool` is null, show "Tool not found" message in right pane

?  Open: Should "not found" redirect to `/tools` or show an inline message? Inline message feels better — it's informative without losing context.

## Captured artifacts

- `brief.md` — user's original request

## Product Summary

### What we explored
Whether the `/tools` section needs a new ID-based routing system or just cleanup of the existing `:name`-based routing. We read the full implementation to understand what already exists.

### Key shifts
The routing IS already implemented — deep linking, browser history, and mobile back navigation all work. The request isn't about adding routing but about two small gaps: no 404-style message when a URL doesn't match a tool, and a cosmetically misnamed route param.

### Implications
This is a small implementation task (2 targeted changes), not an architectural decision. No new ID field or UUID system is needed for the backlog. The work can be done in a single focused feature.

### Still open
Should the not-found state redirect to `/tools` or show an inline "Tool not found" message in the detail pane? Decision needed before implementation.
