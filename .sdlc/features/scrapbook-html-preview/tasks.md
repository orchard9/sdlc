# Tasks: Scrapbook renders *-mockup.html artifacts in an inline iframe

## T1 — Update ArtifactContent iframe height

**File:** `frontend/src/components/shared/ArtifactContent.tsx`

Change the non-fullscreen iframe Tailwind classes from `'min-h-64 max-h-80'` to
`'min-h-[300px] max-h-96'`.

Acceptance: the iframe height range is 300–384 px in normal mode and unchanged
(`min-h-[60vh]`) in fullscreen mode.

---

## T2 — Add Monitor icon and Preview badge for HTML artifacts in WorkspacePanel

**File:** `frontend/src/components/ponder/WorkspacePanel.tsx`

1. Add `Monitor` to the lucide-react import.
2. In the artifact list row button, compute `isHtml` from the filename extension.
3. Render `Monitor` icon (instead of `FileText`) for HTML/HTM files.
4. Render a small `preview` badge (styled `bg-primary/10 text-primary text-[10px]
   px-1.5 py-0.5 rounded font-mono`) after the filename span for HTML/HTM files.
5. Change the expanded content panel height from `max-h-64` to `max-h-96`.

Acceptance: HTML artifact rows show Monitor icon + "preview" badge; Markdown rows
are visually unchanged; expanded HTML content has 384 px max height.

---

## T3 — Verify build passes

Run `npm run build` in `frontend/` and confirm zero TypeScript errors. Run
`SDLC_NO_NPM=1 cargo test --all` to confirm no Rust breakage.
