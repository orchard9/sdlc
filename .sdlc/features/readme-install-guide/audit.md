# Audit: SSH and make install in README

## Scope

This feature makes three additive documentation edits to `README.md`. No code, tests, configuration, or executable artifacts were changed.

## Security Surface

**None.** Documentation changes have no execution surface. The two URLs introduced are:

1. `ssh://git@github.com/orchard9/sdlc` — this is the existing GitHub repo, SSH URL form. It is not new; it is an alternative protocol for a repo users are already choosing to trust.
2. `DEVELOPER.md` — an internal file link, no network request.

## Findings

| # | Finding | Severity | Action |
|---|---------|----------|--------|
| 1 | SSH URL matches existing HTTPS repo exactly — no supply-chain risk | n/a | Accept |
| 2 | `make install` runs project Makefile — users who clone the repo already implicitly trust project build scripts | n/a | Accept |
| 3 | No credential exposure, no secrets, no new dependencies | n/a | Accept |

## Verdict

No security findings. Documentation-only change. All explicit findings reviewed and accepted with rationale above.
