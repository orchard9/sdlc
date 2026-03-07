# Design: Rich Onboarding Empty States

## Layout Structure

All empty states follow a single layout pattern embedded inline in each page component. No new shared component is introduced — the pattern is simple enough that inline JSX keeps each page self-contained.

```
┌─────────────────────────────────────────┐
│          max-w-xl mx-auto               │
│          px-6 py-10 space-y-8           │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │          HERO SECTION           │    │
│  │   [Icon in tinted 12x12 box]   │    │
│  │   "Tagline headline"            │    │
│  │   Description paragraph          │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │    HOW IT WORKS                 │    │
│  │   ┌──────────────────────────┐  │    │
│  │   │ [Icon] Title             │  │    │
│  │   │        Description       │  │    │
│  │   └──────────────────────────┘  │    │
│  │   ┌──────────────────────────┐  │    │
│  │   │ [Icon] Title             │  │    │
│  │   │        Description       │  │    │
│  │   └──────────────────────────┘  │    │
│  │   (3-4 cards)                   │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │   [Optional: extra section]     │    │
│  │   (Lifecycle/Verdicts strip)    │    │
│  └─────────────────────────────────┘    │
│                                         │
│  ┌─────────────────────────────────┐    │
│  │         CTA SECTION             │    │
│  │   [ Primary Button ]            │    │
│  │   "Or select from list..."      │    │
│  └─────────────────────────────────┘    │
│                                         │
└─────────────────────────────────────────┘
```

## Visual Tokens

| Token | Value |
|-------|-------|
| Icon container | `w-12 h-12 rounded-xl bg-primary/10` |
| Icon | `w-6 h-6 text-primary` |
| Headline | `text-xl font-semibold` |
| Body text | `text-sm text-muted-foreground leading-relaxed` |
| Section heading | `text-xs font-semibold uppercase tracking-wider text-muted-foreground` |
| Step card | `p-3 rounded-lg border border-border/50 bg-card/50` with `flex items-start gap-3` |
| Step icon | `w-4 h-4 mt-0.5 shrink-0 text-muted-foreground` |
| Step title | `text-sm font-medium` |
| Step desc | `text-xs text-muted-foreground mt-0.5 leading-relaxed` |
| CTA button | `px-4 py-2 text-sm font-medium bg-primary text-primary-foreground rounded-lg hover:bg-primary/90` |
| Select hint | `text-xs text-muted-foreground/40` (conditional on entries.length > 0) |

## Per-Page Content

### Ponder
- **Steps**: Numbered (1-4) using `rounded-full bg-primary/10 text-primary` circle badges
- **Extra section**: Lifecycle strip — Exploring -> Converging -> Committed with colored pill badges
- **CTA**: Two buttons — "Suggest an idea" (primary, with Sparkles icon) + "New idea" (secondary outline)

### Root Cause (Investigation)
- **Steps**: 4 icon-based cards (Target, Layers, FileText, CheckCircle2)
- **CTA**: "New Root Cause" button opening CreateWorkspaceModal

### Guidelines
- **Steps**: 4 icon-based cards (Search, Scale, FileEdit, BookOpen)
- **CTA**: "New Guideline" button opening CreateWorkspaceModal

### Spikes
- **Steps**: 3 icon-based cards (HelpCircle, Beaker, Scale)
- **Extra section**: Verdict strip showing ADOPT/ADAPT/REJECT colored pills with explanation
- **CTA**: CLI command block `/sdlc-spike <slug>` (monospace, muted background)

### Knowledge
- **Steps**: 3 icon-based cards (BookOpen, Search, Layers)
- **CTA**: CLI command block `sdlc knowledge add <slug>` (monospace, muted background)

## No Mockup Needed

The implementation is already in the working tree and has been visually verified. The pattern is consistent CSS utility classes with no complex layout logic.
