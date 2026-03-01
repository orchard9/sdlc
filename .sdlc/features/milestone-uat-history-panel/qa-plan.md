# QA Plan: UatHistoryPanel

## Manual Verification

1. **Renders panel** — navigate to a milestone detail page; verify the "UAT History" section and `data-testid="uat-history-panel"` element are present in the DOM
2. **Empty state** — for a milestone with no UAT runs, verify the text "No UAT runs yet." is displayed
3. **Run list** — for a milestone with UAT runs, verify each run shows a verdict badge, date, test count, and tasks created count
4. **Verdict colors** — verify green badge for `pass`, amber for `pass_with_tasks`, red for `failed`
5. **Sort order** — verify runs are sorted most-recent-first
6. **Zero tasks omission** — verify the tasks created count is omitted when `tasks_created` is empty
7. **TypeScript** — run `npm run build` in `frontend/` and confirm no type errors
