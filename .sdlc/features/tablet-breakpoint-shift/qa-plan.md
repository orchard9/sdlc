# QA Plan: Shift Layout Breakpoint md→lg

## Scope

Verify that the responsive layout switch happens at 1024px (not 768px), and that no regressions exist at mobile or desktop widths.

## Test Cases

### TC-1: Tablet width shows mobile UX (768px–1023px)

**Setup:** Open the application in a browser, resize viewport to 900px wide (or use DevTools device emulation at 768px, 900px, 1023px).

**Checks:**
- [ ] BottomTabBar is visible at the bottom of the screen
- [ ] Mobile top header (hamburger/back button + title) is visible
- [ ] Desktop sidebar is NOT visible
- [ ] AgentPanelFab (floating action button) is visible
- [ ] Tapping the FAB opens the agent panel drawer (not an inline panel)

### TC-2: Desktop width shows desktop UX (1024px+)

**Setup:** Resize viewport to 1024px, 1280px, 1440px.

**Checks:**
- [ ] Sidebar is always visible (no overlay needed)
- [ ] BottomTabBar is NOT visible
- [ ] Mobile top header is NOT visible
- [ ] Inline AgentPanel is shown when open
- [ ] Panel-open button appears when AgentPanel is collapsed

### TC-3: Mobile width (≤767px) still works

**Setup:** Resize viewport to 375px and 414px.

**Checks:**
- [ ] BottomTabBar is visible
- [ ] Mobile header is visible
- [ ] Sidebar is hidden by default, slides in when hamburger is tapped
- [ ] All interactions work

### TC-4: Page-level layout branches at correct threshold

**Pages to check at 900px and 1024px:** PonderPage, InvestigationPage, EvolvePage, GuidelinePage, SpikePage, KnowledgePage, ToolsPage, ThreadsPage

**At 900px (should show mobile layout):**
- [ ] List/detail split — bottom sheet for detail, not side panel
- [ ] Mobile back-button visible in page header

**At 1024px (should show desktop layout):**
- [ ] Side panel for detail shown inline
- [ ] No bottom sheets

### TC-5: Non-layout md: classes unchanged

**Setup:** Load Dashboard and FeaturesPage at 768px–1023px.

**Checks:**
- [ ] Dashboard milestone grid still shows 2 columns at 768px (md:grid-cols-2 not changed)
- [ ] FeaturesPage grid still shows 2 columns at 768px

### TC-6: Build passes

**Command:** `cd frontend && npm run build`
- [ ] Build completes without errors or warnings related to CSS

## Pass Criteria

All checked items pass. No visible regressions on desktop (≥1280px) or mobile (≤480px) layouts.
