# QA Plan: RunInitCard header — column layout

## Visual Verification

1. **All metadata present**: With model, tools count, and MCP servers all set — verify the card renders three rows: label, metadata, prompt.
2. **No metadata**: When `raw.model` is null, `tools_count` is null, and `mcpList` is empty — verify the metadata row is not rendered.
3. **Partial metadata**: With only model set (no tools, no MCP) — verify metadata row renders with just the model badge.
4. **No prompt**: When prompt is null — verify only label and metadata rows render.
5. **Long MCP list**: Multiple MCP servers — verify `flex-wrap` prevents horizontal overflow.

## Build Verification

6. `npm run build` in `frontend/` completes without errors.
7. TypeScript compilation passes with no type errors.
