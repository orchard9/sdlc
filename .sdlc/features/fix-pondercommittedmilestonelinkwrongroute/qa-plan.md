# QA Plan

## Test 1: Milestone link uses correct route
1. Navigate to `/ponder/iterative-ponder` (or any committed ponder entry)
2. Inspect the "View Milestone" link
3. **Expected**: Link href is `/milestones/<slug>` (plural), not `/milestone/<slug>` (singular)
4. Click the link — should navigate to the milestone detail page, not a 404
