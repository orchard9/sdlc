# QA Plan: CompactMarkdown Component

## Build verification

1. `cd frontend && npm run build` succeeds with no TypeScript errors
2. No new dependencies added to `package.json`

## Visual verification

1. Open the Ponder UI and trigger or view an agent run that produces markdown in its output (bold, code, lists)
2. Verify `AssistantTextBlock` renders markdown formatting (bold text is bold, inline code has monospace styling, lists render as bullets)
3. Verify `RunResultCard` renders result text with markdown formatting
4. Verify `RunInitCard` renders the prompt with markdown formatting
5. Verify `ToolCallCard` renders the summary with markdown formatting
6. Verify code fences render as monospace blocks without syntax highlighting colors
7. Verify links are clickable and open in new tabs
8. Verify visual density is comparable to the previous plain-text rendering — no excessive margins or font size jumps

## Regression checks

1. `MarkdownContent` (used in artifact viewers, vision/architecture pages) is unchanged
2. `AgentLog` and `AgentEventLine` remain plain-text (not affected)
3. Activity feed cards that have no markdown in their content still render cleanly (plain text content should look identical to before)

## Edge cases

1. Empty string input renders nothing (no empty wrapper divs)
2. Content with only whitespace renders nothing
3. Very long single-line content does not overflow the card boundary
4. Deeply nested lists render without breaking layout
