# QA Plan: Dynamic Browser Tab Title

## Test Cases

### TC1: Default page title on Dashboard
- Navigate to `/`
- Verify `document.title` equals `{projectName} · Dashboard · Ponder`

### TC2: List page titles
- Navigate to `/milestones`, `/features`, `/ponder`, `/guidelines`, `/knowledge`, `/investigations`, `/spikes`
- Verify each sets title with correct focus label

### TC3: Detail page titles include slug
- Navigate to `/features/some-feature`
- Verify title equals `{projectName} · some-feature · Features · Ponder`

### TC4: Title updates on navigation
- Start on `/` (title = `... · Dashboard · Ponder`)
- Navigate to `/milestones`
- Verify title changes to `... · Milestones · Ponder`

### TC5: Fallback project name
- Before config loads, title should use "Ponder" as project name

### TC6: Hub mode
- In hub mode, title should be "Ponder Hub"

## Verification Method

Visual inspection via browser dev tools (`document.title`) or Playwright snapshot of tab title after navigation.
