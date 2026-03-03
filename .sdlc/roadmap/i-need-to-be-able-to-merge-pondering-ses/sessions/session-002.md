---
session: 2
timestamp: 2026-03-03T07:15:00Z
orientation:
  current: "All design decisions resolved. Pre-conditions tightened (no committed source or target). List filtering design settled (hide merged-parked by default, --all flag, redirect banner in show). Implementation scope confirmed: 2 core functions + 3 CLI touch-points."
  next: "Run /sdlc-ponder-commit to crystallize into a milestone/feature. Spec should include: PonderEntry fields, merge() fn signature, CLI subcommand, list filtering, show redirect banner."
  commit: "Met — all open questions resolved, design is complete, implementation scope is bounded."
---

**jordan · Owner**
I need to be able to merge pondering sessions together

---

## Facilitator opens Session 2

We landed at `converging` in Session 1 with commit signal met. The design is solid:

- CLI: `sdlc ponder merge <source> --into <target>`
- Data model: `merged_into: Option<String>` + `merged_from: Vec<String>` on `PonderEntry`
- Behavior: non-destructive full merge (copy sessions, prefix artifacts, union team)
- Status: source → `parked (merged)`, target gets `merged_from` appended

Two open questions survived Session 1. Let's close them.

---

## Open Question 1: Should target validation reject merging into a `committed` ponder?

**Felix Wagner:** "Both committed source and committed target should be rejected. A committed ponder has been crystallized into milestones — it's a closed chapter. Merging into it after the fact would pollute a clean audit trail. And merging a committed source would divorce its sessions from the feature trail they produced."

**Dan Reeves:** "Agree. This is cheap to check — two status comparisons at the top of the function. The error messages should be specific enough that an agent doesn't have to guess why it failed."

⚑  **Decided:**
- Source = `committed` → reject: `Cannot merge '<source>' — it is committed. Use git to trace its history.`
- Target = `committed` → reject: `Cannot merge into '<target>' — it is committed. Target must be active (exploring or converging).`

**Updated pre-conditions (final):**
```
1. Source exists
2. Target exists
3. Source ≠ target
4. Source has no merged_into already set (not already absorbed)
5. Source is not committed
6. Target is not parked
7. Target is not committed
```

---

## Open Question 2: Should `sdlc ponder list` hide `parked (merged)` entries by default?

**Felix Wagner:** "The `--status` filter already exists in the `list()` function. The minimal change: exclude entries where `merged_into` is set by default. These are the absorbed entries — pure noise. Regular parked entries (no `merged_into`) keep showing. That distinction matters: a parked entry is a deliberate choice, a merged-parked entry is a cleanup artifact."

"In the table, when `--all` is passed and a merged entry appears, show `↗ parked→<target-slug>` in the STATUS column. Three lines of change to the `list()` function."

⚑  **Decided:**
- Default: hide entries with `merged_into` set
- `--all` flag: show everything including merged entries
- STATUS display for merged entries: `↗ parked→<target-slug>`
- Regular `parked` entries (no `merged_into`) continue showing in default mode

---

## Dana Cho raises the `show` redirect behavior

"When an agent does `sdlc ponder show feedback-20260302` on a merged entry, it needs a clear signal — not silent wrong data. The redirect banner matters."

"But don't suppress the full output. Agents may still need the artifact list. Show the banner first, then show everything else."

⚑  **Decided:** `sdlc ponder show <merged-source>` displays at the top:
```
⚠  This entry was merged into '<target>'. Run: sdlc ponder show <target>
```
Then continues with normal show output (status, sessions, team, artifacts).

---

## Implementation plan (validated against codebase)

Code inspection confirms `PonderEntry` struct uses the same `#[serde(default, skip_serializing_if = ...)]` pattern for optional fields. `merged_into` and `merged_from` follow exactly the same pattern as `committed_to` and `committed_at`.

| File | Change |
|---|---|
| `crates/sdlc-core/src/ponder.rs` | Add `merged_into: Option<String>`, `merged_from: Vec<String>` to `PonderEntry`; add `pub fn merge(root, source_slug, target_slug) -> Result<MergeReport>` |
| `crates/sdlc-cli/src/cmd/ponder.rs` | Add `Merge { source, into_slug }` subcommand; add `merge()` handler; update `show()` for redirect banner; update `list()` for `--all` flag and default filtering |

**Out of scope for v1 (unchanged from Session 1):**
- Server route `/api/ponder/merge`
- UI banner in PonderPage
- `sdlc ponder unmerge` (git is the undo button)
- `--link-only` flag (full merge is already non-destructive)

---

## Commit signal assessment

**Status: MET.** All open questions resolved, no new blockers. The design is complete and bounded.

Next: `/sdlc-ponder-commit i-need-to-be-able-to-merge-pondering-ses` to crystallize into a feature.
