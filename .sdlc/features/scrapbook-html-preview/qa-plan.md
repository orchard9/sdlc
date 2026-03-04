# QA Plan: Scrapbook renders *-mockup.html artifacts in an inline iframe

## Scope

Two frontend component changes verified via build check + manual browser test.
No Rust code changes — `cargo test` baseline confirms no regression.

---

## QC-1: TypeScript build passes

**Command:** `cd frontend && npm run build`

**Pass:** Zero TypeScript errors, `dist/` updated.

**Fail:** Any TS error in `WorkspacePanel.tsx` or `ArtifactContent.tsx`.

---

## QC-2: Rust tests pass

**Command:** `SDLC_NO_NPM=1 cargo test --all`

**Pass:** All tests pass.

**Fail:** Any test failure (indicates unintended side-effect — unlikely for frontend-only changes but must verify).

---

## QC-3: Clippy clean

**Command:** `cargo clippy --all -- -D warnings`

**Pass:** Zero warnings (frontend-only change, no Rust touched).

---

## QC-4: ArtifactContent iframe height

**Method:** Open browser DevTools, inspect the iframe rendered inside an HTML
artifact's expanded panel.

**Pass:** iframe has CSS `min-height: 300px` and `max-height: 384px` (24rem) in
normal mode; `min-height: 60vh` in the fullscreen modal.

**Fail:** iframe height unchanged from old values (256 px min / 320 px max).

---

## QC-5: WorkspacePanel — Monitor icon for HTML artifacts

**Method:** Navigate to a ponder entry that contains a `.html` scrapbook artifact
(e.g., `i-need-a-better-feedback-system-that-let`).

**Pass:** The artifact row shows a monitor/computer icon (not the generic file icon).

**Fail:** FileText icon still shown for HTML artifact.

---

## QC-6: WorkspacePanel — "preview" badge for HTML artifacts

**Method:** Same ponder entry as QC-5.

**Pass:** A small `preview` badge appears inline with the filename in the artifact row.

**Fail:** No badge shown.

---

## QC-7: Markdown artifacts visually unchanged

**Method:** Same ponder entry, look at `.md` artifacts.

**Pass:** Markdown artifacts still show the `FileText` icon and no `preview` badge.

**Fail:** Markdown artifacts affected by HTML icon/badge changes.

---

## QC-8: Expanded HTML panel renders mockup content

**Method:** Click a `.html` artifact row to expand it.

**Pass:** An iframe renders the HTML content of the mockup (visible design, not raw HTML source). Panel height is `max-h-96` (384 px).

**Fail:** Raw HTML displayed as text, or panel height remains 256 px.

---

## QC-9: Fullscreen HTML rendering (regression)

**Method:** Click the Maximize button on a `.html` artifact.

**Pass:** Fullscreen modal opens with the iframe occupying `min-h-[60vh]`,
showing the full mockup.

**Fail:** Fullscreen broken or height incorrect.
