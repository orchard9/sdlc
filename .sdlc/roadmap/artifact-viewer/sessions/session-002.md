---
session: 2
timestamp: 2026-03-01T00:00:00Z
orientation:
  current: "Three thought partners recruited and interrogated. Clear V1 direction reached: remove the height cap, add a sticky TOC, add file-path link detection with vscode:// URIs, and unify ArtifactViewer with WorkspacePanel. TLDR mechanism deferred to agent-instruction fix. Commenting deferred to V2."
  next: "Write the feature spec and build V1. The spec should cover: (1) ArtifactViewer refactor to remove height cap and add fullscreen-native layout, (2) TOC extraction from Markdown headings, (3) file path auto-detection, (4) agent instruction update for ## Summary convention."
  commit: "Feature direction established. V1 scope is bounded and implementable in one sprint. V2 commenting model design is documented but deferred."
---

## Session 2: Interrogating the Brief with Thought Partners

**Participants:** Nadia Osei (Rich Document Experience), Ben Hartley (Developer Productivity UX), Tobias Krenn (Skeptical Engineering Lead)

---

### Starting point: What does the current viewer actually do?

Before interrogating options, I traced exactly what exists today:

`ArtifactViewer.tsx` (used in `FeatureDetail.tsx`) renders each artifact (spec, design, tasks, qa_plan, etc.) as a card with:
- A header: artifact type label + status badge + fullscreen button
- Body: `<div className="p-4 max-h-96 overflow-y-auto">` containing `<MarkdownContent>`

The `max-h-96` constraint is `384px`. A plan artifact is routinely 300-600 lines of Markdown. At `~20px` per line, that is roughly 15-30 lines visible without scrolling — for a document that may be 300 lines. This is the literal source of Xist's complaint: "I can't really see that."

`WorkspacePanel.tsx` (used in `PonderPage.tsx`, `InvestigationPage.tsx`, `GuidelinePage.tsx`, `EvolvePage.tsx`) is a more capable viewer:
- Artifact list with file sizes and relative timestamps
- Expand-in-place content panel at the bottom (with `max-h-64` — also capped, but less painful because ponder artifacts are shorter)
- Fullscreen modal with in-modal pagination across artifacts
- `ArtifactContent.tsx` as the renderer (dispatches to `MarkdownContent` for `.md` files)

`FullscreenModal.tsx` takes over the full viewport (`fixed inset-0`) with `max-w-4xl mx-auto` content width. It works. The problem is that:
1. The in-panel view (before going fullscreen) is too small to be useful
2. The fullscreen modal has no TOC, no file link resolution, no way to navigate between artifact types

`MarkdownContent.tsx` uses `react-markdown` + `remark-gfm` + `mermaid` + syntax highlighting. It is already rendering correctly — tables, code blocks, headings, mermaid diagrams. The problem is not rendering quality; it is container constraints and missing navigation affordances.

**The diagnosis before talking to thought partners:** The gap is not "we need to build a new viewer." The gap is: (1) container height constraint, (2) no document navigation (TOC), (3) no file link resolution, (4) TLDR mechanism undefined, (5) commenting model undefined.

---

### Question 1: What is the MINIMUM change that eliminates Xist's workaround?

**Nadia Osei · Rich Document Experience Builder**

The workaround is: save plan as `.md` file → open in Agy markdown previewer → Agy has clickable file/class links. There are two things Agy provides that SDLC does not:

1. Rendered Markdown in a sufficiently large viewport
2. Clickable file paths that open in the IDE

The minimum change that eliminates the workaround is both of these together, because the workaround is motivated by both. If you only fix the viewport (render Markdown in a larger area), Xist still loses the file links and still has partial motivation to use Agy.

What does "fix the viewport" mean concretely? Remove `max-h-96` from `ArtifactViewer.tsx`. The content should breathe — no fixed maximum height for in-panel display. The page itself scrolls; the artifact card should expand to its natural height.

What does "file links" mean concretely? Scan Markdown content for patterns like `` `src/cmd/next.rs` `` or `crates/sdlc-core/src/feature.rs` — any path-like string inside backtick spans or parenthesized link targets that looks like a file path — and replace it with a `vscode://file/{workspaceRoot}/{path}` link. This is a client-side transform in `MarkdownContent.tsx`. No server changes required.

These two changes together — maybe 60 lines of code — eliminate the workaround.

**Tobias Krenn · Skeptical Engineering Lead**

I agree on the diagnosis. But let me be precise about what "minimum" means: it means the smallest thing that makes Xist stop using the workaround for his primary use case.

His primary use case is: "I just ran an agent that updated the plan. I want to see the updated plan." That is a read-only, single-artifact, full-content-visible use case. The file links are secondary — they matter during planning when he is navigating to source — but they are not what initially forces him out of SDLC. The scroll constraint is what forces him out.

So the absolute minimum is: remove `max-h-96`. Ship it. Then add file links in the next PR. Sequence matters.

Do not conflate "minimum to address the workaround" with "complete solution." The minimum is one CSS change. Do it this week.

**Ben Hartley · Developer Productivity UX Designer**

Both are right on the minimum. I want to add one thing: the fullscreen modal as it exists is actually fine for the "I want to read this properly" case. It is `fixed inset-0` with a `max-w-4xl` content area. The problem is the entry point — users should not need to click a tiny expand button to get to a readable view of a plan artifact.

The UX fix is: make the artifact card default to expanded (no height cap), with the fullscreen button as an escape hatch for distraction-free reading. Currently it's the reverse: the card is cramped and fullscreen is the primary way to read. That's backwards.

⚑  Decided: **The minimum viable change is: (1) remove `max-h-96` from `ArtifactViewer.tsx` in-panel view, (2) add file path auto-detection to `MarkdownContent.tsx` converting backtick-wrapped file paths to `vscode://` links. These are two separate PRs, in that order.**

---

### Question 2: The TLDR mechanism — agent problem or UI problem?

**Tobias Krenn · Skeptical Engineering Lead**

This is entirely an agent instruction problem, not a UI problem. Xist's exact quote is: "Agents think for 15 mins, TONS of output. I want at the end of that output: Summary of new plan as revised."

He is not asking for a UI component that generates a summary. He is asking that after a long agent run, the plan artifact itself is updated and he can find the current state quickly. The current plan artifact IS the summary — the problem is that he doesn't know how to get to it without reading the entire run log.

The fix is in two places:
1. **Agent instruction** (`init.rs`): `sdlc-run` should instruct agents to write `## Summary` as the first section of every spec and design artifact. This is a 3-line addition to `SDLC_RUN_COMMAND` content. The summary section should answer: "What is the current state of this plan?"
2. **FeatureDetail UI**: When rendering a plan artifact that has a `## Summary` section, pin the summary to the top of the artifact card (even before fullscreen). This means extracting the first heading's content and displaying it as a teaser.

There is no need for a second LLM call or a client-side summarizer. The agent wrote the plan; the agent can write the summary. We just need to ask it to.

**Nadia Osei · Rich Document Experience Builder**

I agree on the agent instruction fix. But I want to push on the UI layer too.

The real problem is not just "no summary at the top." It is: the artifact card gives me no orientation before I open it. I see "spec" with a status badge. I click expand. I see 400 lines of Markdown. I have to scroll for 30 seconds to understand what changed.

What I want is: the artifact card shows me the modification time (when did the agent last write to this?), the first heading level 1, and the first paragraph or `## Summary` content — as a preview/teaser. That is enough to tell me "this is the updated spec, written 12 minutes ago, the headline is XYZ." Then I can decide whether to read the full content.

This is a display pattern, not a generative pattern. It requires: (a) structured access to Markdown content (parse headings and first paragraph), (b) a teaser line in the card header. Both are client-side React changes.

**Ben Hartley · Developer Productivity UX Designer**

The teaser approach is right but needs a precision: it should show the content immediately below the first `# Heading` — which is either a `## Summary` section (if agents write one) or the first paragraph of body text (if they don't). This degrades gracefully.

For the artifact card in `FeatureDetail.tsx`: show `{artifact_type}` + status badge + last-modified time + teaser text (first 120 chars of body after the leading heading). This replaces the empty "no visible content until you scroll" experience with an immediately meaningful card.

?  Open: Should the teaser be parsed client-side (regex over the raw Markdown content) or should it be a field in the artifact API response? Client-side parsing is simpler; API field is more reliable but requires Rust changes. Given that the content is already returned in `artifact.content`, client-side parsing is correct for V1.

⚑  Decided: **The TLDR mechanism is two independent changes: (1) Add `## Summary` convention to `sdlc-run` agent instruction in `init.rs`. (2) Add a teaser line to `ArtifactViewer.tsx` card header showing last-modified time and first 120 chars of content below the leading heading. No LLM call, no separate summary endpoint.**

---

### Question 3: The commenting model — what is it actually, and what's the MVP?

**Nadia Osei · Rich Document Experience Builder**

Let me be blunt: inline commenting on plan Markdown is a 3-month project done right. The difficulty is not the UI — it is the data model. To anchor a comment to a specific span of text, you need:

1. A stable identifier for each text span (either a character offset or a hash of the surrounding text — both drift when the document changes)
2. A comment data model with: `artifact_id`, `span_anchor`, `author` (user or agent), `body`, `status` (open, resolved)
3. Backend: a new table or YAML structure for comments, indexed by artifact
4. Frontend: text selection API → create comment → render comment markers in the margin

This is Google Docs, simplified. It is not simple. I have tried to build this twice and abandoned it once because the span drift problem is unsolvable without a CRDT underneath the document.

**The question to answer first**: what is Xist actually trying to do? He says "add 3-4 comments on different parts of the plan, then submit all at once." If I translate this charitably, he wants: I can read the plan, mark sections I disagree with or want to change, and then send all that feedback to the agent as a single message. He is not asking for persistent comments visible to collaborators. He is asking for an annotation-to-message conversion tool.

That is a radically simpler problem.

**Ben Hartley · Developer Productivity UX Designer**

Nadia is right that the framing is wrong. Xist's desire is "submit all feedback as one LLM request" — not "persistent inline comments." He is using Google Docs as a metaphor for the annotation experience, not as a spec.

The MVP annotation model is:

1. The artifact viewer has a sidebar (or a bottom panel) with a text area labeled "Feedback on this artifact"
2. Users can highlight text in the plan and click "Add note" → the highlighted text + their note gets appended to the feedback textarea (like quoting in an email reply)
3. When they are done adding notes, they click "Send feedback" → this creates a task on the feature: "User feedback on [artifact type]: [body]" with the notes as the body, or — better — it opens a chat that pre-fills the agent prompt with the feedback
4. The agent receives all notes in one message and processes them

This is implementable in one sprint. It does not require persistent comment anchors. It does not require a new data model in Rust. It only requires: text selection API + textarea state accumulation + a "create feedback task" or "send to agent" action.

The difference from the full Google Docs model:
- Annotations are ephemeral (not persisted, not visible next time)
- No agent responding to individual inline comments
- No comment threads
- No collaboration (one user, one agent)

That is fine for V1.

**Tobias Krenn · Skeptical Engineering Lead**

I want to push on whether we should build this at all in V1.

We have one user asking for it. That user specifically praised Agy's planning capability — which does NOT have inline commenting. The annotation feature is additive: Xist is asking for something Agy doesn't have. That means (a) there is no competitive baseline to meet, and (b) the demand signal is weaker than for the viewer improvements (which ARE about feature parity).

My position: commenting is V2, and only after we validate that the viewer improvements alone are not enough. If Xist uses the SDLC viewer for two weeks after we fix the height cap and file links, and then comes back and says "I still need to annotate plans," we build it. If he doesn't come back, we saved a sprint.

?  Open: Should we still design the annotation data model (even if deferred) to avoid having to retrofit later? Yes — the data model is cheap to design and expensive to retrofit. But design, don't implement.

⚑  Decided: **Commenting/annotation is V2. The V1 viewer does not include annotation. The data model for annotations will be designed in session 3 (or as a separate ponder branch) without being implemented. If post-V1 user feedback revalidates the need, build it in the next cycle.**

---

### Question 4: Ponder scrapbook vs. feature artifact viewer — same or different?

**Nadia Osei · Rich Document Experience Builder**

They are the same component with different inputs and context. The rendering requirement is identical: Markdown content, file links, TOC, fullscreen. The difference is the surrounding context:

- Ponder scrapbook: unordered collection of freeform artifacts accumulated during ideation. No fixed set, no status badges, no "approve" or "reject" actions.
- Feature artifact viewer: ordered, typed artifacts (spec, design, tasks, etc.) with lifecycle state (draft, approved, rejected). Each artifact has a specific semantic role.

The component should be unified at the rendering layer (`MarkdownContent`, file link detection, TOC) but the wrapper / chrome is different:
- `WorkspacePanel.tsx` is the right wrapper for ponder and investigation (unordered artifacts)
- `ArtifactViewer.tsx` is the right wrapper for feature artifacts (typed, stateful artifacts)

Both should use `MarkdownContent.tsx` with the file-link enhancement. Both should use `FullscreenModal.tsx` for fullscreen. The TOC feature should be added to `MarkdownContent.tsx` so both surfaces get it.

**Tobias Krenn · Skeptical Engineering Lead**

The key observation here is that `WorkspacePanel.tsx` is already the more capable viewer. It has pagination, in-modal navigation, and artifact list with timestamps. `ArtifactViewer.tsx` is the less capable viewer.

The refactor direction is: improve `MarkdownContent.tsx` (the rendering core), and let both surfaces benefit. Do NOT try to make `ArtifactViewer.tsx` use `WorkspacePanel.tsx` — they have different chrome requirements and that would be over-engineering. Two separate wrappers, one rendering core.

**Ben Hartley · Developer Productivity UX Designer**

Agreed. The unification point is `MarkdownContent.tsx`, not the container components. Adding the TOC and file link detection to `MarkdownContent.tsx` propagates to every surface that uses it: `ArtifactViewer`, `WorkspacePanel`, the fullscreen modal for both.

⚑  Decided: **Ponder scrapbook and feature artifact viewer are different components (`WorkspacePanel` vs `ArtifactViewer`) with shared rendering via `MarkdownContent.tsx`. All rendering improvements (TOC, file links) go into `MarkdownContent.tsx` and both surfaces benefit automatically.**

---

### Question 5: The TOC — is it the killer feature?

**Ben Hartley · Developer Productivity UX Designer**

Yes. A plan document at 300+ lines is unnavigable without a table of contents. Even with the height cap removed, a user landing on the plan artifact still has to scroll through the entire document to find the section they care about.

The TOC implementation: extract all `#`, `##`, `###` headings from the Markdown content, assign anchor IDs, render a sticky sidebar (on desktop, where there is horizontal space) or a collapsible drawer (on mobile/narrow). Clicking a TOC entry scrolls to the anchor.

This is a client-side React feature that requires:
1. Parsing headings from Markdown content (trivial — iterate over the AST via `react-markdown` or regex over raw content)
2. Assigning IDs to heading elements during rendering (extend the `h1`, `h2`, `h3` components in `MarkdownContent.tsx`)
3. Building the TOC list component
4. Scroll-to-anchor behavior

Estimated effort: 1-2 days. Value: transforms a 300-line Markdown document from "unnavigable" to "immediately useful."

The TOC should appear inside the `FullscreenModal` view, not in the in-panel card view (where space is constrained). In-panel: remove height cap, show teaser. Fullscreen: full document + sticky TOC.

**Nadia Osei · Rich Document Experience Builder**

The TOC position matters. In the fullscreen view, the TOC should be a floating left panel (`w-48`, sticky, `overflow-y-auto`) alongside the content. Not above the content, not below — left rail, always visible. This is the Notion/GitHub docs/Stripe docs pattern. It works because your eyes can jump to the TOC without scrolling.

On narrow screens (below `lg:` breakpoint), collapse the TOC to a "Jump to..." dropdown at the top of the content.

**Tobias Krenn · Skeptical Engineering Lead**

The TOC is high value but I want to sequence it correctly. Order of priority:

1. Remove height cap (highest impact, lowest effort)
2. File path auto-detection (eliminates the core workaround)
3. TOC in fullscreen (transforms fullscreen from "readable" to "navigable")
4. Card teaser/preview (reduces friction before opening fullscreen)
5. Agent instruction for `## Summary` convention

Items 1 and 2 are week 1. Items 3-5 are week 2. Do not skip ahead.

⚑  Decided: **TOC is a V1 feature, but in fullscreen only. In-panel view gets the height cap removed and a teaser. TOC is implemented in the fullscreen layer. The `MarkdownContent.tsx` heading components get stable IDs added.**

---

### Question 6: File link auto-detection — implementation path

**Nadia Osei · Rich Document Experience Builder**

The Agy feature Xist is using: plan documents include `[[src/path/to/file.ts]]` or ``[`ClassName`](path/to/file.ts)`` or just `` `src/cmd/next.rs` `` and Agy renders them as clickable links that open the file in the IDE.

For SDLC, the implementation path:

1. **Detection pattern**: In `MarkdownContent.tsx`, in the `code` component handler (which handles backtick spans), add a check: if the content matches a file path pattern (`/^[a-z_][a-z0-9_\-/]*\.[a-z]{1,5}$/i`) and it is an inline code span (not a block), render it as a `<a href="vscode://file/...">` link.

2. **Path resolution**: The `vscode://file/{absolute-path}` URI requires an absolute path. The frontend knows the project root (it can be passed from the server, or it can be embedded in the page metadata from the existing API response). The detected relative path gets joined with the project root.

3. **Link target**: `vscode://file/{root}/{path}` opens VS Code. `cursor://file/{root}/{path}` opens Cursor. Rather than hardcoding, add a config option in `.sdlc/config.yaml`: `ide: vscode | cursor | zed`. The frontend reads this from the settings API and generates the correct URI scheme.

**Ben Hartley · Developer Productivity UX Designer**

One important precision: the detection should be in the `code` inline component handler, not a regex over raw Markdown content. The `react-markdown` component model means we already know that something is an inline code span vs. a code block vs. heading text — we should use that structural information, not a regex that might match paths in unexpected places.

The component handler: if it's an inline code span AND it matches the file path pattern AND the resulting URI is resolvable, render as a link. Otherwise, render as normal inline code. The "resolvable" check can be skipped in V1 (just link everything that looks like a path) — broken links just fail silently in VS Code.

?  Open: Should we show a different visual style for file-path links vs. regular web links? The existing `a` component in `MarkdownContent.tsx` renders with `text-primary underline`. File-path links could use a different icon (a file icon from lucide) to signal "opens in IDE, not browser." Worth doing but deferred to V1.1.

**Tobias Krenn · Skeptical Engineering Lead**

The IDE config is the right call. Don't hardcode VS Code. But don't over-engineer the config either: the default is `vscode`. A user with Cursor adds one line to `.sdlc/config.yaml`. That is sufficient.

The implementation check: does the server already return `project_root` or a similar path somewhere in the API? If not, it needs to be added to the `/api/state` or similar endpoint. Without the project root, we can't build absolute file URIs.

?  Open: Is the project root already available in the frontend API responses? Check `crates/sdlc-server/src/routes/` for what state the frontend receives. If not, add it to the project state API — one field, `project_root: String`, in the server response.

⚑  Decided: **File path auto-detection is V1. Implementation: extend inline `code` component in `MarkdownContent.tsx` to detect file paths and render as `{ide}://file/{root}/{path}` links. Add `ide` config field to `.sdlc/config.yaml` (default: `vscode`). Add `project_root` to server API response if not present.**

---

### Synthesis: What is the V1 spec?

**Nadia Osei · Rich Document Experience Builder**

V1 is four things:

1. **Remove height cap in `ArtifactViewer.tsx`** — `max-h-96 overflow-y-auto` becomes `overflow-visible`. The artifact card expands to its natural height. The page scrolls.

2. **File path auto-detection in `MarkdownContent.tsx`** — inline code spans matching file path patterns become `{ide}://file/{root}/{path}` links. Requires `project_root` from API and `ide` setting from config.

3. **TOC in fullscreen (`FullscreenModal`)** — when `MarkdownContent` is rendered in fullscreen mode, add a left-rail sticky TOC extracted from heading elements. Headings get stable IDs (`slugify` of heading text) for scroll-to-anchor. On narrow screens, collapse to a "Jump to..." dropdown.

4. **Card teaser in `ArtifactViewer.tsx`** — artifact card header shows last-modified timestamp and first 120 chars of body content (below the first H1 heading). Replaces the "nothing until you scroll" experience with immediate orientation.

V1 is NOT: commenting, annotation, TLDR generation, diff between versions, real-time collaboration, a new unified viewer component.

**Ben Hartley · Developer Productivity UX Designer**

I want to add one more: the **agent instruction update**.

The `sdlc-run` command in `init.rs` should include an instruction to agents: "Each spec and design artifact you write must begin with a `## Summary` section (2-4 sentences) stating the current state of the plan." This costs nothing to implement (3 lines in `init.rs`) and makes the card teaser feature dramatically more useful — because the teaser will consistently show a human-readable summary rather than the first sentence of body prose.

This is technically a V1 agent-side change, not a UI change. But it unlocks the full value of the card teaser.

**Tobias Krenn · Skeptical Engineering Lead**

Correct. The final V1 scope:

1. Remove `max-h-96` from `ArtifactViewer.tsx`
2. File path auto-detection in `MarkdownContent.tsx` (+ project root in API + ide config)
3. TOC in `FullscreenModal` (for `MarkdownContent` content)
4. Card teaser in `ArtifactViewer.tsx`
5. `## Summary` convention in `sdlc-run` agent instruction

V2 (documented, not built):

- Annotation/commenting model (design the data model in the next ponder session)
- Diff between artifact versions
- Real-time sync of artifact content during agent run (so you see the plan update live)

The V1 scope is bounded. It can be built in one sprint. It eliminates Xist's workaround. Ship it.

---

### V2 design note (commented, not built): The annotation model

**Nadia Osei · Rich Document Experience Builder**

For when this comes back: the correct annotation model for V1 of commenting is NOT inline spans. It is a sidebar annotation list, structurally similar to GitHub's PR review process:

1. User selects text in the artifact → a "+comment" button appears
2. Clicking opens a sidebar annotation entry: `{quoted_text: "...", note: "..."}`
3. User adds multiple annotations to the sidebar (accumulating, not inline)
4. "Submit feedback" converts all annotations to a single agent prompt: "Here is feedback on the [spec] artifact:\n\n[for each annotation: quoted text + note]\n\nPlease revise the spec addressing this feedback."
5. The agent run starts; the annotations are discarded (ephemeral)

No persistent data model needed in V1 of commenting. The annotations are session-state in React (a `useState` array). They are converted to text and submitted. This is implementable without any Rust changes.

The only tricky part: stable text selection. Text selection in a rendered Markdown document works via `window.getSelection()`. Quoted text is the raw selected string. No anchor IDs needed because we are not persisting the annotation — we are only quoting the text in the prompt.

This should be designed but not implemented until there is validated demand post-V1.

---

### Open questions for session 3 (or implementation phase)

?  Open: Is `project_root` already in the server API response? Check `/api/state` or `/api/features`. If not, cheapest path is: add it to the project-level state struct in `sdlc-core`.

?  Open: Should the TOC extraction happen in `MarkdownContent.tsx` (parsed from AST during render) or in a separate headings-extraction utility? The AST approach is cleaner; a regex approach is simpler but less reliable for edge cases (headings inside code blocks).

?  Open: The `ide` config field — what is the right YAML key? Suggest `.sdlc/config.yaml` → `settings.ide_uri_scheme: vscode`. This way it's grouped with other UI-adjacent settings, not with gates or quality thresholds.

?  Open: The teaser extraction — should it be: (a) first 120 chars of raw Markdown after the first heading, (b) content of the first `## Summary` section only, or (c) first paragraph only? Recommend (a) as default with (b) as preferred when the `## Summary` convention is followed. Handle both cases client-side.

?  Open: Does removing `max-h-96` from `ArtifactViewer` break any existing layout assumptions in `FeatureDetail.tsx`? The page uses `max-w-4xl mx-auto p-6` which is fine. The artifact section is `<section className="mb-6">`. No layout breakage anticipated, but needs visual review.
