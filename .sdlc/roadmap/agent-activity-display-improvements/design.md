# Agent Activity Display — Design Decisions

## The Three Improvements

### 1. Subagent Input + Output Cards

**Current state:** Subagent events (subagent_started, subagent_completed, subagent_progress) are processed in `buildTimeSeries.ts` for the time-series heatmap but are completely skipped in `pairEvents.ts`. Zero visibility into what subagents were doing.

**Target state:** Each subagent invocation appears as a card in the activity feed showing:
- Input: the `description` field from `subagent_started` (this is the task prompt sent to the subagent)
- Output: whatever is available from `subagent_completed` (text result, or last_tool_name as fallback)

**Implementation plan:**

Add to `types.ts`:
```ts
export interface PairedSubagentBlock {
  kind: 'subagent'
  taskId?: string
  description?: string        // from subagent_started — the input
  result?: string             // from subagent_completed — the output
  last_tool_name?: string     // what the subagent last did
  elapsed_seconds?: number    // from subagent_progress events
  isError: boolean
}
```

Update `PairedEvent` union to include `PairedSubagentBlock`.

In `pairEvents.ts`:
- Track pending subagents in a `Map<string, PairedSubagentBlock>` keyed by task_id
- On `subagent_started`: create entry with description
- On `subagent_progress`: update elapsed_seconds and last_tool_name
- On `subagent_completed`: finalize with result text, flush to output
- On `user` event: also flush any remaining pending subagents (they completed between turns)

New component `SubagentCard.tsx`:
- Distinct visual identity: purple/violet left border (different from blue tool border)
- Shows description as input (collapsible if long, with markdown rendering)
- Shows result as output with markdown
- Shows elapsed time if available

### 2. Markdown Everywhere in Activity Feed

**Current state — places without markdown:**
| Component | Field | Current rendering |
|-----------|-------|-------------------|
| `AssistantTextBlock` | text | `whitespace-pre-wrap` |
| `ToolCallCard` | summary | `line-clamp-3 text-[10px]` plain |
| `RunResultCard` | text | `whitespace-pre-wrap` |
| `RunInitCard` | prompt | `whitespace-pre-wrap line-clamp-6` |

**Why not use MarkdownContent directly?**

`MarkdownContent` has: TOC sidebar, raw/rendered toggle button, mermaid block rendering, full heading hierarchy with IDs. This chrome is appropriate for artifact viewers but is wrong for compact activity feed cards. The raw/rendered toggle is especially disruptive — it changes the layout of individual event cards.

**Solution: `CompactMarkdown` component**

A thin wrapper (`~40 lines`) that uses the same ReactMarkdown config but strips the chrome:
- No TOC sidebar
- No raw/rendered toggle  
- No mermaid (subagents don't output diagrams in activity logs)
- Same prose/code styling as MarkdownContent

Apply in:
- `AssistantTextBlock` — full markdown
- `ToolCallCard.summary` — compact markdown for summary line
- `RunResultCard.text` — full markdown
- `RunInitCard.prompt` — full markdown (line-clamp via CSS still applicable)

⚑  Decided: Create CompactMarkdown, do NOT use MarkdownContent in activity feed cards. The chrome is wrong for compact cards.

### 3. RunInitCard Header: Row → Column

**Current layout:**
```
[🤖 Run started] [claude-sonnet-4-6] [12 tools] [· MCP: sdlc, playwright]
```

**Problem:** At typical sidebar widths, this wraps inconsistently. MCP server names can be long. The visual hierarchy is flat — 'Run started' is the same visual weight as 'MCP: sdlc'.

**Target layout (column):**
```
🤖 Run started
   claude-sonnet-4-6
   12 tools  
   MCP: sdlc, playwright
```

**Implementation:**
Change `RunInitCard` from a single `flex items-center gap-2` row to a `space-y-1` column where:
- First row: Bot icon + 'Run started' label
- Subsequent rows: each metadata item indented (`pl-5`) to align under the label text

⚑  Decided: Simple layout swap, no new components needed.

## Open Questions

?  Open: What does `subagent_completed` actually put in the `text` field? The RawRunEvent interface shows `text?: string` but we need to confirm the Rust event logger populates it for subagent_completed events. If not, we may only have `description` and `last_tool_name` for the output section.

?  Open: For ToolCallCard summary — should it be full markdown or just inline markdown (no block elements)? Summaries are short one-liners typically. Full markdown allows code spans but unlikely to have headers or lists.

## Implementation Scope

Three small, contained changes:
1. `types.ts` — add PairedSubagentBlock type (~10 lines)
2. `pairEvents.ts` — add subagent pairing logic (~30 lines)
3. `SubagentCard.tsx` — new component (~60 lines)
4. `CompactMarkdown.tsx` — new component (~40 lines)
5. Update 4 existing components to use CompactMarkdown
6. Update RunInitCard layout

No new routes, no new API endpoints, no Rust changes.
