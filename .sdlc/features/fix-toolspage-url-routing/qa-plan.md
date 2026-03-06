# QA Plan: ToolsPage URL Routing

## Test Cases

### TC1: Direct URL navigation
- Navigate to `/tools/some-tool-name` directly
- Verify the tool is selected and its detail panel is shown
- Verify the sidebar highlights the correct tool

### TC2: Tool selection updates URL
- Navigate to `/tools`
- Click a tool in the sidebar
- Verify the URL changes to `/tools/<tool-name>`
- Verify the detail panel shows the selected tool

### TC3: Browser back/forward
- Navigate to `/tools`
- Select tool A → URL becomes `/tools/tool-a`
- Select tool B → URL becomes `/tools/tool-b`
- Press browser back → URL returns to `/tools/tool-a`, tool A is shown
- Press browser forward → URL returns to `/tools/tool-b`, tool B is shown

### TC4: Back button on mobile
- On mobile viewport, navigate to `/tools/some-tool`
- Tap the back arrow in the detail panel
- Verify URL changes to `/tools` and the tool list is shown

### TC5: Desktop auto-select
- On desktop viewport (>= 768px), navigate to `/tools`
- Verify URL is replaced with `/tools/<first-tool-name>` and the first tool is shown
- Verify this uses replace (back button does not return to bare `/tools`)

### TC6: Invalid tool name in URL
- Navigate to `/tools/nonexistent-tool-name`
- Verify the empty state / placeholder is shown (no crash, no redirect loop)

### TC7: Tool creation flow
- Create a new tool via the modal
- After build completes, verify URL updates to `/tools/<new-tool-name>`
- Verify the new tool is selected in the sidebar

### TC8: Existing functionality preserved
- Run a tool, verify results display correctly
- Open AMA panel, verify it works
- Open quality check, verify it works

## Verification Method
- Code review of changed files (App.tsx, ToolsPage.tsx)
- Manual browser testing of all test cases
- Verify no TypeScript compilation errors
