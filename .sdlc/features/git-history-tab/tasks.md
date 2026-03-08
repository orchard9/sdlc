# Tasks: History Tab UI with Compact Commit List

## Task List

1. **Create useGitLog hook** — Data-fetching hook for `GET /api/git/log` with pagination support, error handling, and loading state
2. **Create relativeTime utility** — Lightweight function to convert ISO timestamps to human-readable relative time strings
3. **Create GitHistoryTab component** — Commit list component with compact rows, skeleton loading, empty state, and error state
4. **Create GitPage shell** — Page component at `/git` with tab bar (History active, Files/Diff as placeholders)
5. **Wire routing and sidebar** — Add `/git` route to App.tsx and "Git" nav entry to Sidebar.tsx under "integrate" group
