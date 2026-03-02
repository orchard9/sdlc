---
session: 3
timestamp: 2026-03-01T02:00:00Z
orientation:
  current: "All three open questions resolved — spec is final, ready to commit to features"
  next: "/sdlc-ponder-commit knowledge-librarian"
  commit: "Done — spec updated with all decisions, no remaining open questions"
---

## Session 3: Three Decisions, Full Resolution

**jordan · Owner**
> 1. init is auto
> 2. both
> 3. don't do the cron here - we can figure out how to do it with the orchestrator

---

Three open questions from session 2 answered in one message. Let's work through the
implications with the team.

---

### Decision 1: init is auto

⚑ Decided: `sdlc knowledge librarian init` runs automatically as part of `sdlc init`.

**Yuki Tanaka:**
"Auto is the right call. The catalog-from-nothing problem is solvable — if VISION.md
and ARCHITECTURE.md exist when librarian-init runs (and sdlc-init generates them),
we have enough domain signal. The librarian agent file gets written with whatever catalog
we can derive. For a genuinely empty project with nothing to harvest: sparse 3-5 category
catalog, a note to evolve it as entries accumulate. The catalog's job in that state is
just to hold structure, not to be comprehensive."

**Lars Andersen:**
"My concern is init reliability. The last thing we need is `sdlc init` failing because
the knowledge bootstrap threw on an edge case. Non-negotiable: knowledge librarian init
must be non-fatal. If it encounters a project with zero investigations, zero completed
ponders, and a blank VISION.md — it succeeds with a minimal result. The report reads:
'Knowledge base initialized. 0 entries. Add knowledge with `sdlc knowledge add`.'
That's fine. The important thing is the hooks get registered and the agent file exists."

**Remi Okonkwo:**
"Speed matters too. Auto-init shouldn't make `sdlc init` feel slow. The project scan
and workspace harvest need to be fast on an empty project — if there's nothing to harvest,
it should be near-instant. The expensive AI work only happens when there *is* something
to process. Empty project = fast init."

**What this means for the implementation:**

The librarian-init must run AFTER sdlc-init generates VISION.md + ARCHITECTURE.md, so
it has domain signals to work from. The init sequence inside `sdlc init` becomes:
1. Interview + generate VISION.md, ARCHITECTURE.md, CLAUDE.md
2. Generate .sdlc/config.yaml, register quality gates
3. `sdlc knowledge librarian init` — scan the just-generated docs, create minimal catalog,
   write agent file, register harvest hooks

Empty-project path through librarian-init:
- Project scan: reads VISION.md (may be stub), ARCHITECTURE.md (may be stub)
- Workspace harvest: zero completed investigations → skip, log "nothing to harvest yet"
- Domain extraction: derive 3-5 categories from VISION.md keywords at minimum
- Catalog generation: write minimal catalog.yaml
- Librarian agent generation: write agent file with stub catalog + project name
- Hook registration: register post-investigate-complete + post-ponder-commit hooks
- First maintenance pass: nothing to cross-ref yet → skip
- Report: "Knowledge base initialized. 0 entries across 3 categories. No completed workspaces to harvest."

⚑ Decided: librarian-init is resilient on empty projects — graceful degradation at every step.

---

### Decision 2: both (advisory integration)

⚑ Decided: Advisory integration uses BOTH tag match AND keyword overlap scoring.

**Remi Okonkwo:**
"'Both' is the right answer because they catch different things. Tag match is fast and
precise when someone was disciplined about tagging. Keyword overlap catches entries where
the tags differ but the topic is the same — which happens constantly when different people
added entries at different times. The combination gives you recall without requiring perfect
taxonomy."

**Yuki Tanaka:**
"The v1 semantic proxy without embeddings: TF-IDF-lite over entry summaries. Extract
key terms from the advisory prompt, score summaries by term overlap weighted by IDF
(inverse document frequency across the corpus). Simple, no dependencies, fast. When
the corpus grows large enough to justify it, we can swap in vector embeddings — the
interface is the same."

**Lars Andersen:**
"I want to know what 'top-N' means concretely. If N is too small you miss things; too
large and you're burning context tokens with noise. Recommendation: N=5 by default,
configurable in .sdlc/config.yaml under `advisory.knowledge_context_entries`. Start
with 5, tune from telemetry."

**Two-pass implementation:**
1. **Tag pass**: entries where `entry.tags ∩ advisory_context_tags ≠ ∅`, ranked by recency
2. **Keyword pass**: extract key noun phrases from advisory prompt text, score all entry
   summaries by keyword overlap (TF-IDF-lite), take top results
3. **Merge + deduplicate**: combine both result sets, rank by combined score
4. **Inject top-N**: as "Project Knowledge" context block in advisory prompt header

No external embeddings in v1. Vector semantic search is explicitly a future phase.

⚑ Decided: Two-pass tag + keyword approach, N=5 default, configurable. No embeddings in v1.

---

### Decision 3: no cron — orchestrator handles it

⚑ Decided: Scheduled maintenance is the orchestrator-tick-cli's responsibility.
The knowledge feature only owns post-workspace-complete harvest hooks.

**Lars Andersen:**
"This is the cleanest call in the whole design. Cron is infrastructure — it belongs at
the orchestration layer, not buried in a knowledge feature. Every time we've put scheduling
logic in a specific feature, it becomes a maintenance tax: the feature team owns it, nobody
else understands it, it breaks silently. The orchestrator-tick is exactly the right home
for this."

**Remi Okonkwo:**
"`POST /api/knowledge/maintain` is already defined in the REST API. The orchestrator
calls it on a tick. From the knowledge feature's perspective, it's just an endpoint that
runs a maintenance pass. The knowledge codebase has zero scheduling logic. The orchestrator
has zero knowledge-specific logic. Clean separation."

**Yuki Tanaka:**
"The hooks + orchestrator-tick combination gives complete coverage without cron:
- Immediate: post-workspace-complete harvest fires within seconds of an investigation finishing
- Scheduled: orchestrator-tick calls maintain on whatever cadence the project configures
Both triggers hit the same endpoint. The knowledge system doesn't need to know which
triggered it."

**What changes in the spec:**

Hook registration in librarian-init step 7 drops the weekly cron line:
```
7. Hook registration:
   - post-investigate-complete → sdlc knowledge librarian harvest investigation <slug>
   - post-ponder-commit → sdlc knowledge librarian harvest ponder <slug>
   (Scheduled maintenance handled by orchestrator-tick)
```

No `sdlc config.yaml` cron field for the knowledge feature.
Orchestrator-tick configuration is out of scope for this ponder — that's the orchestrator's design space.

⚑ Decided: Knowledge feature removes all cron configuration. Orchestrator-tick owns scheduling.

---

### Spec updates applied

Three concrete updates made to `knowledge-spec.md`:
1. Librarian Init Flow step 7: removed weekly cron line, added orchestrator-tick note
2. Integration table: advisory row updated to "tag match + keyword overlap scoring";
   sdlc-init row updated to "automatically" (not optional prompt)
3. "Open Questions for Commit" section replaced with "Resolved Decisions (Session 3)"

---

### Status

All three open questions are resolved. The spec is complete with no remaining open questions.
The plan.md covers three milestones with features correctly mapped.
The synthesis.md and decisions-final.md scrapbook artifacts capture the full design rationale.

This ponder is ready to commit.

**Next:** `/sdlc-ponder-commit knowledge-librarian`
