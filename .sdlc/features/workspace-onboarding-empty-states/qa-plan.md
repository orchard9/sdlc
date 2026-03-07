# QA Plan: Rich Onboarding Empty States

## Scope
Verify that all five workspace pages (Ponder, Root Cause, Guidelines, Spikes, Knowledge) display rich onboarding content when no item is selected in the detail pane.

## Test Cases

### TC1: Ponder empty state renders correctly
- Navigate to `/ponder` with no slug
- Verify Hero section with Lightbulb icon and "Think before you build." tagline
- Verify 4-step "How it works" section (Seed, Explore, Capture, Commit)
- Verify Lifecycle strip (Exploring, Converging, Committed)
- Verify "Suggest an idea" and "New idea" CTAs are present and clickable

### TC2: Root Cause empty state renders correctly
- Navigate to `/investigations` with no slug
- Verify Hero section with Microscope icon and "Find the root cause." tagline
- Verify 4-step "How it works" section (Describe, Investigate, Synthesis, Action plan)
- Verify "New Root Cause" CTA button opens the create modal

### TC3: Guidelines empty state renders correctly
- Navigate to `/guidelines` with no slug
- Verify Hero section with ScrollText icon and "Codify what works." tagline
- Verify 4-step "How it works" section (Evidence, Principles, Draft, Publish)
- Verify "New Guideline" CTA button opens the create modal

### TC4: Spikes empty state renders correctly
- Navigate to `/spikes` with no slug
- Verify Hero section with FlaskConical icon and "Answer one question fast." tagline
- Verify 3-step "How it works" section (Ask, Investigate, Verdict)
- Verify Verdicts strip with ADOPT/ADAPT/REJECT pills
- Verify CLI command CTA is displayed

### TC5: Knowledge empty state renders correctly
- Navigate to `/knowledge` with no slug
- Verify Hero section with Library icon and "What the team knows." tagline
- Verify 3-step "How it works" section (Catalog, Research, Staleness)
- Verify CLI command CTA is displayed

### TC6: Empty states disappear when item selected
- For each page, click an item in the list (if any exist)
- Verify the empty state is replaced by the item detail pane
- Navigate back to no-selection state and verify empty state returns

### TC7: Visual consistency across pages
- Compare all five empty states side by side
- Verify consistent spacing (py-10 px-6), typography (text-xl headline), and card styling
- Verify icon containers all use the same tinted style (bg-primary/10)

### TC8: No TypeScript build errors
- Run `cd frontend && npx tsc --noEmit`
- Verify zero errors related to the modified page files
