# Acceptance Test: v23-ponder-rich-input

## Prerequisites
- sdlc-server running at http://localhost:7777
- Frontend loaded and accessible

## Checklist

### Feature 1: Ponder Creation Modal (ponder-creation-modal)

- [ ] **Step 1 — Modal opens from New Idea button**
  Navigate to `/ponder`. Click the "New Idea" button. Verify a centered modal dialog opens (not an inline form).

- [ ] **Step 2 — Modal has all required fields**
  Verify the modal contains: Title (text input, auto-focused), Slug (derived from title, editable), Description (multi-line textarea, at least 6 rows), and References (URL input list with add/remove).

- [ ] **Step 3 — Slug auto-derives from title**
  Type a title like "Test Rich Input Modal". Verify the slug field auto-populates with a lowercase-hyphenated version (e.g. "test-rich-input-modal").

- [ ] **Step 4 — Create ponder with description and references**
  Fill in: Title, Description (multi-line text), and at least one reference URL. Click "Create". Verify the modal closes and navigates to `/ponder/<slug>`.

- [ ] **Step 5 — Verify references.md artifact saved**
  Check that `.sdlc/roadmap/<slug>/references.md` exists and contains the URL(s) in markdown list format.

- [ ] **Step 6 — Escape closes modal without creating**
  Open the modal again, type a title, press Escape. Verify the modal closes and no new ponder entry was created.

### Feature 2: Ponder Agent Web Tools (ponder-agent-web-tools)

- [ ] **Step 7 — Verify web tools in ponder agent config**
  Inspect `start_ponder_chat` in `crates/sdlc-server/src/routes/runs.rs`. Confirm `WebSearch`, `WebFetch`, and Playwright MCP tools are included in allowed_tools and mcp_servers.

- [ ] **Step 8 — Server compiles without warnings**
  Confirm the running server built successfully (it's already running at localhost:7777).
