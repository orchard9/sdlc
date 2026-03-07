## Implementation: Ponder Empty State Onboarding

Replaced the minimal empty state (lightbulb icon + 'Select an idea to explore' + 2 small buttons) with a rich onboarding experience in `frontend/src/pages/PonderPage.tsx`:

### What changed
- **Hero section**: 'Think before you build.' headline with primary-colored lightbulb icon and value proposition paragraph
- **How it works**: 4-step numbered guide — Seed an idea, Explore with AI thought partners, Capture what matters, Commit when ready
- **Lifecycle strip**: Visual pill badges showing Exploring → Converging → Committed with explanatory text
- **CTA buttons**: Prominent 'Suggest an idea' (primary) and 'New idea' (outlined) buttons
- **Contextual hint**: 'Or select an idea from the list to continue exploring' shown when entries exist

### Design decisions
- Content is scrollable (`h-full overflow-y-auto`) so it works at any viewport height
- Max-width constrained (`max-w-xl`) for readability
- Consistent with existing design language (card borders, primary colors, muted-foreground text)
- No new components or files — all inline in PonderPage.tsx detailPane