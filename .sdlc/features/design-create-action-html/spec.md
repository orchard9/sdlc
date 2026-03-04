# Spec: Feature create_design action produces HTML mockup

## Problem

When the `create_design` directive is issued for a UI feature, agents currently produce
`design.md` with ASCII wireframes. ASCII art is hard to review at a glance and does not
give users or agents an interactive, browser-renderable picture of the intended UI. A
single self-contained HTML file — with inline CSS and no external dependencies — would
make UI intent immediately navigable in any browser and give reviewers a richer artifact
than text-only wireframes.

## Goal

When an agent executes the `create_design` action for a UI feature, it produces **two
files** in the feature directory:

1. `design.md` — the authoritative Markdown design document (unchanged from current
   behaviour), with a reference link to the companion HTML file.
2. `mockup.html` — a single self-contained HTML file (inline `<style>` and `<script>`,
   zero external dependencies) that renders the major screens or states of the UI.

The HTML mockup is a **companion artifact**, not a replacement. Approval of the design
artifact still gates on `design.md` only. The mockup is referenced from `design.md` and
lives alongside it.

## Scope

The change is entirely in the agent-facing instruction layer (slash commands and guidance
text). No changes are needed to:
- The state machine classifier (`rules.rs`, `types.rs`)
- The CLI (`sdlc artifact`, `sdlc next`, etc.)
- Server routes or the React frontend

The `create_design` rule in `rules.rs` already mentions HTML prototypes in its message
text; this feature upgrades that hint into an explicit deliverable with format guidance.

## Changes Required

### 1. `crates/sdlc-cli/src/cmd/init/commands/sdlc_next.rs` — instruction update

The "artifact creation" section in `SDLC_NEXT_COMMAND` must state that for a
`create_design` action on a UI feature:

- Write `design.md` as the primary artifact (existing requirement).
- Also write `mockup.html` — a self-contained HTML mockup in the same feature directory.
- Reference `mockup.html` from `design.md` with a relative link.
- The mockup must be zero-dependency (no CDN, no external fonts), valid HTML5, and
  viewable by double-clicking the file.

The same guidance must be added to `SDLC_NEXT_PLAYBOOK` (Gemini / OpenCode) and
`SDLC_NEXT_SKILL` (Agent Skills format).

### 2. `crates/sdlc-core/src/rules.rs` — `needs_design` message update

The message for rule `needs_design` should be updated to make the HTML mockup requirement
explicit rather than optional for UI features:

> "For UI features, produce `mockup.html` (self-contained, inline CSS/JS) alongside
> `design.md` and reference it from the design. For non-UI features, ASCII wireframes or
> plain Markdown are fine."

### 3. HTML mockup format contract

The following constraints must be documented in the instruction text so agents generate
consistent, reviewable mockups:

| Constraint | Rationale |
|---|---|
| Single `mockup.html` file; no external resources | Viewable offline, committable to git |
| All CSS inline (`<style>`) | No separate stylesheet dependency |
| All JS inline (`<script>`) if used | Navigation between views |
| Valid HTML5 doctype | Renders correctly in all browsers |
| Multiple named sections or `<div id="screen-*">` blocks | Covers distinct UI states/screens |
| Navigation bar or `<nav>` at the top to jump between screens | Reviewers can explore without a server |
| Representative color palette and typography (no pixel-perfect fidelity required) | Communicates intent; not a production asset |

## Out of Scope

- Changing the state machine to require a separate `mockup` artifact type — the mockup
  is a companion file, not a new artifact gate.
- Auto-generating the mockup from spec text (this is an agent responsibility via
  instructions).
- Validating that `mockup.html` exists before the design artifact can be approved (nice
  to have but intentionally deferred to keep this change minimal).

## Success Criteria

1. When an agent runs the `create_design` action and reads the `sdlc-next` directive, it
   finds unambiguous instructions to produce both `design.md` and `mockup.html`.
2. The message from `sdlc next --for <slug> --json` (the `message` field) explicitly
   mentions the HTML mockup requirement for UI features.
3. Existing non-UI design flows are unaffected — the guidance continues to allow waiving
   the mockup for non-UI changes.
4. A sample `mockup.html` can be opened by double-clicking it in a file manager with no
   server or internet required.
