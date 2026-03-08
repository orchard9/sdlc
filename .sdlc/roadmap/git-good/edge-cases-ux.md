# Edge Cases & UX Details

## Edge Cases to Handle

| Scenario | Handling |
|----------|----------|
| No remote configured | Show green locally, subtle "no remote" note |
| Detached HEAD | Show branch as commit hash, yellow warning |
| Merge in progress | Red state, show "merge in progress" with file count |
| Rebase in progress | Red state, show "rebase in progress" with step N/M |
| Binary files | "Binary file changed" with size delta, no diff render |
| Very large diffs (>2000 lines) | Truncate with expandable "show more" |
| Renamed files | Show old→new path, diff content changes |
| Permission-only changes | Show but visually muted |
| Submodules | Single entry with dirty/clean indicator |
| Empty repo (no commits) | Special state: "initial commit pending" |
| Shallow clone | Status works normally, note limitations |

## Green State ("Zen Mode")

When everything is clean and synced, show something that rewards good hygiene:
- A small checkmark with a gentle pulse animation
- Tooltip: "All clean — nothing to commit, in sync with origin"
- Maybe rotate through small encouraging messages
- Keep it tasteful, not distracting

## Keyboard Shortcuts (Git Page)

| Key | Action |
|-----|--------|
| j/k | Navigate file list |
| Enter | Open file diff |
| [/] | Jump between diff hunks |
| f | Toggle flat/tree view |
| m | Filter: modified only |
| a | Filter: all files |
| s | Filter: staged only |

## Responsive Breakpoints

- Content area >= 900px: side-by-side diff
- Content area < 900px: unified diff (patch-style)
- Sidebar collapse at < 768px viewport (existing behavior)
- File browser panel: 280-350px, resizable

⚑ Decided: Modified-only as default file filter
⚑ Decided: 900px breakpoint for diff mode switch
? Open: Should the green state animation be configurable/dismissable?
? Open: Resizable file browser panel or fixed width?