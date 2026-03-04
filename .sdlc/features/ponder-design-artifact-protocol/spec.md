# Spec: Ponder Design Artifact Protocol — HTML mockups in ponder skill

## Problem

When an agent running the `/sdlc-ponder` skill is asked to produce a design artifact,
there is no convention governing the format. Agents currently produce Markdown prose
that *describes* a UI — requiring reviewers to mentally construct layouts. This is
high-friction and under-informative.

The ponder session for this feature (`ponder-design-improvements`) explored this fully
and established clear decisions. This feature implements those decisions.

## What We're Building

Add a **Design Artifact Protocol** section to the `sdlc-ponder` skill instruction
(`sdlc_ponder.rs`) that tells agents:

1. **When to produce HTML**: only for user-interface designs (screens, panels, modals,
   widgets, layouts, interaction flows). Non-UI designs (data schemas, CLI syntax, API
   contracts, algorithm sketches) remain Markdown.

2. **What the HTML must look like**: a concrete format spec using Tailwind CDN, dark
   theme, yellow prototype banner, 2–3 states shown, placeholder data, no external
   dependencies. The spec is already captured in
   `.sdlc/roadmap/ponder-design-improvements/design-artifact-protocol.md`.

3. **Filename convention**: `<descriptive-name>-mockup.html`

4. **Capture procedure**: write HTML to `/tmp/<name>-mockup.html`, then
   `sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html`

The change is additive and narrow — no existing behavior breaks. Agents producing
non-UI artifacts continue using Markdown.

## Scope

- Modify `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`:
  - Add a `### Design Artifact Protocol` subsection under `### Capturing artifacts` in
    `SDLC_PONDER_COMMAND` (the full Claude command template)
  - Add the same protocol (condensed) to `SDLC_PONDER_PLAYBOOK` (Gemini/OpenCode)
  - Add the same protocol (minimal) to `SDLC_PONDER_SKILL` (SKILL.md)

- The feature `create_design` action (state machine's design.md artifact) is **not in
  scope** — that is a separate feature (`design-create-action-html`) in the same
  milestone.

## Acceptance Criteria

1. `SDLC_PONDER_COMMAND` has a `### Design Artifact Protocol` section that includes:
   - The UI vs. non-UI trigger condition
   - The full HTML format spec (Tailwind CDN, dark theme, prototype banner, 2–3 states,
     placeholder data, self-contained)
   - The `<name>-mockup.html` filename convention
   - The two-step capture procedure (write to `/tmp/`, then `sdlc ponder capture`)
   - A "when NOT to use HTML" list (data models, CLI syntax, API contracts, algorithms)

2. `SDLC_PONDER_PLAYBOOK` has the same rules in condensed form (a step added to the
   capturing step).

3. `SDLC_PONDER_SKILL` has the same rules in minimal form (a single note added to the
   capture instruction).

4. `cargo build --all` passes with `SDLC_NO_NPM=1`.

5. `SDLC_NO_NPM=1 cargo test --all` passes.

6. `cargo clippy --all -- -D warnings` passes.

## Non-Goals

- Rendering HTML mockups in the scrapbook UI (tracked as open question)
- Changing `create_design` action behavior (separate feature)
- Any runtime Rust logic changes — this is a skill instruction text change only
