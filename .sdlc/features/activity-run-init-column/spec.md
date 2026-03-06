# Spec: RunInitCard header — row layout to column layout

## Problem

The `RunInitCard` header currently arranges all metadata (icon, "Run started" label, model badge, tools count, MCP servers) in a single horizontal row using `flex items-center gap-2`. On narrow viewports or when multiple metadata items are present, this row can overflow or feel cramped.

## Solution

Change the RunInitCard header from a single-row flex layout to a column layout that stacks the primary label row and metadata badges vertically:

1. **Top row**: Bot icon + "Run started" label (unchanged)
2. **Second row**: Model badge, tools count, and MCP server list — displayed as a horizontal group of metadata chips below the label

This gives the card a cleaner vertical rhythm and avoids horizontal overflow when metadata items accumulate.

## Scope

- **File**: `frontend/src/components/runs/RunInitCard.tsx`
- Change the header container from `flex items-center gap-2` to `flex flex-col gap-1`
- Move the top-level icon + label into their own `flex items-center gap-2` sub-row
- Group model badge, tools count, and MCP list into a second `flex items-center gap-2 pl-5` sub-row (indented to align with the label text, matching the prompt indentation below)
- Only render the metadata row if at least one metadata item exists

## Out of Scope

- No changes to the prompt section
- No changes to RunActivityFeed or other components
- No responsive breakpoint logic — the column layout is the default at all sizes
