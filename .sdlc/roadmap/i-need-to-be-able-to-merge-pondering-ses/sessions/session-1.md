---
session: 1
timestamp: 2026-03-03T05:55:00Z
orientation:
  current: "Problem is real: 42 ponder entries, with duplicate/fragmented threads (feedback-20260302 / feedback-20260302-2, multiple feedback-* entries). Merge should be directional — source absorbed into target, source parked with `merged_into` pointer. Full merge: sessions renumbered, artifacts copied with `from-<source>-` prefix, team union-merged. Simpler option debated — link-only vs file operations. Landed on full merge as default but non-destructive (source files stay, just copy to target)."
  next: "Capture design sketch as scrapbook artifact. Spec out the CLI interface, data model changes (merged_into / merged_from fields), and file operation behavior. Ready to commit to a feature."
  commit: "Met — problem is concrete (42 entries, visible duplicates), design is clear enough to spec. Ship when: sdlc ponder merge CLI designed, data model fields identified, behavior spec written."
---

**jordan · Owner**
I need to be able to merge pondering sessions together

---

## Facilitator opens the session

The brief is deceptively simple. "Merge pondering sessions" could mean three different things depending on what's actually painful:

1. **Entry-level merge** — two ponder entries (slugs) that cover the same ground, need to be consolidated into one
2. **Session-level merge** — multiple session logs within one entry, want a synthesis artifact
3. **Cross-entry linking** — pondering happened in two places, want a way to say "these are related"

Before we design anything, let's look at the actual evidence in the workspace.

**Concrete examples of the problem:**
- `feedback-20260302` and `feedback-20260302-2` — identical titles, both parked, clearly should have been one entry
- `feedback-improvements` and `i-need-a-better-feedback-system-that-let` — related ideas explored separately
- `ponder-conversations-need-to-be-more-acc` and `ponder-design-improvements` — overlapping problem spaces
- 42 total entries in the roadmap — the space gets cluttered fast

This is primarily an **entry-level merge** problem. Jordan starts a pondering session, something interrupts, a second entry gets created (maybe by an agent, maybe directly), and now there are two partial contexts on the same topic.

---

## Felix Wagner weighs in

*"Before anything else — tell me the lifecycle. A ponder entry has three exits: committed (becomes a milestone/feature), parked (dead end), or merge (absorbed by a sibling). Merge is just a fourth terminal status with a forwarding pointer. That's the mental model."*

**The CLI I'd want:**
```bash
sdlc ponder merge feedback-20260302 --into feedback-20260302-2
```

Directional. Clear. The `--into` flag names the survivor. The source gets absorbed. Output:
```
Merged 'feedback-20260302' into 'feedback-20260302-2':
  Sessions moved:   0
  Artifacts moved:  1  (notes.md → from-feedback-20260302-notes.md)
  Team merged:      0 members
  Source status:    parked (merged_into: feedback-20260302-2)
```

**Data model:**
Add two optional fields to `PonderEntry`:
```yaml
merged_into: feedback-20260302-2    # on the source (parked)
merged_from:                        # on the target
  - feedback-20260302
```

These are additive. `merged_from` is a list because you might absorb multiple sources over time.

**Sessions:** Copy session files from source to target, renumbering from `(target.sessions + 1)` onward. Each moved session file gets a header comment:
```markdown
<!-- merged from: feedback-20260302, original session: 1 -->
```

**Artifacts:** Copy with `from-<source>-` prefix. If `notes.md` exists in both source and target, the source copy becomes `from-feedback-20260302-notes.md`. No overwrite.

**Team:** Union merge by `agent` field. Duplicates (same agent name) are silently dropped.

---

## Dan Reeves pushes back

*"Hold on. You have 42 ponder entries. Most of them are parked or committed. When was the last time you read a parked ponder entry?"*

*"The real question is: does keeping two entries as separate parked entries actually cause you to lose anything? They're in git. They're queryable. The UI can show 'related entries' without a merge operation at all."*

*"What I'd accept: the `merged_into` pointer with zero file operations. When you view a merged entry, the CLI says 'This entry was merged into X — run `sdlc ponder show X` for the full picture.' You've spent 5 minutes implementing it instead of 2 hours. That covers 90% of the use case."*

**Dan's minimum viable version:**
```bash
sdlc ponder merge feedback-20260302 --into feedback-20260302-2
# Only adds merged_into: feedback-20260302-2 to source manifest
# Sets source status → parked
# Adds merged_from: [feedback-20260302] to target manifest
# Zero file operations
```

?  **Open:** Is the link-only version enough, or do file operations (session/artifact portability) matter?

---

## Dana Cho frames it as a workflow problem

*"I want to understand the failure mode. Is this 'I accidentally created two entries and want to clean up,' or is it 'I have meaningful work in both entries and need to synthesize it'?"*

*"For the cleanup case — two entries with 0 sessions and trivial notes — link-only is fine. For the synthesis case — two entries with 3+ sessions each, real artifacts, active thinking — you need file operations, because session history is how you reconstruct the thinking."*

*"The two concrete cases in this workspace: `feedback-20260302` / `feedback-20260302-2` are cleanup cases (0 sessions, short notes). The agent-created duplicates are probably cleanup cases too. But if Jordan is going to use `/sdlc-ponder` heavily, the synthesis case will show up eventually."*

*"My recommendation: ship the link-only version first. If file operations are needed, you'll know because someone will ask 'why didn't the sessions move?' — that's a signal to add it. Don't over-engineer pre-emptively."*

---

## Facilitator synthesizes

**Two real use cases, two different weights:**

| Use Case | Frequency | What's Needed |
|---|---|---|
| Cleanup (0-1 sessions, trivial artifacts) | High | Link-only: `merged_into` pointer, status → parked |
| Synthesis (multi-session, real artifacts) | Lower | Full merge: sessions moved, artifacts copied |

?  **Open tension:** Ship link-only first and add file ops later, OR ship full merge as default from day one?

**Arguments for shipping full merge from day one:**
- The CLI interface is the same either way — implementation complexity is mostly in core, not the command
- If you only ship link-only, agents will be confused when they `sdlc ponder show <target>` and don't see the source's sessions
- `sessions: 0` on the merged target looks wrong if the source had sessions

**Arguments for link-only first:**
- Dan's point: most merges are cleanup cases, not synthesis cases
- File operations (renumbering sessions, prefixing artifacts) have more edge cases
- Validation: if link-only gets used frequently, you have evidence for the full version

⚑  **Decided:** Ship **full merge with non-destructive file operations** — sessions are *copied* (not moved) to the target, source files remain but source entry is parked. This gives completeness without the fear of data loss.

---

## Behavior spec

### `sdlc ponder merge <source> --into <target>`

**Pre-conditions checked:**
- Source and target both exist
- Source is not already parked with another `merged_into`
- Source ≠ target
- Target is not parked (merging into a parked entry is probably a mistake)

**File operations (all non-destructive — copies, not moves):**

1. **Sessions**: For each session file in source `sessions/session-N.md`:
   - Read content, prepend `<!-- merged from: <source>, original session: N -->\n\n`
   - Write to target as `sessions/session-M.md` where M = target.sessions + counter
   - Increment target's `sessions` counter

2. **Artifacts**: For each scrapbook artifact in source dir (skip `manifest.yaml`, `sessions/`):
   - If filename doesn't exist in target: copy as-is
   - If filename already exists in target: copy with `from-<source>-<filename>` prefix

3. **Team**: Load both `team.yaml` (if present), union-merge by `agent` field, write to target

**Manifest updates:**
- Source: `status: parked`, add `merged_into: <target>`, set `updated_at`
- Target: add/append `merged_from: [<source>]`, set `updated_at`

**CLI output (table):**
```
Merged 'feedback-20260302' → 'feedback-20260302-2'
Sessions copied:   0
Artifacts copied:  1 (notes.md)
Team merged:       0 members
Source is now:     parked (merged_into: feedback-20260302-2)
```

**`sdlc ponder show`** should display `merged_into`/`merged_from` when present.

---

## What about the UI?

**`sdlc ponder show <merged-source>`** should output a redirect notice:
```
This entry was merged into 'feedback-20260302-2'.
Run: sdlc ponder show feedback-20260302-2
```

**Web UI (PonderPage):** Merged entries (status=parked + `merged_into` set) should show a banner:
> Merged into: [feedback-20260302-2 — Feedback March 02]

This is a display concern, not blocking for the initial feature.

---

## Implementation path

**Scope:**
1. `crates/sdlc-core/src/ponder.rs` — add `merged_into: Option<String>` and `merged_from: Vec<String>` to `PonderEntry`
2. `crates/sdlc-core/src/ponder.rs` — add `fn merge(root, source_slug, target_slug) -> Result<MergeReport>`
3. `crates/sdlc-cli/src/cmd/ponder.rs` — add `Merge { source, into_slug }` subcommand + `merge()` fn
4. `sdlc ponder show` output — display `merged_into` / `merged_from` when present

**What's explicitly out of scope for v1:**
- Server routes (no `/api/ponder/merge` endpoint — CLI-only is fine)
- UI banner (nice-to-have, not blocking)
- `--link-only` flag (the full merge is already non-destructive, no need for a downgrade option)
- `sdlc ponder unmerge` (git is the undo button)

⚑  **Decided:** This is a CLI+core feature only. No server route needed. The commit condition is met: concrete design, clear scope, ~4 files to touch.

---

## Open questions remaining

?  **Open:** Should target validation reject merging into a `committed` ponder? Probably yes — once committed, the ponder is closed. But it's not a hard blocker.

?  **Open:** Should `sdlc ponder list` hide `parked (merged)` entries by default, or show them with a `↗ merged` indicator? Cosmetic, but affects UX of a cluttered roadmap.

---

## Commit signal assessment

**Commit signal:** "idea is concrete enough to act on. merge CLI designed, data model fields identified, behavior spec written."

**Status:** MET.

- Problem is real (42 entries, visible duplicates, agent-created clutter)
- Design is settled (directional merge, full non-destructive file ops, `merged_into/merged_from` fields)
- Scope is bounded (CLI + core, ~4 files, no server routes in v1)
- Implementation path is clear

This is ready to become a feature.
