# Tasks

## Task 1: Replace Prepare button with View Milestone link
- [ ] In `frontend/src/pages/PonderPage.tsx`, replace the committed-state block (lines ~509-533) that renders a "Prepare" button with a navigation link to `/milestone/<slug>`
- [ ] Use `useNavigate` or `<Link>` from react-router-dom
- [ ] Change icon from `Play` to `ArrowRight` (or similar)
- [ ] Change label from "Prepare" to "View Milestone"
- [ ] Remove the `startRun` call and associated `prepareKey`/`prepareRunning` variables
- [ ] Keep emerald styling for visual consistency
