# Design: RunInitCard header — column layout

## Current Structure

```
┌─────────────────────────────────────────────────────┐
│ 🤖 Run started  claude-opus-4  12 tools · MCP: x,y │
│      Prompt text here...                            │
└─────────────────────────────────────────────────────┘
```

All items in a single `flex items-center gap-2` row.

## New Structure

```
┌──────────────────────────────────────────┐
│ 🤖 Run started                           │
│      claude-opus-4  12 tools · MCP: x,y  │
│      Prompt text here...                  │
└──────────────────────────────────────────┘
```

### Layout

```
div.flex.flex-col.gap-1           ← outer: column stack
  ├─ div.flex.items-center.gap-2  ← row 1: icon + label
  │    ├─ Bot icon
  │    └─ "Run started" span
  ├─ div.flex.items-center.gap-2.pl-5.flex-wrap  ← row 2: metadata (conditional)
  │    ├─ model badge (if present)
  │    ├─ tools count (if present)
  │    └─ MCP list (if present)
  └─ div.pl-5                     ← prompt (unchanged, conditional)
       └─ prompt text
```

- The metadata row uses `pl-5` to align with the label text (same indent as the prompt section).
- The metadata row is only rendered if any metadata is present (`raw.model || raw.tools_count != null || mcpList.length > 0`).
- `flex-wrap` on metadata row prevents overflow if many MCP servers are listed.

## Complexity

Minimal — restructuring JSX within a single component, no new props, no new state, no API changes.
