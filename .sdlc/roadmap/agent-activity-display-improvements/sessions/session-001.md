---
session: 1
timestamp: 2026-03-03T06:45:00Z
orientation:
  current: "Design complete for all three improvements. Implementation scope is clear: CompactMarkdown component, SubagentCard + pairEvents update, RunInitCard column layout. One open question on subagent_completed text field population from Rust."
  next: "Create a feature (or implement directly). If implementing: start with RunInitCard column layout (simplest), then CompactMarkdown + component updates, then subagent pairing."
  commit: "Three targeted changes are fully designed. Ready to implement as a single feature or directly."
---

**jordan · Owner**
agent activity display improvements

currently we're displaying subagent input in agent activity. we want to show input and output

also any place in agent activity that we arent running markdown, we want to add it

finally, the header at the top (run started, model, tokens, mcp sdlc playwright) is in a row, it should be in a column

---

## Session 1 — Design Pass

### Context load

I read the full implementation: `RunActivityFeed`, `pairEvents.ts`, `RunInitCard`, `ToolCallCard`, `AssistantTextBlock`, `RunResultCard`, `MarkdownContent`, and the relevant types. The architecture is clean. Three independent improvements with no cross-dependencies.

---

### Improvement 1: Subagent Input + Output

**Ben Hartley:** The subagent time-series already exists — it's in the heatmap strip. So users _know_ subagents ran, they just can't see _what_ they ran or _what they returned_. That's the gap. The fix is mechanical: surface the subagent_started and subagent_completed events as cards in the feed, same way tool calls are surfaced. The design pattern is established.

The key design decision is visual identity. Subagents are different from tool calls — they're nested agent invocations, not function calls. They should have a distinct left-border color. I'd use violet/purple (blue is already tools, green is success, red is error). Something like `border-violet-500`.

**Dan Reeves:** Before designing a new card type, confirm what data is actually populated. The types show `description` and `last_tool_name` for subagent events. If `subagent_completed` doesn't populate a `text` field with the output, the "output" section of the card will be empty or just `last_tool_name`. Worth checking the Rust event logger before over-designing.

That's a fair point. Looking at `RawRunEvent`:
- `description?: string` — this appears on subagent events (likely the subagent prompt)
- `last_tool_name?: string` — the last tool the subagent called before completing
- `text?: string` — shared with assistant events; may or may not be set on subagent_completed

⚑  Decided: Show `description` as input. For output, show `text` if present, else show `last_tool_name` as a "last action" label. This covers both data-rich and data-sparse cases gracefully.

**Pairing logic in pairEvents.ts:** The current pairer uses a sequential scan with simple state (pendingTools array). Subagents are different — they can be interleaved with tool calls if task_ids don't serialize. The safe approach: use a `Map<string, PairedSubagentBlock>` keyed by task_id. On `subagent_started`, add to map. On `subagent_completed`, finalize and push to result. Flush remaining on `user` event or at end.

**Ben Hartley:** One more thing — the subagent card should be collapsible for the description input. Subagent prompts can be very long (an entire SDLC spec). Default collapsed for input, with a "show input" toggle. The output/result should be visible by default since that's what users care about.

⚑  Decided: SubagentCard = violet border, collapsible input (default collapsed), visible output.

---

### Improvement 2: Markdown Everywhere

**Ben Hartley:** The `whitespace-pre-wrap` in AssistantTextBlock is a problem. Claude's assistant responses are markdown — they have headers, bullet lists, code spans, bold. Without a renderer, the user is reading raw markdown syntax. That's noise. The fix is non-negotiable: render markdown in assistant text.

The question is what renderer to use. `MarkdownContent` is the project's existing renderer but it has significant chrome: a raw/rendered toggle, TOC sidebar, Mermaid rendering. For a compact activity card, that chrome is wrong. The toggle button alone changes the card height and layout — it's disruptive.

**Dan Reeves:** How often does assistant text in activity feed have headings? Probably rarely. The headings are in artifacts. The activity feed is more likely to have bullet lists, code spans, and bold/italic emphasis. A minimal renderer is fine — we don't need Mermaid or TOC support here.

⚑  Decided: Create `CompactMarkdown.tsx` — uses same ReactMarkdown + remark-gfm + SyntaxHighlighter config, but without TOC, raw toggle, and Mermaid. ~40 lines. Reuse the styling definitions from MarkdownContent (same className patterns).

Apply to:
1. `AssistantTextBlock.tsx` — primary beneficiary, replace `whitespace-pre-wrap`
2. `RunResultCard.tsx` — final output text
3. `RunInitCard.tsx` — the prompt/seed text
4. `ToolCallCard.tsx` summary — the one-liner result summary (inline markdown only, no block elements expected here)

**Ben Hartley on ToolCallCard summary:** The summary is already `text-[10px]` (10px font, tiny). Running ReactMarkdown on it will apply default block margins and heading styles that look wrong at that scale. For the summary line specifically, use a simpler approach: just render it as-is but strip any leading/trailing whitespace. Or use a `prose-xs` class that scales down all the markdown elements. The `CompactMarkdown` should accept a `size` prop: `'sm'` (default) and `'xs'` for the summary context.

⚑  Decided: CompactMarkdown gets an optional `compact` boolean prop that reduces font size and element margins for use in tight contexts like ToolCallCard summary.

---

### Improvement 3: RunInitCard Row → Column

**Ben Hartley:** This is the right call. The horizontal layout breaks at sidebar widths because MCP server names can be long strings. But more importantly, the hierarchy is wrong — "Run started" is a section header, and model/tools/MCP are metadata under it. They should read vertically, not compete horizontally.

**Dan Reeves:** This is a one-line layout change. Don't overthink it.

The change: replace the single `flex items-center gap-2` div with a `space-y-0.5` column. The first item stays as a `flex items-center gap-1.5` row (bot icon + "Run started" label). Each subsequent metadata item gets its own row with `pl-5` indent (to align under the label text, past the icon).

Result:
```
🤖  Run started
    [claude-sonnet-4-6]   ← blue badge
    12 tools
    MCP: sdlc, playwright
```

⚑  Decided: Simple layout swap, no abstraction needed, ~10 line change.

---

### Scope Summary

No new routes, no new API endpoints, no Rust changes. Six frontend files:

| File | Change |
|------|--------|
| `types.ts` | Add `PairedSubagentBlock` interface, extend `PairedEvent` union |
| `pairEvents.ts` | Add subagent pairing (task_id Map), ~30 lines |
| `SubagentCard.tsx` | New component, ~70 lines |
| `CompactMarkdown.tsx` | New component, ~50 lines |
| `AssistantTextBlock.tsx`, `RunResultCard.tsx`, `RunInitCard.tsx`, `ToolCallCard.tsx` | Use CompactMarkdown |
| `RunInitCard.tsx` | Column layout swap |
| `RunActivityFeed.tsx` | Add `case 'subagent': return <SubagentCard event={event} />` |

This is a single well-scoped feature. Could be committed as one PR titled "feat(activity): subagent cards, markdown rendering, column init header".

---

### Open Question

?  Open: Does the Rust event logger populate `text` on `subagent_completed` events? If not, the output section of SubagentCard will fall back to `last_tool_name`. This should be verified before implementation by checking `crates/sdlc-core/src/event_log.rs`.
