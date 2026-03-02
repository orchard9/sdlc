# Design: Add §12 Audit & Review Findings to .sdlc/guidance.md

## Approach

This is a pure documentation change. There is no code, no architecture, no UI, and no data model involved. The implementation is a single append to `.sdlc/guidance.md`.

## Change

Append the following section to the end of `.sdlc/guidance.md` (after §11):

---

```markdown
## 12. Audit & Review Findings

When `approve_audit` or `approve_review` is the directive, enumerate every finding
and assign it exactly one of three dispositions before approving:

| Disposition | What to do |
|---|---|
| **Fix now** | Make a targeted code change and commit it before calling `sdlc artifact approve` |
| **Track** | `sdlc task add <slug> "<finding summary>"` — creates a follow-up task |
| **Accept** | Write an inline rationale explaining why the finding is acceptable as-is |

**Silence is not acceptance.** Every finding must have an explicit disposition. Skipping
a finding is a protocol violation — the next agent cycle will re-raise it.

**Targeted fixes vs. systemic commands:** Use targeted code changes for individual
findings. Reserve `fix-all` and `remediate` for codebase-wide pattern corrections —
they are blunt instruments and should not be used to address a single audit item.
```

---

## No Alternatives Considered

The content is already authoritative in CLAUDE.md. This change copies it — with minor formatting adjustments for the guidance.md table style — into the document agents read first.

## File to Modify

- `.sdlc/guidance.md` — append §12 at end of file
