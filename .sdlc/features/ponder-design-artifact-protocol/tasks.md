# Tasks: Ponder Design Artifact Protocol

## T1 — Add Design Artifact Protocol to SDLC_PONDER_COMMAND

Add a `### Design Artifact Protocol` subsection to the `### Capturing artifacts` section
in the `SDLC_PONDER_COMMAND` constant in
`crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`.

The new subsection must include:
- Trigger condition: UI designs (screen, panel, modal, widget, layout, interaction flow)
  → HTML; non-UI designs (data models, CLI syntax, API contracts, algorithms) → Markdown
- Full HTML format spec: Tailwind CDN, dark bg-gray-950, yellow prototype banner, 2–3
  states, placeholder data, self-contained
- Filename convention: `<name>-mockup.html`
- Two-step capture procedure: write to `/tmp/<name>-mockup.html`, then
  `sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html`
- "When NOT to use HTML" list

Insert after the existing capture examples and before `### Recruiting additional partners`.

## T2 — Update SDLC_PONDER_PLAYBOOK

Extend step 5 of `SDLC_PONDER_PLAYBOOK` in the same file to include the HTML protocol
in condensed form:
- UI designs → HTML mockup (`<name>-mockup.html`), Tailwind CDN, dark theme, prototype
  banner, 2–3 states, write to `/tmp/` first
- Non-UI designs → stay Markdown

## T3 — Update SDLC_PONDER_SKILL

Extend the capture step of `SDLC_PONDER_SKILL` in the same file to include the HTML
protocol in minimal form (one or two lines):
- UI designs → HTML mockup, write to `/tmp/` first, then capture
- Non-UI designs → Markdown

## T4 — Verify build passes

Run:
```bash
SDLC_NO_NPM=1 cargo build --all
SDLC_NO_NPM=1 cargo test --all
cargo clippy --all -- -D warnings
```

All must pass with no errors or warnings.
