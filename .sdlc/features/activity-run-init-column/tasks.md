# Tasks: RunInitCard header — column layout

## T1: Restructure RunInitCard header to column layout

Refactor the JSX in `RunInitCard.tsx`:
- Change outer header div from `flex items-center gap-2` to `flex flex-col gap-1`
- Move Bot icon + "Run started" label into their own `flex items-center gap-2` row
- Group model badge, tools count, and MCP list into a conditional metadata row with `flex items-center gap-2 pl-5 flex-wrap`
- Prompt section remains unchanged
