# QA Plan: tools-route-param-and-notfound

## Scenarios

### 1. Known tool deep link
- Navigate directly to `/tools/quality-check`
- Expected: tool opens, run panel visible, sidebar highlights quality-check

### 2. Unknown tool deep link
- Navigate directly to `/tools/nonexistent-tool`
- Expected: right pane shows "Tool 'nonexistent-tool' not found.", sidebar shows full tool list

### 3. Tool selection updates URL
- Go to `/tools`, click any tool in the sidebar
- Expected: URL changes to `/tools/<tool-name>`, tool panel opens

### 4. Mobile back navigation
- On narrow viewport, select a tool, press back
- Expected: returns to tool list view

### 5. Load from /tools with no selection
- On desktop viewport, navigate to `/tools`
- Expected: auto-selects first tool, URL updates to `/tools/<first-tool-name>`
