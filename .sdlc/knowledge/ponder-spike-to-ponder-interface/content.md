# Plan: Spike Interface + Promote to Ponder

## Milestone: v36-spike-interface

**Title:** Spike Interface — surface spikes, promote to ponder, file rejects to knowledge

**Vision:** After running a spike, you can see all your spikes in the web UI, read their findings, and promote ADAPT spikes to a pre-seeded ponder workspace in one click. REJECT spikes are automatically filed to the knowledge base. Nothing is lost — spike learnings flow forward regardless of verdict.

**Acceptance test:** Run `sdlc spike promote <adapt-slug>`. Confirm a ponder is created with `spike-findings.md` and `open-questions.md` pre-loaded, and the spike's manifest records the ponder slug. Navigate to `/spikes` in the web UI and see the spike listed with its verdict badge. Click the spike to view findings. Click "Promote to Ponder" and confirm redirect to the ponder. Simulate a REJECT spike and confirm the findings are filed to the knowledge base.

---

## Features

### 1. spike-data-layer
**Title:** Spike data layer — list, load, promote, and reject-to-knowledge

Core Rust data layer in `sdlc-core/src/spikes.rs`. Dumb data only — no decisions.

Tasks:
- Add `SpikeEntry` struct: slug, title, verdict (ADOPT/ADAPT/REJECT/None), date, the_question
- Implement `list(root)` — reads `.sdlc/spikes/*/findings.md` headers, returns Vec<SpikeEntry>
- Implement `load_findings(root, slug)` — returns raw findings.md content
- Implement `promote_to_ponder(root, slug, ponder_slug_override)`:
  - Creates ponder entry (title from The Question)
  - Captures full findings as `spike-findings.md` in ponder scrapbook
  - Extracts `## Risks and Open Questions` section → `open-questions.md` in ponder scrapbook
  - Writes `ponder_slug` back into spike manifest (`.sdlc/spikes/<slug>/manifest.yaml`)
  - Returns the created ponder slug
- Implement `store_in_knowledge(root, slug)`:
  - Adds entry to knowledge base with code derived from spike slug
  - Attaches findings.md as the knowledge entry's content
  - Tags with `spike`, `rejected`
- Add spike manifest format: `.sdlc/spikes/<slug>/manifest.yaml` — verdict, date, ponder_slug (nullable), title, the_question
- Unit tests: parse findings.md headers, extract sections, promote creates correct ponder artifacts

### 2. spike-cli-commands
**Title:** Spike CLI — list, show, promote subcommands

New `sdlc spike` subcommand with three actions.

Tasks:
- Add `crates/sdlc-cli/src/cmd/spike.rs` with subcommands:
  - `sdlc spike list` — tabular output: slug | verdict | date | title (truncated)
  - `sdlc spike show <slug>` — print full findings.md content
  - `sdlc spike promote <slug> [--as <ponder-slug>]` — calls promote_to_ponder, prints ponder slug on stdout
- Register `spike` subcommand in `crates/sdlc-cli/src/main.rs`
- `sdlc spike promote` confirms success: "Promoted to ponder '<ponder-slug>'. Run: /sdlc-ponder <ponder-slug>"

### 3. spike-rest-routes
**Title:** Spike REST routes — list, detail, and promote endpoints

Server-side REST layer.

Tasks:
- Add `crates/sdlc-server/src/routes/spikes.rs`:
  - `GET /api/spikes` — list with SpikeEntry fields
  - `GET /api/spikes/:slug` — manifest + full findings content
  - `POST /api/spikes/:slug/promote` — body: `{ ponder_slug?: string }` — calls promote_to_ponder, returns `{ ponder_slug }`
  - `POST /api/spikes/:slug/reject-to-knowledge` — calls store_in_knowledge
- Register routes in `crates/sdlc-server/src/routes/mod.rs` and main router
- Add SSE event: `SpikePromoted { spike_slug, ponder_slug }` emitted on promote

### 4. spike-ui-page
**Title:** Spike UI — list, detail, and promote views with sidebar nav

Frontend SpikePage with three views matching the design mockup.

Tasks:
- Add `frontend/src/pages/SpikePage.tsx`:
  - List view: spike rows with verdict badges (ADOPT=green, ADAPT=yellow, REJECT=red, IN_PROGRESS=gray), need one-liner, date, per-row affordance
  - ADAPT rows: inline "promote to ponder →" link
  - REJECT rows: inline "filed to knowledge" badge (if stored) or "file to knowledge →" link
  - ADOPT rows: "findings →" link only (no promote)
  - Detail view: breadcrumb, verdict badge, findings rendered as structured sections, "Promote to Ponder" button (ADAPT only)
  - Promote view: what-gets-carried checklist, editable ponder slug input, primary action button, CLI equivalent hint, success → navigate to ponder
- Add sidebar link in Explore group (alongside Ponder, Investigations): Zap icon, label "Spikes", route `/spikes`
- Add `/spikes` and `/spikes/:slug` routes in `frontend/src/App.tsx` (or router config)
- Add to BottomTabBar for mobile: Spikes in Explore tab roots
- On promote success: navigate to `/ponder/<ponder-slug>`
- No SSE needed for list refresh (spikes are agent-created, not server-pushed)
