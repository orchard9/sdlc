# Audit: dist-release-infra

## Scope

Security audit of the workflow change: `.github/workflows/release.yml` — added `homebrew` job using `mislav/bump-homebrew-formula-action@v3`.

## Surface Analysis

This feature adds a CI step that:
1. Reads a GitHub secret (`HOMEBREW_TAP_TOKEN`)
2. Uses a third-party GitHub Action (`mislav/bump-homebrew-formula-action@v3`)
3. Commits to an external repository (`orchard9/homebrew-tap`) with the secret token
4. Runs on every push matching `v[0-9]+.[0-9]+.[0-9]+*`

## Findings

### A1 — Third-party action not pinned to SHA (MEDIUM)

**Finding:** `uses: mislav/bump-homebrew-formula-action@v3` references a mutable tag (`v3`), not a specific commit SHA. If the action maintainer pushes a malicious commit to the `v3` tag, it would execute with `COMMITTER_TOKEN` in scope.

**Mitigated by:**
- `mislav/bump-homebrew-formula-action` is a well-maintained, widely used action (GitHub's own Homebrew tap tooling uses it)
- Dependabot, if enabled, monitors action version drift
- The `COMMITTER_TOKEN` scope is restricted to `orchard9/homebrew-tap` only — the blast radius is limited to the tap repo

**Action:** Pin to a commit SHA for hardened security. For now, the mutable tag is acceptable given the constrained token scope. Create a follow-up task.

**Track:** T-future — pin third-party actions to commit SHAs

### A2 — Secret token is appropriately scoped (PASS)

**Finding:** `HOMEBREW_TAP_TOKEN` is a fine-grained PAT with `Contents: Write` on `orchard9/homebrew-tap` only (as specified in T2/T3). It cannot be used to access `orchard9/sdlc` or any other repository.

**Verdict:** PASS — minimal-privilege design is correct.

### A3 — Token exposure risk via workflow logs (PASS)

**Finding:** `COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}` is passed as an environment variable. GitHub Actions automatically redacts secret values from workflow logs.

**Verdict:** PASS — no log exposure risk.

### A4 — Tag trigger scope (FYI)

**Finding:** The workflow triggers on `v[0-9]+.[0-9]+.[0-9]+*` which is a glob allowing any tag starting with `v`. Tags like `v1.0.0-alpha` or `v1.0.0-rc.1` would trigger the homebrew update.

**Consideration:** Pre-release tags updating the Homebrew tap may not be desirable — Homebrew users expect stable releases. The `mislav/bump-homebrew-formula-action` does not filter pre-release tags.

**Action:** Add an `if: "!contains(github.ref_name, '-')"` condition to the `homebrew` job to skip pre-release tags. This prevents alpha/rc tags from updating the tap.

**Track:** Fix this before first release tag.

### A5 — Write permission scope in workflow (PASS)

**Finding:** The workflow has `permissions: contents: write` set at the top level, which applies to all jobs. The `homebrew` job only needs to write to the external `homebrew-tap` repo via the PAT — it does not need the built-in `GITHUB_TOKEN` write permission.

**Verdict:** PASS — the `contents: write` is needed by the `release` job for `softprops/action-gh-release`. The `homebrew` job doesn't use `GITHUB_TOKEN` for writes, so no over-permission here.

## A4 Fix

Apply the pre-release tag guard:

**File:** `.github/workflows/release.yml`

Add `if` condition to the `homebrew` job:

```yaml
  homebrew:
    needs: release
    runs-on: ubuntu-latest
    if: "!contains(github.ref_name, '-')"
    steps:
```

## Verdict

The change is safe to ship with one targeted fix applied (A4). A1 is a low-urgency hardening task for the future. All other findings pass.

**Findings summary:**

| ID | Severity | Status | Action |
|---|---|---|---|
| A1 | Medium | Accept (track) | Create task to pin action SHA |
| A2 | — | Pass | No action |
| A3 | — | Pass | No action |
| A4 | Medium | Fix now | Add `if: "!contains(github.ref_name, '-')"` |
| A5 | — | Pass | No action |
