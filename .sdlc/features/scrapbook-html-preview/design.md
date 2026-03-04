# Design: Scrapbook renders *-mockup.html artifacts in an inline iframe

## Overview

Two small, targeted frontend changes to two existing components. No new components,
no backend work, no routing changes.

---

## Component Changes

### 1. `ArtifactContent` — taller iframe

**File:** `frontend/src/components/shared/ArtifactContent.tsx`

The only change is the non-fullscreen iframe height classes:

```diff
- fullscreen ? 'min-h-[60vh]' : 'min-h-64 max-h-80',
+ fullscreen ? 'min-h-[60vh]' : 'min-h-[300px] max-h-96',
```

This gives the iframe 300–384 px in the expanded panel instead of 256–320 px.
The `min-h-[300px]` ensures the iframe has enough vertical space to render
the mockup's prototype banner and first state without clipping.

---

### 2. `WorkspacePanel` — HTML artifact visual treatment

**File:** `frontend/src/components/ponder/WorkspacePanel.tsx`

#### 2a. Icon differentiation

Add `Monitor` to the lucide-react import:

```diff
- import { FileText, ChevronDown, X, Maximize2, ChevronLeft, ChevronRight } from 'lucide-react'
+ import { FileText, Monitor, ChevronDown, X, Maximize2, ChevronLeft, ChevronRight } from 'lucide-react'
```

In the artifact list row, choose the icon based on file extension:

```tsx
const isHtml = /\.(html|htm)$/i.test(artifact.filename)
// ...
{isHtml
  ? <Monitor className={cn('w-3.5 h-3.5 shrink-0 transition-colors', activeIndex === i ? 'text-primary' : 'text-muted-foreground/50')} />
  : <FileText className={cn('w-3.5 h-3.5 shrink-0 transition-colors', activeIndex === i ? 'text-primary' : 'text-muted-foreground/50')} />
}
```

#### 2b. Preview badge

Immediately after the filename `<span>`, add a conditional badge for HTML files:

```tsx
{isHtml && (
  <span className="shrink-0 text-[10px] bg-primary/10 text-primary px-1.5 py-0.5 rounded font-mono">
    preview
  </span>
)}
```

The badge sits between the filename and the size indicator in the row.

#### 2c. Expanded panel height

Change the expanded content panel's height constraint:

```diff
- <div className="overflow-auto max-h-64 px-3 py-2">
+ <div className="overflow-auto max-h-96 px-3 py-2">
```

This applies to all artifact types — giving more breathing room universally — but
primarily benefits the HTML iframe which previously clipped inside a 256 px box.

---

## Visual Description

**Artifact list (before):**
```
[FileText] feedback-threads-ui.html            42.3 KB   2h ago  [v]
```

**Artifact list (after):**
```
[Monitor]  feedback-threads-ui.html  [preview]  42.3 KB   2h ago  [v]
```

When expanded:
- Markdown: renders as before, in a 384 px max scrollable area (was 256 px)
- HTML: renders as a sandboxed iframe occupying 300–384 px, with the Maximize
  button for full 60vh review

---

## What Is Not Changing

- The Maximize button and fullscreen modal are unchanged — they are already the
  correct path for full design review.
- The artifact list sort order is unchanged.
- The pagination bar and keyboard navigation are unchanged.
- The backend returns HTML content already — no Rust changes.

---

## Testing Notes

Manual test path:
1. Open a ponder entry that has a `*-mockup.html` scrapbook artifact.
2. Verify the artifact row shows a `Monitor` icon and `preview` badge.
3. Click to expand — verify the iframe renders the mockup content at ~300–384 px.
4. Click Maximize — verify the fullscreen modal shows the iframe at 60vh.
5. Click a Markdown artifact — verify no icon or badge change; expands normally.
