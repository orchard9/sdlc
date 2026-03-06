# Review: RunInitCard header — column layout

## Files Changed

- `frontend/src/components/runs/RunInitCard.tsx` — restructured header from row to column layout

## Findings

### 1. Correctness — PASS
The outer header `div` changed from `flex items-center gap-2` (single row) to `flex flex-col gap-1` (column stack). Icon + label occupy their own row; metadata badges sit in a second conditional row. Prompt section is untouched.

### 2. Conditional rendering — PASS
`hasMetadata` computed with `!!(raw.model || raw.tools_count != null || mcpList.length > 0)`. The `!= null` check correctly handles `tools_count === 0` as truthy (metadata row shown), while `null`/`undefined` hides it. The `!!` coercion ensures a boolean.

### 3. Alignment — PASS
Metadata row uses `pl-5` matching the prompt section's indentation, giving consistent left alignment for all sub-label content.

### 4. Overflow handling — PASS
`flex-wrap` on the metadata row prevents horizontal overflow with many MCP servers.

### 5. No regressions — PASS
No props changed, no new state, no API changes. Component interface identical. TypeScript compilation passes.

## Verdict

Approved — clean, minimal change that achieves the design goal.
