# Guidelines Interface

## What it is

A pattern distillation workspace. The user identifies a recurring problem — "we keep writing Go APIs the wrong way," "auth logic leaks into handlers," "every new developer does X differently" — and the interface extracts the principles, writes a guideline document, and stores it in the project where agents can read it.

Unlike Root Cause (reactive, bug-driven) and Evolve (proactive, growth-driven), Guidelines is compositional. It doesn't investigate why something broke or how a system should grow — it codifies "here is how we do this here," permanently.

The output is always a document. There is no "fix task" option. Guidelines can optionally produce enforcement tasks (add a linting rule, add to PR checklist, add to CLAUDE.md) but the primary artifact is the document.

---

## Phase Model

```
Problem → Evidence → Principles → Draft → Publish
```

| Phase | What happens | Agent does |
|-------|-------------|-----------|
| **Problem** | Define the recurring problem precisely — what breaks, who it affects, how often | Writes `problem.md` — problem statement, anti-patterns seen, impact |
| **Evidence** | Find concrete examples in the codebase: violations, edge cases, existing good patterns | Searches codebase, collects file:line examples, writes `evidence.md` |
| **Principles** | Extract rules that prevent the problem — specific, actionable, testable | Writes `principles.md` — numbered rules with rationale for each |
| **Draft** | Compose the full guideline document | Writes `draft.md` — the actual guideline in publishable form |
| **Publish** | Review draft, finalize storage location, write to project | Copies draft to `.sdlc/guidelines/<slug>.md` or `docs/<slug>.md` |

---

## Perspectives (Evidence Lenses)

During Evidence phase, the agent searches through four evidence types. These appear as cards in the workspace panel:

| # | Name | What it finds |
|---|------|--------------|
| 1 | Anti-patterns | Code that violates the target pattern — the bad examples |
| 2 | Good examples | Code in this project that already does it right |
| 3 | External prior art | Related RFCs, standards, or established patterns from other projects |
| 4 | Adjacent guidelines | Existing guidelines in this project that interact with or depend on this one |

These aren't parallel investigations like root-cause — they're evidence categories. The agent fills them progressively as it searches. Each card shows a count of examples found and lists file:line references.

During Principles phase, the cards pivot to show principle extraction — each card shows a draft rule being formed. The agent iterates with the user: "Does this rule capture what you mean?"

---

## UI Structure

The guidelines interface has an important difference from root-cause and evolve: the right pane is a **live document preview**, not a workspace file browser.

```
┌──────────────────────────────────────────────────────────┐
│ ← Title                              [status]            │  ← No workspace toggle; right pane is always doc preview
├──────────────────────────────────────────────────────────┤
│ [Problem] → [Evidence] → [Principles] → [Draft] → [Publish] │
├─────────────────────────────────────┬────────────────────┤
│                                     │                    │
│  Session dialogue stream            │  Document preview  │  ← Right pane is live preview
│                                     │  (evidence cards   │     of emerging guideline
│                                     │   during Evidence; │
│                                     │   rule cards       │
│                                     │   during Principles│
│                                     │   live doc preview │
│                                     │   during Draft+)   │
│                                     │                    │
├─────────────────────────────────────┴────────────────────┤
│  Describe the problem / give feedback on draft...        │
└──────────────────────────────────────────────────────────┘
```

The right pane transitions:
- **Problem/Evidence**: Evidence cards with file:line references (clickable → fullscreen)
- **Principles**: Principle cards — each rule with rationale, editable by the user
- **Draft/Publish**: Live markdown preview of the guideline document (uses `MarkdownContent`)

This makes the interface feel like collaborative document editing — the agent writes, the user reviews in real time, gives feedback, and the agent revises.

---

## Data Model

```yaml
# .sdlc/investigations/<slug>/manifest.yaml
slug: go-hexagonal-api
title: "Go API — Hexagonal Architecture"
kind: guideline            # discriminator field (root_cause | evolve | guideline)
phase: draft               # problem | evidence | principles | draft | publish | done
status: in_progress
problem_statement: "Auth logic leaks into HTTP handlers; no consistent port/adapter boundary"
guideline_scope: "Go API services"  # where this guideline applies
publish_path: null         # set during publish phase
created_at: "..."
updated_at: "..."
evidence_counts:
  anti_patterns: 4         # count of examples found
  good_examples: 2
  prior_art: 3
  adjacent: 1
principles_count: 6        # how many rules extracted so far
```

All investigations share a flat directory — `kind` in the manifest discriminates the type. Note: the field is `guideline_scope` (not `scope`) to distinguish from evolve's `scope`.

Artifacts:
```
.sdlc/investigations/<slug>/
  manifest.yaml
  sessions/
    session-001.md
  problem.md
  evidence.md
  principles.md
  draft.md
  # no synthesis.md — the draft IS the synthesis
```

Published output:
```
.sdlc/guidelines/go-hexagonal-api.md    ← default publish location
# OR
docs/guidelines/go-hexagonal-api.md    ← if user has a docs/ convention
```

---

## Document Format (Published)

The guideline document is a first-class project artifact, readable by agents. Structure:

```markdown
---
title: Go API — Hexagonal Architecture
scope: Go API services
created: 2026-02-27
investigation: .sdlc/investigations/guidelines/go-hexagonal-api/
---

# Go API — Hexagonal Architecture

## Problem

[Why this guideline exists. What breaks without it. Who is affected.]

## Rules

1. **Ports are interfaces, adapters are implementations.** HTTP handlers, CLI commands, and gRPC handlers implement port interfaces. They never contain business logic.
2. **Domain types do not import infrastructure.** ...
3. ...

## Patterns

### Do
```go
// Port interface in domain layer
type UserRepository interface {
    FindByID(ctx context.Context, id UserID) (*User, error)
}
```

### Don't
```go
// Infrastructure dependency leaking into domain
func (s *UserService) FindByID(id string) (*User, error) {
    db := sql.Open("postgres", os.Getenv("DB_URL")) // ← wrong
```

## Evidence

Anti-patterns found during investigation:
- `crates/sdlc-server/src/routes/users.rs:45` — DB query inside HTTP handler
- `crates/sdlc-server/src/routes/auth.rs:23` — business rule in route handler

Good examples:
- `crates/sdlc-core/src/feature.rs:12` — domain struct with no external imports

## Enforcement

- [ ] Add clippy lint for direct DB imports in routes layer
- [ ] Add to PR checklist: "Do handlers contain business logic?"
- [ ] Add reference to CLAUDE.md §Architecture

## Related Guidelines

- [Error Handling](.sdlc/guidelines/go-error-handling.md)
```

---

## Guidelines Index

All published guidelines are indexed at `.sdlc/guidelines/index.yaml`:

```yaml
guidelines:
  - slug: go-hexagonal-api
    title: "Go API — Hexagonal Architecture"
    scope: "Go API services"
    path: .sdlc/guidelines/go-hexagonal-api.md
    created: 2026-02-27
  - slug: go-error-handling
    title: "Go Error Handling"
    ...
```

This index is injected into agent context (via `GUIDANCE_MD_CONTENT` in `init.rs`) so agents read guidelines before acting. The Guidelines page in the UI reads this index for browsing.

---

## Output: Enforcement Tasks

After publishing, the interface offers optional enforcement tasks:

1. **Lint rule** — create a task to add a static analysis rule enforcing the guideline
2. **PR checklist** — create a task to add checklist items to PR templates
3. **CLAUDE.md reference** — create a task to reference the guideline from CLAUDE.md so agents load it

These are standard tasks (not urgent). They're presented as a checklist in the OutputGate — the user picks which ones to create.

---

## Relationship to Other Interfaces

Guidelines can be created from any context:

```
Root Cause → (systemic cause) → OutputGate → "Create Guideline" → Guidelines (pre-seeded)
Evolve → (architecture pattern) → OutputGate → "Write Architecture Guideline" → Guidelines (pre-seeded)
Standalone → User starts a Guidelines session directly
```

When seeded from Root Cause or Evolve, the problem statement, evidence, and principles are pre-populated from those investigations. The guideline session skips to Draft phase with the prior work as context.

---

## Agent Prompt

```
You are distilling a coding guideline.
Current phase: {phase}
Problem: {problem_statement}
Scope: {scope}

Your output is a document that developers and AI agents will read before writing code.
Rules must be specific, actionable, and testable — not vague principles.
Evidence must cite file:line. Good examples are as important as anti-patterns.

[Loaded prior artifacts: {list}]
```

The agent iterates with the user on each principle before moving to Draft. It explicitly asks: "Is rule 3 specific enough that a developer could verify compliance without asking for clarification?"

---

## UI: Guidelines Browser

A separate view (not the investigation list) browses published guidelines:

```
/guidelines
  Lists all .sdlc/guidelines/*.md entries from the index
  Search by title, scope, tag
  Click → opens the published guideline in a fullscreen reader
  "New" button → starts a new guidelines investigation session
```

This is the consumption interface. The investigation interface is for creation. Published guidelines are read-only in the browser (to edit, open the investigation session).

---

## CLI Commands

```bash
sdlc investigate create <slug> --kind guideline --title "..." --context "..."
sdlc investigate update <slug> --scope "Go API services"  # sets guideline_scope
sdlc investigate capture <slug> --content "..." --as evidence.md
sdlc investigate session log <slug> --file /tmp/investigation-session-<slug>.md
```

Session protocol is identical to root-cause (see root-cause.md).

---

## Implementation Order

1. **Data layer** — `InvestigationEntry` with `kind: guideline`, `guideline_scope`, `problem_statement`, `evidence_counts`, `principles_count`, `publish_path` — **DONE** (`sdlc-core/src/investigation.rs`)
2. **`EvidenceCards` component** — four evidence category cards with file:line lists
3. **`PrincipleCards` component** — rule cards with rationale, editable
4. **Phase-aware right pane** — switches between evidence cards, principle cards, and live markdown preview
5. **Publish flow** — writes doc to `.sdlc/guidelines/`, updates index, shows enforcement task picker
6. **Guidelines browser page** — `/guidelines` route, reads index, fullscreen reader
7. **Seeding from Root Cause / Evolve** — OutputGate "Create Guideline" pre-populates problem + evidence
8. **CLAUDE.md injection** — update `GUIDANCE_MD_CONTENT` in `init.rs` to include guidelines index path

---

## What Makes This Different from Just Writing a Doc

A developer could open a text editor and write `docs/go-hexagonal.md`. The interface adds:

- **Evidence grounding** — rules are derived from actual code, not gut feel
- **Completeness pressure** — four evidence categories and numbered rules create a checklist
- **Discoverability** — published to a known location, indexed, injected into agent context
- **Iteration** — the agent asks "is this specific enough?" before locking in principles
- **Chain of custody** — the investigation history explains why the guideline says what it says
