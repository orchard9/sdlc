# Design: Ponder Design Artifact Protocol

## Overview

This feature is a pure skill-instruction text change — no Rust logic, no data model
changes, no new routes. The change lives entirely in `sdlc_ponder.rs` as additions
to the three string constants that define the ponder skill variants.

## Change Location

**File:** `crates/sdlc-cli/src/cmd/init/commands/sdlc_ponder.rs`

Three constants need updates:
- `SDLC_PONDER_COMMAND` — full Claude Code slash command template
- `SDLC_PONDER_PLAYBOOK` — condensed Gemini/OpenCode playbook
- `SDLC_PONDER_SKILL` — minimal SKILL.md for generic agents

## SDLC_PONDER_COMMAND Change

The `### Capturing artifacts` section in `SDLC_PONDER_COMMAND` currently shows the
generic `sdlc ponder capture` invocation. After this change it gains a subsection
immediately after the existing capture examples:

```
### Design Artifact Protocol

When producing a design for a **user interface** — a screen, panel, modal, widget,
layout, or interaction flow — produce a self-contained HTML mockup, not a Markdown
description.

**Format:**
- `<!DOCTYPE html>` with Tailwind CDN (`<script src="https://cdn.tailwindcss.com"></script>`)
- `<body class="bg-gray-950 text-gray-100 p-8 font-mono">`
- Yellow prototype banner at the top:
  ```html
  <div class="text-xs text-yellow-400 border border-yellow-900 rounded px-3 py-1 inline-block mb-6">
    ⚠ Design Prototype — not production code
  </div>
  ```
- Show 2–3 key states (empty / populated; before / after; state A / state B) using
  tab buttons or labeled sections
- Placeholder data throughout — no real data, no complex animations
- Self-contained — no external dependencies beyond Tailwind CDN

**Filename:** `<descriptive-name>-mockup.html`
Examples: `dashboard-layout-mockup.html`, `thread-detail-mockup.html`

**Capture:**
```bash
# Write HTML to temp file first, then capture into scrapbook
sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html
```

**When NOT to use HTML:** data model designs, CLI command syntax, API contracts,
algorithm sketches — these remain Markdown with code blocks.
```

This new subsection is inserted after the existing capture invocation examples and
before `### Recruiting additional partners`.

## SDLC_PONDER_PLAYBOOK Change

In `SDLC_PONDER_PLAYBOOK`, step 5 currently reads:
> `When artifacts are ready: sdlc ponder capture <slug> --content "..." --as <name>.md`

It becomes:
> When artifacts are ready: `sdlc ponder capture <slug> --content "..." --as <name>.md`.
> For UI/layout designs (screens, panels, modals), produce a self-contained HTML mockup
> (`<name>-mockup.html`) using Tailwind CDN, dark theme (`bg-gray-950`), a yellow
> prototype banner, and 2–3 states. Write to `/tmp/<name>-mockup.html` first, then
> `sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html`.
> Non-UI designs (data schemas, CLI syntax, API contracts) stay as Markdown.

## SDLC_PONDER_SKILL Change

In `SDLC_PONDER_SKILL`, step 5 currently reads:
> `Capture with sdlc ponder capture <slug> --content "..." --as <name>.md`

It becomes:
> Capture with `sdlc ponder capture <slug> --content "..." --as <name>.md`.
> For UI/layout designs, produce HTML mockup (`<name>-mockup.html`): Tailwind CDN,
> dark bg-gray-950, yellow prototype banner, 2–3 states. Write to `/tmp/` first,
> then `sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html`.
> Non-UI designs stay Markdown.

## No Other Changes

- No Rust structs modified
- No CLI commands added or renamed
- No server routes changed
- No frontend changes
- `sdlc init` and `sdlc update` install the updated skill automatically via the existing
  `install_user_scaffolding()` call — no registration changes needed

## Test Plan

Build and clippy verify the string constants compile. No unit test changes needed since
this is prose content, not logic.
