---
session: 1
timestamp: 2026-03-03T03:00:00Z
orientation:
  current: "Decisions made: HTML mockups replace prose design artifacts in ponder; Tailwind CDN, dark theme, self-contained; .html extension captured via sdlc ponder capture. Scope: ponder skill first, feature create_design as follow-on."
  next: "Implement: add Design Artifact Protocol section to sdlc_ponder.rs, with format spec, filename convention, and capture instructions. Also update sdlc-next's create_design guidance."
  commit: "When the skill instruction update is clear enough that a single agent can implement it without ambiguity — REACHED."
---

**jordan · Owner**
ponder design improvements

any ponder skill call that asks for a "design" should result in an html mockup as an artifact

---

## Session 1 — What does "design" mean here, and what should HTML mockups look like?

### Framing the problem

The brief is terse but clear in intent: when an agent running the ponder skill is asked to produce a design artifact, it should output a *visual HTML mockup* rather than a Markdown prose description.

Right now, the ponder skill has no explicit convention for design artifacts. When a session reaches the feature shaping protocol's "design decisions" step, or when an agent naturally captures something called `design-sketch.md` or `layout.md`, they produce Markdown text that *describes* a UI — which requires the reader to mentally construct the layout. An HTML file can be opened in a browser and immediately understood.

Let me interrogate the brief before diving into solutions.

---

**Ben Hartley · Developer Productivity UX**

This is exactly the right instinct. When I'm reviewing a ponder scrapbook and I see `design-sketch.md`, I have to do a lot of mental work. It's like reading a recipe without seeing the dish. An HTML file — even a rough one — lets me *look* at the thing being designed. The cognitive load drops immediately.

The key question is what "a design" means in context. I'd define it narrowly: any artifact that is primarily about *how something looks or is arranged*. That's distinct from:
- A spec (what the thing does)
- A rationale doc (why we made these decisions)
- An architecture sketch (how it fits together technically)

When an agent is producing something about *layout, visual hierarchy, or interaction flow*, that's a design — and it should be HTML.

There's a practical concern: the ponder skill needs to be explicit about this. Right now the skill says "capture artifacts" with generic instructions. If you add a clear rule — "when producing a design artifact, write HTML not Markdown" — agents will follow it. But it needs to be a hard rule, not a suggestion.

---

**Dana Cho · Product Skeptic**

Before we commit to this, I want to understand what problem we're actually solving. Are we solving:

A) Agents produce Markdown prose when they should produce visual artifacts (→ format problem)
B) The ponder scrapbook doesn't surface design artifacts visually (→ UI problem)
C) Reviewers have trouble evaluating design decisions made in ponder sessions (→ communication problem)

My guess is all three, but they have different solutions. HTML mockups fix A and partially C. They do nothing for B unless the frontend can render them (right now the scrapbook shows Markdown files as text).

I'm not opposed — but let's make sure we're solving the right problem. Is the real friction that agents produce bad design artifacts, or that the artifacts they produce don't get reviewed?

My concern with HTML is scope creep. A "design" in a ponder session can range from a quick layout sketch (2 hours of work in HTML to make presentable) to a high-level architecture diagram (which should probably be Mermaid or ASCII, not HTML). If the rule is "all design = HTML", we'll get bloated artifacts for things that needed 3 bullet points.

---

**Tobias Krenn · Skeptical Engineering Lead**

I'll push on the implementation angle. What does it mean for an agent to produce "a self-contained HTML file"?

The risk I see: agents will produce mediocre HTML — inline styles everywhere, no structure, looks nothing like the actual sdlc UI. Then you have an HTML file that's *worse* than reading the Markdown description because it's misleading. A crude HTML mockup might communicate the wrong information about spacing, color, and interaction.

The solution to this is a **format spec** — a concrete template the skill instructs agents to use. If we say "use Tailwind CDN, dark theme, show these three states, label it as a prototype" — that's actionable. Without a spec, we'll get inconsistency.

I also want to flag: `.html` files in the ponder scrapbook aren't currently rendered anywhere. The web UI shows Markdown. So this is a file that exists but can't be viewed without downloading it and opening it in a browser. That's fine for now — better than no artifact at all — but it should be in scope to make the UI render HTML previews eventually.

---

### Interrogating the brief: "any ponder skill call that asks for a design"

Let me map the specific places in the ponder skill where "design" surfaces:

1. **Feature shaping protocol, step 4**: "Shape — core value prop, user stories, design decisions, trade-offs" — this is where the design is articulated
2. **Explicit artifact captures**: when an agent naturally reaches for `sdlc ponder capture --as design.md` or `design-sketch.md` or `layout.md`
3. **Cross-cutting**: the feature lifecycle's `create_design` action (separate from ponder but same workflow)

Jordan's brief is about "ponder skill calls" — so initially scoped to #1 and #2. The feature `create_design` action is a natural extension but a separate change.

? Open: Does the feature `create_design` action also need to produce HTML? (Natural extension, but separate implementation target)

---

### Pushing past "HTML mockup" to find the real shape

**Ben Hartley · Developer Productivity UX**

Let me make this concrete. What should a good HTML design artifact for sdlc look like?

The sdlc UI is dark theme, uses React + Vite, and has a specific visual language (cards, sidebar, panels). A good mockup should:

1. **Match the visual language** — dark background, muted text, monospace where appropriate
2. **Be self-contained** — Tailwind CDN, no external fonts needed
3. **Show multiple states** — empty state, populated state, error/loading where relevant
4. **Be labeled as a prototype** — a banner at the top preventing confusion with production UI
5. **Include a few interactive elements** — tab navigation between states, hover states — to demonstrate interaction patterns, not just static layout
6. **Skip animation and real data** — placeholder data is fine; fake loading spinners waste token budget

The HTML doesn't need to be polished. It needs to be *representative* of the key layout decisions. If an agent spends 80% of the budget on visual polish, that's wrong. 20% visual, 80% structural accuracy.

---

**Dana Cho · Product Skeptic**

What about designs that aren't visual interfaces? In ponder sessions we also design:
- Data model shapes (not visual)
- CLI command syntax (not visual)
- API endpoint contracts (not visual)

Should those produce HTML too? I'd say no. HTML is for UI/layout designs specifically. The skill should distinguish: *"if this design is about a user-facing interface, produce HTML; if it's about a data contract or CLI shape, produce Markdown with code blocks."*

The trigger for HTML should be: is this a layout, a screen, a panel, a modal, a widget? If yes → HTML. If it's a schema, a command signature, an algorithm → Markdown.

This distinction makes the rule more precise and prevents over-specification.

---

**Tobias Krenn · Skeptical Engineering Lead**

I agree with Dana's framing. Let me add one more: the filename convention matters.

Currently, agents capture whatever they want. If we establish a clear convention:
- `<name>-mockup.html` — for UI layout designs
- `<name>-design.md` — for non-visual design decisions (schema, API, CLI)

...then future agents and the web UI have a deterministic signal. The UI can detect `*-mockup.html` files and render them in an iframe. The skill can be clear: "if you're capturing a UI design, always use the `*-mockup.html` convention."

---

### Key decisions

⚑ Decided: HTML mockups are the correct format for UI layout/visual design artifacts in ponder sessions. Markdown design docs remain valid for non-visual design decisions (data schemas, CLI syntax, API contracts).

⚑ Decided: The trigger for HTML is whether the design describes a *user interface* — a screen, panel, modal, layout, component, or interaction flow. Non-UI designs stay Markdown.

⚑ Decided: Format spec for HTML mockups:
- Tailwind CDN (`<script src="https://cdn.tailwindcss.com"></script>`)
- Dark theme (`class="dark bg-gray-950 text-gray-100"` on body)
- Yellow prototype banner at top: `⚠ Design Prototype — not production code`
- Show 2-3 states (empty, populated; or state-A, state-B) using tab navigation or labeled sections
- Placeholder data — no real data, no complex animations
- Self-contained — no external dependencies except Tailwind CDN

⚑ Decided: Filename convention: `<descriptive-name>-mockup.html`. Examples: `dashboard-layout-mockup.html`, `thread-detail-mockup.html`. Captured via:
```bash
# Write to temp file first
# Then:
sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html
```

⚑ Decided: Scope is the ponder skill first. The feature `create_design` action is a natural extension to handle in a follow-on feature (e.g., feature `design-artifact-html` in the v22 or v23 milestone).

? Open: Should the ponder scrapbook UI in the frontend render `*-mockup.html` artifacts in an iframe? This would make them viewable in the web UI without downloading. Currently all scrapbook files are shown as Markdown text — HTML would need special handling.

? Open: Should the feature `create_design` action also produce `design-mockup.html` instead of (or alongside) `design.md`? The state machine currently expects `design.md` — changing this would require `sdlc-core` updates.

---

### What the skill instruction update needs to say

The ponder skill (`sdlc_ponder.rs`) needs a new section in "Capturing artifacts" — a **Design Artifact Protocol**:

```
### Design Artifact Protocol

When producing a design for a user interface (screen, panel, modal, widget, layout,
or interaction flow), produce a self-contained HTML mockup — not a Markdown description.

**Format:**
- `<!DOCTYPE html>` with Tailwind CDN
- `<body class="dark bg-gray-950 text-gray-100 p-8 font-mono">`
- Yellow prototype banner: `⚠ Design Prototype`
- Show 2–3 key states using tab buttons or labeled sections
- Placeholder data only
- No external dependencies beyond Tailwind CDN

**Filename:** `<descriptive-name>-mockup.html`

**Capture:**
```bash
# Write HTML to temp file, then capture
sdlc ponder capture <slug> --file /tmp/<name>-mockup.html --as <name>-mockup.html
```

**When NOT to use HTML:** data model designs, CLI syntax, API contracts, algorithm sketches
— these should remain Markdown with code blocks.
```

This change is narrow and additive — no existing behavior breaks. Agents producing non-UI artifacts continue using Markdown. Agents producing UI designs now have a clear, spec'd protocol.

---

### The commit signal is met

The idea is shaped, the format is specified, the scope is bounded. The change to `sdlc_ponder.rs` is clear enough that an agent can implement it without ambiguity.

The follow-on work (HTML rendering in the scrapbook UI, extending to `create_design` in the feature lifecycle) is noted as open questions but not blockers for this commit.

---

## Product Summary

### What we explored
We explored what it means for the ponder skill to produce a "design" artifact, and what format an HTML mockup should take. We mapped the specific points in the ponder skill where design artifacts are produced and established clear conventions for when HTML is appropriate versus when Markdown remains the right tool.

### Key shifts
⚑ The design artifact format is now clearly specified: self-contained HTML with Tailwind CDN, dark theme, prototype banner, 2-3 states shown. Previously there was no convention at all.
⚑ The trigger for HTML vs. Markdown is now precise: UI/layout designs → HTML; schema/API/CLI designs → Markdown.
⚑ Filename convention established: `<name>-mockup.html` — making future UI rendering in the scrapbook deterministic.

### Implications
Agents running ponder sessions will produce visual, browser-openable design artifacts instead of prose descriptions when designing interfaces. This makes design review faster (open a file, see the layout immediately) and makes ponder sessions more useful as a design tool, not just an ideation tool. The scrapbook becomes a design portfolio, not just a text archive. The follow-on question of whether the web UI should render these mockups inline is a clear next product decision.

### Still open
? Should the web UI render `*-mockup.html` scrapbook artifacts in an iframe (making them viewable without downloading)?
? Should the feature lifecycle's `create_design` action also produce HTML mockups instead of `design.md`?
