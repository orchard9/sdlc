# QA Plan: File Browser Component

## Unit Tests

### useGitFiles hook
- Returns loading=true initially, then files after fetch resolves
- Sets error=true on fetch failure
- Polls at configured interval
- Pauses polling when document is hidden, resumes on visible
- Refetches on window focus

### StatusBadge component
- Renders correct letter for each status (M, A, D, R, C, ??)
- Applies correct color class for each status

### Filter logic
- Modified filter returns only status=M files
- All filter returns all non-clean files
- Staged filter returns only staged=true files
- Untracked filter returns only status=?? files
- Default filter is Modified

### Tree building
- Builds correct hierarchy from flat paths
- Sorts directories before files at each level
- Computes correct changedCount for each directory
- Handles single-level files (no directory)
- Handles deeply nested paths

### Keyboard navigation
- j/ArrowDown moves cursor down
- k/ArrowUp moves cursor up
- Cursor wraps or clamps at boundaries
- Enter triggers onSelect with current file
- f toggles view mode
- m/a/s/u set the corresponding filter
- Keys are ignored when panel is not focused

## Integration Tests

### Flat view rendering
- Renders file rows with correct paths and status badges
- Selected file has bg-accent styling
- Staged files show the staged dot indicator
- Clicking a file row calls onSelect

### Tree view rendering
- Directories render with chevron and name
- Clicking a directory toggles expansion
- Nested files appear indented under their directory
- Directory count badge shows correct number

### Filter switching
- Clicking filter buttons updates the active filter
- File list updates to show only matching files
- Count badge updates to reflect filtered count
- Cursor resets to 0 on filter change

### Loading and error states
- Shows skeleton rows when loading=true
- Shows error message with retry button when error=true
- Shows empty state when filtered list is empty

## Manual Verification

- Visual check: status badge colors match the design mockup
- Visual check: tree indentation looks correct and consistent
- Visual check: panel scrolls independently from the rest of the page
- Keyboard feel: navigation is responsive and does not conflict with browser shortcuts
