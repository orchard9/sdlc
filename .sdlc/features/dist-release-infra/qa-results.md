# QA Results: dist-release-infra

## Run Date: 2026-03-03

## Summary

PARTIAL PASS — code change (T4) verified; Jordan's manual steps (T1–T3) are pending.

## Test Results

### QA-1: Homebrew tap repo exists (FAIL — pending Jordan)

```
$ gh repo view orchard9/homebrew-tap
GraphQL: Could not resolve to a Repository with the name 'orchard9/homebrew-tap'.
```

**Result:** FAIL — `orchard9/homebrew-tap` does not exist yet.
**Owner:** Jordan (T1)

---

### QA-2: GitHub Actions secret is configured (FAIL — pending Jordan)

```
$ gh secret list --repo orchard9/sdlc
(no output — no secrets configured)
```

**Result:** FAIL — `HOMEBREW_TAP_TOKEN` not yet configured.
**Owner:** Jordan (T2 + T3)

---

### QA-3: Release workflow contains `homebrew` job (PASS)

```
$ grep "mislav/bump-homebrew-formula-action" .github/workflows/release.yml
        uses: mislav/bump-homebrew-formula-action@v3

$ grep "HOMEBREW_TAP_TOKEN" .github/workflows/release.yml
          COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}

$ grep 'if:.*contains.*ref_name' .github/workflows/release.yml
    if: "!contains(github.ref_name, '-')"
```

**Result:** PASS — `homebrew` job is correctly configured with pre-release guard.

---

### QA-4: CI run passes (PARTIAL)

Previous `v0.1.0` release run (ID: 22612942704) failed at the `host` step in the cargo-dist workflow (missing `GH_TOKEN` env var, and `publish-homebrew-formula` would have required `orchard9/homebrew-tap` to exist).

The current workflow (from commit `7a9d44e`) replaced the cargo-dist workflow with the simpler custom release workflow. The `homebrew` job added by this feature is new — it will be tested when Jordan completes T1–T3 and pushes the next release tag.

**Result:** PARTIAL — new workflow not yet exercised; previous workflow has known failure documented.

---

### QA-5: Homebrew formula exists in tap repo (FAIL — pending QA-1)

Prerequisite: `orchard9/homebrew-tap` must exist (QA-1 failed).

**Result:** BLOCKED on Jordan completing T1.

---

### QA-6: Homebrew install on macOS (DEFERRED)

Deferred to `dist-first-release` feature verification. Requires QA-1 through QA-5 to pass first.

---

## Workflow YAML Correctness Check (PASS)

Verified the full `release.yml`:
- `homebrew` job depends on `release`
- Pre-release guard: `if: "!contains(github.ref_name, '-')"`
- `download-url` points to correct aarch64 macOS binary URL format
- Secret passed correctly as `COMMITTER_TOKEN`
- No hardcoded credentials

---

## Outstanding Work Before Full QA Pass

| Item | Owner | Status |
|---|---|---|
| Create `orchard9/homebrew-tap` repo | Jordan (T1) | Pending |
| Create `HOMEBREW_TAP_TOKEN` PAT | Jordan (T2) | Pending |
| Set secret on `orchard9/sdlc` | Jordan (T3) | Pending |
| Push release tag to trigger end-to-end | Jordan (T5) | Pending T1–T3 |

## Verdict

The agent-implementable work is complete and verified. The remaining blockers are all Jordan's manual GitHub setup steps. The code change (T4) is correct and will function as intended once the prerequisites exist.

**QA-3 PASS. All other tests blocked pending Jordan's T1–T3 steps.**

Mark for `approve_merge` once Jordan confirms T1–T3 complete and a test release tag succeeds.
