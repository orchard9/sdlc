## Summary

Add a sticky left-rail TOC to the fullscreen modal view of artifact content. When `MarkdownContent` renders in fullscreen mode, headings are assigned stable slug IDs and a TOC panel appears on the left for instant section navigation. In-panel (non-fullscreen) view is unchanged beyond the height cap removal (Feature 1). On narrow screens, TOC collapses to a "Jump to..." dropdown.

## Problem

After removing the height cap (Feature 1), a 300+ line plan artifact is fully visible — but still requires significant scrolling to navigate. A user who wants to jump to the "Implementation Notes" section of a 400-line spec has no navigation affordances. The fullscreen modal is a full-viewport reading context where a document navigation rail fits naturally.

## Solution

### `MarkdownContent.tsx` — Heading ID assignment and TOC extraction

Add a `showToc?: boolean` prop to `MarkdownContent`. When true:

1. **Heading ID assignment:** Override the `h1`, `h2`, `h3` components in `react-markdown`'s `components` prop to assign stable slug-based IDs:
   ```tsx
   const slugify = (text: string) =>
     text.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');

   // In components:
   h1: ({ children }) => <h1 id={slugify(String(children))}>{children}</h1>,
   h2: ({ children }) => <h2 id={slugify(String(children))}>{children}</h2>,
   h3: ({ children }) => <h3 id={slugify(String(children))}>{children}</h3>,
   ```

2. **TOC extraction:** Parse headings from raw content (pre-render) using a simple regex pass to build the TOC list without a full AST traversal:
   ```tsx
   function extractHeadings(content: string): { level: number; text: string; id: string }[] {
     const lines = content.split('\n');
     return lines
       .filter(l => /^#{1,3} /.test(l))
       .map(l => {
         const m = l.match(/^(#{1,3}) (.+)/);
         if (!m) return null;
         return { level: m[1].length, text: m[2].trim(), id: slugify(m[2].trim()) };
       })
       .filter(Boolean) as { level: number; text: string; id: string }[];
   }
   ```

3. **TOC component:** Rendered as a sticky left-rail panel when `showToc` is true:
   ```tsx
   // w-48 sticky left rail, text-sm, indented by heading level
   <nav className="hidden lg:block w-48 shrink-0 sticky top-0 max-h-screen overflow-y-auto pr-4">
     <p className="text-xs font-semibold uppercase tracking-wide text-muted-foreground mb-2">Contents</p>
     {headings.map(h => (
       <a
         key={h.id}
         href={`#${h.id}`}
         className={`block py-0.5 text-sm hover:text-primary ${h.level === 1 ? '' : h.level === 2 ? 'pl-3' : 'pl-6'} text-muted-foreground`}
         onClick={e => { e.preventDefault(); document.getElementById(h.id)?.scrollIntoView({ behavior: 'smooth' }); }}
       >
         {h.text}
       </a>
     ))}
   </nav>
   ```

4. **Narrow screen fallback (below `lg:`):** A `<select>` "Jump to..." dropdown above the content:
   ```tsx
   <select
     className="lg:hidden w-full mb-4 text-sm border rounded px-2 py-1"
     onChange={e => document.getElementById(e.target.value)?.scrollIntoView({ behavior: 'smooth' })}
     defaultValue=""
   >
     <option value="" disabled>Jump to...</option>
     {headings.map(h => <option key={h.id} value={h.id}>{h.text}</option>)}
   </select>
   ```

### `FullscreenModal.tsx` — Two-column layout when TOC is present

Add a `hasToc?: boolean` prop. When true, switch from single-column to two-column flex layout:

```tsx
<div className={`flex ${hasToc ? 'flex-row gap-6' : 'flex-col'} max-w-5xl mx-auto`}>
  {/* TOC is rendered inside MarkdownContent when showToc=true */}
  {children}
</div>
```

Pass `showToc={true}` and `hasToc={true}` when rendering artifact content in `FullscreenModal`. The modal's `max-w-4xl` becomes `max-w-5xl` to accommodate the TOC rail (48 + gap + content).

## Acceptance Criteria

- Clicking the fullscreen button on an artifact card opens the modal with a left-rail TOC (on desktop `lg:` and above)
- The TOC lists all `#`, `##`, `###` headings from the artifact content
- Clicking a TOC entry smooth-scrolls to the corresponding heading anchor
- H2 entries are indented relative to H1; H3 entries are further indented
- On a narrow viewport (simulated by resizing below `lg:` breakpoint), the left rail is hidden and a "Jump to..." dropdown appears above the content
- Artifacts with no headings show no TOC (empty headings array → render without TOC)
- The in-panel artifact card (non-fullscreen) is unchanged by this feature
- `WorkspacePanel.tsx` fullscreen also benefits automatically if it uses `MarkdownContent` with `showToc` — confirm and enable if applicable

## Files Changed

- `frontend/src/components/shared/MarkdownContent.tsx` — `showToc` prop, heading ID assignment, TOC extraction, TOC component
- `frontend/src/components/shared/FullscreenModal.tsx` — `hasToc` prop, two-column flex layout

## What is NOT in scope

- Active TOC item highlighting (scroll-spy — tracking which section is currently in viewport)
- TOC in the in-panel (non-fullscreen) card view
- Collapsible TOC sections (accordion tree)
- TOC persistence (remembering which sections were expanded)
