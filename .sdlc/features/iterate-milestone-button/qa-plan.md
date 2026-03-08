# QA Plan: Iterate button on ReleasedPanel

## Test scenarios

### S1: Button visibility
- Navigate to a released milestone detail page
- Verify the "Iterate" button is visible in the actions row alongside "Re-run UAT"
- Navigate to an active (non-released) milestone — verify no Iterate button appears (ReleasedPanel not shown)

### S2: Modal pre-population
- Click the "Iterate" button on a released milestone
- Verify NewIdeaModal opens with:
  - Title = milestone title (no version suffix)
  - Slug = versioned slug (e.g. `milestone-slug-v2`)
  - Brief = contains "Iteration of milestone:", original vision text, and reflection prompt

### S3: Slug versioning
- If no prior ponder exists for the milestone slug, slug should be `{slug}-v2`
- If `{slug}-v2` already exists, slug should be `{slug}-v3`
- Verify slug is truncated to 40 characters maximum

### S4: Successful creation and navigation
- Fill in/confirm the modal and click "Create Idea"
- Verify a new ponder entry is created via the API
- Verify the browser navigates to `/ponder/{newSlug}`

### S5: Error handling
- If `api.getRoadmap(true)` fails, the modal should not open (or show an error)
- If ponder creation fails (409 conflict), the modal should display an error message

## Test approach

Manual verification through the running UI. The `nextIterationSlug` utility has its own unit tests in the `iterate-slug-utility` feature. This QA focuses on the integration of the button click flow.
