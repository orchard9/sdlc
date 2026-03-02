# Knowledge Librarian: All Decisions Resolved

## Resolved: sdlc-init is auto (session 3)

⚑ Decided: `sdlc knowledge librarian init` runs automatically as part of `sdlc init`.
No opt-in prompt. Every project gets the knowledge system bootstrapped from day one.

**Implementation requirements for empty projects:**
- Must not crash when VISION.md doesn't exist yet
- Catalog may be minimal (2-3 categories) when no prior investigations exist
- Librarian agent file is still generated — with whatever project context is available
- Init should run AFTER sdlc-init generates VISION.md and ARCHITECTURE.md (so it has domain signals)
- Report: 'Knowledge base initialized — 0 entries (add knowledge with `sdlc knowledge add`)'

**Lars's validation:** Auto-init removes the adoption barrier entirely. The system exists from project birth.

**Yuki's requirement:** Empty-catalog case must still produce a sensible librarian agent file.
The 5-7 categories from VISION.md signals should cover it — if VISION.md is empty, use a
minimal default catalog and instruct the librarian to evolve it as entries accumulate.

---

## Resolved: Advisory integration uses both approaches (session 3)

⚑ Decided: Advisory integration uses BOTH tag match AND keyword overlap scoring (v1 semantic proxy).

**Two-pass approach:**
1. Tag match: entries where entry.tags ∩ advisory_tags ≠ ∅, ranked by recency
2. Keyword overlap: entry summaries scored against the advisory prompt's key terms (TF-IDF-lite)
Merge results, deduplicate, inject top-N as 'Project Knowledge' context block.

**No external embeddings in v1** — keyword overlap is sufficient. Semantic vector search
is a future phase when the corpus is large enough to justify it.

**Remi's note:** The combination means the advisory doesn't miss entries that were tagged
differently but discuss the same topic. Better recall at the cost of slightly more context tokens.

---

## Resolved: No cron — orchestrator handles scheduled maintenance (session 3)

⚑ Decided: Remove all cron documentation from the knowledge system.
Scheduled maintenance is the orchestrator-tick-cli's responsibility.

**What this means for the knowledge feature:**
- POST /api/knowledge/maintain already defined — orchestrator calls it on a tick
- No `sdlc config.yaml` cron field needed
- Hook registration in librarian-init drops the weekly cron line
- The knowledge feature only registers: post-workspace-complete harvest hooks

**Lars's validation:** One fewer system concern for the knowledge feature.
The orchestrator-tick pattern is the right layer for scheduling — it has visibility
into what's running and can back off on busy ticks.

**Integration contract:**
- Orchestrator tick → POST /api/knowledge/maintain
- post-investigate-complete → sdlc knowledge librarian harvest investigation <slug>
- post-ponder-commit → sdlc knowledge librarian harvest ponder <slug>

---

## Status: All open questions resolved. Ready to commit.

The spec in knowledge-spec.md needs three updates:
1. Librarian Init Flow step 7: remove weekly cron line
2. Advisory integration: update to tag match + keyword overlap scoring
3. Integration table: mark sdlc-init as auto (not 'optional prompt')

After spec updates: /sdlc-ponder-commit knowledge-librarian

---

# Knowledge Librarian: All Decisions Resolved

## Resolved: sdlc-init is auto (session 3)

⚑ Decided: `sdlc knowledge librarian init` runs automatically as part of `sdlc init`.
No opt-in prompt. Every project gets the knowledge system bootstrapped from day one.

**Implementation requirements for empty projects:**
- Must not crash when VISION.md doesn't exist yet
- Catalog may be minimal (2-3 categories) when no prior investigations exist
- Librarian agent file is still generated — with whatever project context is available
- Init should run AFTER sdlc-init generates VISION.md and ARCHITECTURE.md (so it has domain signals)
- Report: 'Knowledge base initialized — 0 entries (add knowledge with `sdlc knowledge add`)'

**Lars's validation:** Auto-init removes the adoption barrier entirely. The system exists from project birth.

**Yuki's requirement:** Empty-catalog case must still produce a sensible librarian agent file.
The 5-7 categories from VISION.md signals should cover it — if VISION.md is empty, use a
minimal default catalog and instruct the librarian to evolve it as entries accumulate.

---

## Resolved: Advisory integration uses both approaches (session 3)

⚑ Decided: Advisory integration uses BOTH tag match AND keyword overlap scoring (v1 semantic proxy).

**Two-pass approach:**
1. Tag match: entries where entry.tags ∩ advisory_tags ≠ ∅, ranked by recency
2. Keyword overlap: entry summaries scored against the advisory prompt's key terms (TF-IDF-lite)
Merge results, deduplicate, inject top-N as 'Project Knowledge' context block.

**No external embeddings in v1** — keyword overlap is sufficient. Semantic vector search
is a future phase when the corpus is large enough to justify it.

**Remi's note:** The combination means the advisory doesn't miss entries that were tagged
differently but discuss the same topic. Better recall at the cost of slightly more context tokens.

---

## Resolved: No cron — orchestrator handles scheduled maintenance (session 3)

⚑ Decided: Remove all cron documentation from the knowledge system.
Scheduled maintenance is the orchestrator-tick-cli's responsibility.

**What this means for the knowledge feature:**
- POST /api/knowledge/maintain already defined — orchestrator calls it on a tick
- No `sdlc config.yaml` cron field needed
- Hook registration in librarian-init drops the weekly cron line
- The knowledge feature only registers: post-workspace-complete harvest hooks

**Lars's validation:** One fewer system concern for the knowledge feature.
The orchestrator-tick pattern is the right layer for scheduling — it has visibility
into what's running and can back off on busy ticks.

**Integration contract:**
- Orchestrator tick → POST /api/knowledge/maintain
- post-investigate-complete → sdlc knowledge librarian harvest investigation <slug>
- post-ponder-commit → sdlc knowledge librarian harvest ponder <slug>

---

## Status: All open questions resolved. Ready to commit.

The spec in knowledge-spec.md needs three updates:
1. Librarian Init Flow step 7: remove weekly cron line
2. Advisory integration: update to tag match + keyword overlap scoring
3. Integration table: mark sdlc-init as auto (not 'optional prompt')

After spec updates: /sdlc-ponder-commit knowledge-librarian

---

# Knowledge Librarian: All Decisions Resolved

## Resolved: sdlc-init is auto (session 3)

⚑ Decided: `sdlc knowledge librarian init` runs automatically as part of `sdlc init`.
No opt-in prompt. Every project gets the knowledge system bootstrapped from day one.

**Implementation requirements for empty projects:**
- Must not crash when VISION.md doesn't exist yet
- Catalog may be minimal (2-3 categories) when no prior investigations exist
- Librarian agent file is still generated — with whatever project context is available
- Init should run AFTER sdlc-init generates VISION.md and ARCHITECTURE.md (so it has domain signals)
- Report: 'Knowledge base initialized — 0 entries (add knowledge with `sdlc knowledge add`)'

**Lars's validation:** Auto-init removes the adoption barrier entirely. The system exists from project birth.

**Yuki's requirement:** Empty-catalog case must still produce a sensible librarian agent file.
The 5-7 categories from VISION.md signals should cover it — if VISION.md is empty, use a
minimal default catalog and instruct the librarian to evolve it as entries accumulate.

---

## Resolved: Advisory integration uses both approaches (session 3)

⚑ Decided: Advisory integration uses BOTH tag match AND keyword overlap scoring (v1 semantic proxy).

**Two-pass approach:**
1. Tag match: entries where entry.tags ∩ advisory_tags ≠ ∅, ranked by recency
2. Keyword overlap: entry summaries scored against the advisory prompt's key terms (TF-IDF-lite)
Merge results, deduplicate, inject top-N as 'Project Knowledge' context block.

**No external embeddings in v1** — keyword overlap is sufficient. Semantic vector search
is a future phase when the corpus is large enough to justify it.

**Remi's note:** The combination means the advisory doesn't miss entries that were tagged
differently but discuss the same topic. Better recall at the cost of slightly more context tokens.

---

## Resolved: No cron — orchestrator handles scheduled maintenance (session 3)

⚑ Decided: Remove all cron documentation from the knowledge system.
Scheduled maintenance is the orchestrator-tick-cli's responsibility.

**What this means for the knowledge feature:**
- POST /api/knowledge/maintain already defined — orchestrator calls it on a tick
- No `sdlc config.yaml` cron field needed
- Hook registration in librarian-init drops the weekly cron line
- The knowledge feature only registers: post-workspace-complete harvest hooks

**Lars's validation:** One fewer system concern for the knowledge feature.
The orchestrator-tick pattern is the right layer for scheduling — it has visibility
into what's running and can back off on busy ticks.

**Integration contract:**
- Orchestrator tick → POST /api/knowledge/maintain
- post-investigate-complete → sdlc knowledge librarian harvest investigation <slug>
- post-ponder-commit → sdlc knowledge librarian harvest ponder <slug>

---

## Status: All open questions resolved. Ready to commit.

The spec in knowledge-spec.md needs three updates:
1. Librarian Init Flow step 7: remove weekly cron line
2. Advisory integration: update to tag match + keyword overlap scoring
3. Integration table: mark sdlc-init as auto (not 'optional prompt')

After spec updates: /sdlc-ponder-commit knowledge-librarian

---

# Knowledge Librarian: All Decisions Resolved

## Resolved: sdlc-init is auto (session 3)

⚑ Decided: `sdlc knowledge librarian init` runs automatically as part of `sdlc init`.
No opt-in prompt. Every project gets the knowledge system bootstrapped from day one.

**Implementation requirements for empty projects:**
- Must not crash when VISION.md doesn't exist yet
- Catalog may be minimal (2-3 categories) when no prior investigations exist
- Librarian agent file is still generated — with whatever project context is available
- Init should run AFTER sdlc-init generates VISION.md and ARCHITECTURE.md (so it has domain signals)
- Report: 'Knowledge base initialized — 0 entries (add knowledge with `sdlc knowledge add`)'

**Lars's validation:** Auto-init removes the adoption barrier entirely. The system exists from project birth.

**Yuki's requirement:** Empty-catalog case must still produce a sensible librarian agent file.
The 5-7 categories from VISION.md signals should cover it — if VISION.md is empty, use a
minimal default catalog and instruct the librarian to evolve it as entries accumulate.

---

## Resolved: Advisory integration uses both approaches (session 3)

⚑ Decided: Advisory integration uses BOTH tag match AND keyword overlap scoring (v1 semantic proxy).

**Two-pass approach:**
1. Tag match: entries where entry.tags ∩ advisory_tags ≠ ∅, ranked by recency
2. Keyword overlap: entry summaries scored against the advisory prompt's key terms (TF-IDF-lite)
Merge results, deduplicate, inject top-N as 'Project Knowledge' context block.

**No external embeddings in v1** — keyword overlap is sufficient. Semantic vector search
is a future phase when the corpus is large enough to justify it.

**Remi's note:** The combination means the advisory doesn't miss entries that were tagged
differently but discuss the same topic. Better recall at the cost of slightly more context tokens.

---

## Resolved: No cron — orchestrator handles scheduled maintenance (session 3)

⚑ Decided: Remove all cron documentation from the knowledge system.
Scheduled maintenance is the orchestrator-tick-cli's responsibility.

**What this means for the knowledge feature:**
- POST /api/knowledge/maintain already defined — orchestrator calls it on a tick
- No `sdlc config.yaml` cron field needed
- Hook registration in librarian-init drops the weekly cron line
- The knowledge feature only registers: post-workspace-complete harvest hooks

**Lars's validation:** One fewer system concern for the knowledge feature.
The orchestrator-tick pattern is the right layer for scheduling — it has visibility
into what's running and can back off on busy ticks.

**Integration contract:**
- Orchestrator tick → POST /api/knowledge/maintain
- post-investigate-complete → sdlc knowledge librarian harvest investigation <slug>
- post-ponder-commit → sdlc knowledge librarian harvest ponder <slug>

---

## Status: All open questions resolved. Ready to commit.

The spec in knowledge-spec.md needs three updates:
1. Librarian Init Flow step 7: remove weekly cron line
2. Advisory integration: update to tag match + keyword overlap scoring
3. Integration table: mark sdlc-init as auto (not 'optional prompt')

After spec updates: /sdlc-ponder-commit knowledge-librarian