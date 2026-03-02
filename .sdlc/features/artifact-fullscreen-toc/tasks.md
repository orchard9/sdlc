# Tasks: Fullscreen View with Sticky TOC Navigation

## T1 — Add `slugify` utility and `extractHeadings` function to `MarkdownContent.tsx`

Add a module-level `slugify(text: string): string` function and an `extractHeadings(content: string)` function that returns `{ level: number; text: string; id: string }[]`. These are pure functions with no side effects.

**File:** `frontend/src/components/shared/MarkdownContent.tsx`

## T2 — Add `showToc` prop and override heading components with slug IDs

Extend the `MarkdownContentProps` interface with `showToc?: boolean`. When `showToc` is true, override the `h1`, `h2`, and `h3` components in the `react-markdown` `components` prop to assign `id={slugify(String(children))}` to each heading element.

**File:** `frontend/src/components/shared/MarkdownContent.tsx`

## T3 — Render sticky TOC rail (desktop) and "Jump to..." dropdown (mobile)

When `showToc` is true and `headings.length > 0`, render a sticky nav rail for desktop and a `<select>` dropdown for mobile. Wrap output in a `flex flex-row gap-6` container.

**File:** `frontend/src/components/shared/MarkdownContent.tsx`

## T4 — Add `hasToc` prop to `FullscreenModal.tsx` and widen container

Extend `FullscreenModalProps` with `hasToc?: boolean`. When `hasToc` is true, use `max-w-5xl` for the modal content container; otherwise keep the existing `max-w-4xl`.

**File:** `frontend/src/components/shared/FullscreenModal.tsx`

## T5 — Wire `showToc={true}` and `hasToc={true}` at the artifact fullscreen call site

Pass `showToc={true}` to `MarkdownContent` and `hasToc={true}` to `FullscreenModal` at the fullscreen render site in `ArtifactViewer.tsx`.

**Files:** `frontend/src/components/features/ArtifactViewer.tsx`
