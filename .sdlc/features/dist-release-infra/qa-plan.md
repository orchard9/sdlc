# QA Plan: dist-release-infra

## Scope

Infrastructure validation: confirm that the four resources (one GitHub repo, one PAT secret, one release workflow job, and the resulting Homebrew formula) are correctly configured and produce a working `brew install` experience.

## Pre-conditions

All five tasks (T1–T5) must be complete before QA can run:
- T1: `orchard9/homebrew-tap` repo exists on GitHub
- T2+T3: `HOMEBREW_TAP_TOKEN` secret is set on `orchard9/sdlc`
- T4: `homebrew` job is in `.github/workflows/release.yml`
- T5: End-to-end CI run completed successfully with a test tag

## Test Cases

### QA-1: Homebrew tap repo exists and is public

**Steps:**
1. Open `https://github.com/orchard9/homebrew-tap` in a browser (or `gh repo view orchard9/homebrew-tap`)
2. Confirm: repository is public, owned by `orchard9`

**Pass:** Page loads, repo is public.
**Fail:** 404 or private repo.

### QA-2: GitHub Actions secret is configured

**Steps:**
1. Open `https://github.com/orchard9/sdlc/settings/secrets/actions`
2. Confirm: `HOMEBREW_TAP_TOKEN` appears in the repository secrets list

**Pass:** Secret is listed (value hidden is expected behavior).
**Fail:** Secret not listed.

### QA-3: Release workflow contains `homebrew` job

**Steps:**
1. Read `.github/workflows/release.yml`
2. Confirm: `homebrew` job exists, depends on `release`, uses `mislav/bump-homebrew-formula-action@v3`
3. Confirm: `COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}` is set

**Pass:** All three conditions satisfied.
**Fail:** Job missing, wrong dependency, or token misconfigured.

### QA-4: CI run passes with release tag

**Steps:**
1. Check CI run triggered by the most recent version tag: `gh run list --repo orchard9/sdlc --workflow release.yml`
2. Confirm: all jobs including `homebrew` completed with status `success`

**Pass:** `homebrew` job shows `success`.
**Fail:** `homebrew` job shows `failure` or is absent.

### QA-5: Homebrew formula exists in tap repo

**Steps:**
1. Check `https://github.com/orchard9/homebrew-tap/blob/main/Formula/sdlc.rb`
2. Confirm: file exists and contains correct `url`, `sha256`, and `version` for the latest release

**Pass:** File exists with non-zero SHA256 values.
**Fail:** File absent or has placeholder values.

### QA-6: Homebrew install works on macOS

**Steps (macOS only):**
```sh
brew tap orchard9/tap
brew install sdlc
sdlc --version
```

**Pass:** `sdlc --version` prints the version string without error.
**Fail:** Any step fails or wrong binary installed.

## Pass Criteria

All of QA-1 through QA-5 must pass. QA-6 is a stretch goal — it requires a macOS machine and can be deferred to the `dist-first-release` feature's verification if unavailable in this context.

## Automated vs. Manual

| Test | Automated | Notes |
|---|---|---|
| QA-1 | Yes — `gh repo view` | |
| QA-2 | Partial — `gh secret list` only shows names, not values | |
| QA-3 | Yes — file read + grep | |
| QA-4 | Yes — `gh run list` + CI status | |
| QA-5 | Yes — `gh api` or `curl` the raw formula URL | |
| QA-6 | Manual | Requires macOS test machine |
