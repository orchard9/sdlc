# Design: Fullscreen View with Sticky TOC Navigation

## Overview

This design covers the implementation of a sticky left-rail TOC (Table of Contents) panel inside the fullscreen artifact modal. The feature touches two shared components: `MarkdownContent.tsx` and `FullscreenModal.tsx`.

No new files are introduced — this is an additive change to existing components with backward-compatible prop extensions.

## Component Architecture

```
FullscreenModal (hasToc=true)
  └── flex row container (max-w-5xl)
       ├── [desktop lg:] sticky nav rail (w-48)
       │    └── <a> links per heading
       ├── [mobile <lg:] "Jump to..." <select> dropdown (above content)
       └── article (flex-1 min-w-0)
            └── MarkdownContent (showToc=true)
                 └── react-markdown with h1/h2/h3 override (slug IDs)
```

## Props API

### `MarkdownContent`

```tsx
interface MarkdownContentProps {
  content: string;
  showToc?: boolean;   // NEW — enables heading ID assignment + TOC rail
}
```

### `FullscreenModal`

```tsx
interface FullscreenModalProps {
  hasToc?: boolean;    // NEW — widens container to max-w-5xl
}
```

Both props default to `false`/`undefined` — zero behavioral change for existing callers.

## Key Design Decisions

- Heading extraction via regex pass over raw content string (avoids second render pass)
- `slugify` function consistent between extraction and rendering: `text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '')`
- TOC layout lives inside `MarkdownContent`, not `FullscreenModal`
- `FullscreenModal` only needs `hasToc` flag to widen its container to `max-w-5xl`

## Styling

| Element | Classes |
|---|---|
| TOC wrapper (desktop) | `hidden lg:block w-48 shrink-0 sticky top-0 max-h-screen overflow-y-auto pr-4` |
| Dropdown (mobile) | `lg:hidden w-full mb-4 text-sm border rounded px-2 py-1` |
| Two-column wrapper | `flex flex-row gap-6` |
| Content article | `flex-1 min-w-0` |
| Modal container (with TOC) | `max-w-5xl` |

## Files Changed

- `frontend/src/components/shared/MarkdownContent.tsx`
- `frontend/src/components/shared/FullscreenModal.tsx`
- `frontend/src/components/features/ArtifactViewer.tsx`

## Out of Scope

- Scroll-spy (active section highlighting)
- TOC in non-fullscreen card view
- WorkspacePanel TOC (deferred to follow-up)
