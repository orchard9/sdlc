# Review: dist-release-infra

## Code Change Reviewed

**File:** `.github/workflows/release.yml`
**Change:** Added `homebrew` job that runs after `release` completes

## Review

### Correctness

The `homebrew` job correctly:
- Depends on `release` (ensuring the GitHub Release exists before the tap is updated)
- Uses `mislav/bump-homebrew-formula-action@v3` — the standard, widely-adopted action for this purpose
- Targets `orchard9/homebrew-tap` at `main` branch
- Sets `create-pullrequest: false` — commits directly to `main` in the tap repo (appropriate for an automated tap, not a community tap)
- Passes `download-url` pointing to the aarch64-apple-darwin binary — this is used by the action to compute the SHA256 and version

**One concern:** The `download-url` parameter passes only the `aarch64-apple-darwin` URL. The `mislav/bump-homebrew-formula-action` uses this URL to detect the version tag and compute the SHA256 for that one asset. For a dual-arch formula (arm64 + x86_64), the action would need to be run twice or the formula manually maintained. For a first-pass Homebrew formula, starting with the arm64 binary and adding x86_64 in a follow-up is acceptable.

**Action:** Track the x86_64 formula enhancement as a separate task.

### Security

- `COMMITTER_TOKEN` is passed via `${{ secrets.HOMEBREW_TAP_TOKEN }}` — not hardcoded, not logged
- The secret scope is correctly limited to `Contents: Write` on `orchard9/homebrew-tap` only (as specified in T2/T3)
- No other job references this secret — blast radius is minimal

### Workflow Integrity

- The `homebrew` job fails silently if `HOMEBREW_TAP_TOKEN` is not yet set — it won't block the release upload. This is acceptable behavior: releases succeed, Homebrew tap update is best-effort until the secret is configured.
- `fail-fast: false` is not set on `homebrew` (it's a single-job, not a matrix), which is correct.

### What Remains for Jordan (Not Code)

| Task | Status | Action Required |
|---|---|---|
| T1: Create `orchard9/homebrew-tap` repo | Blocked | Jordan: github.com/organizations/orchard9/repositories/new |
| T2: Create `HOMEBREW_TAP_TOKEN` PAT | Blocked | Jordan: github.com/settings/tokens |
| T3: Add secret to `orchard9/sdlc` Actions | Blocked | Jordan: github.com/orchard9/sdlc/settings/secrets/actions |
| T5: End-to-end CI verification | Blocked | Jordan: push a test tag after T1–T3 |

These tasks are infrastructure prerequisites — no code can accomplish them. They are tracked and blocked in the feature task list.

## Findings

### F1 — x86_64 formula architecture missing (track as task)

The `download-url` in the `homebrew` job points only to `aarch64-apple-darwin`. The formula will work for Apple Silicon Macs but will install the arm64 binary on Intel Macs via Rosetta (acceptable but not ideal).

**Action:** Add a follow-up task to update the formula for dual-arch support after the first release confirms the pipeline works.

### F2 — Formula name vs. binary name (FYI)

The binary is named `ponder` but the Homebrew formula is named `sdlc`. The `bump-homebrew-formula-action` will install the `ponder` binary, and the install script in the tap formula must create the `sdlc` symlink. This should be verified during T5 end-to-end testing.

## Verdict

APPROVED with one task created for F1.

The code change is minimal, correct, and follows the standard pattern. No issues block release. Jordan's manual steps (T1–T3) are the remaining blockers for the Homebrew tap to go live.
