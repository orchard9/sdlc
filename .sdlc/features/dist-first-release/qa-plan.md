# QA Plan: dist-first-release

## Scope

Validate that the `v0.1.0` release is correctly published to GitHub and that the install scripts produce working binaries on supported platforms.

## Test Cases

### TC-1: Tag exists in git

**Type:** Automated
**Command:** `git tag -l v0.1.0`
**Pass:** Output is `v0.1.0`
**Fail:** Empty output or tag not found

---

### TC-2: GitHub Release exists with 5 assets

**Type:** Manual inspection
**Steps:**
1. Navigate to `https://github.com/orchard9/sdlc/releases/tag/v0.1.0`
2. Confirm release is published (not draft)
3. Count attached assets

**Pass:** Release is public, exactly 5 assets present:
- `ponder-aarch64-apple-darwin.tar.gz`
- `ponder-x86_64-apple-darwin.tar.gz`
- `ponder-x86_64-unknown-linux-musl.tar.gz`
- `ponder-aarch64-unknown-linux-musl.tar.gz`
- `ponder-x86_64-pc-windows-msvc.zip`

**Fail:** Release is draft, missing assets, or asset names differ

---

### TC-3: All 5 CI build jobs passed green

**Type:** Manual inspection
**Steps:**
1. Open the `Release` workflow run in GitHub Actions
2. Confirm each matrix target shows green checkmark

**Pass:** All 5 jobs pass with ✓
**Fail:** Any job shows failure or was skipped

---

### TC-4: install.sh resolves latest version correctly

**Type:** Manual verification
**Steps:**
```bash
curl -sSfL https://api.github.com/repos/orchard9/sdlc/releases/latest | grep tag_name
```
**Pass:** Returns `"tag_name": "v0.1.0"`
**Fail:** Returns 404 or empty tag_name

---

### TC-5: install.sh installs ponder on macOS

**Type:** Functional test
**Steps:**
```bash
# Run in a clean shell where ~/.local/bin/ponder does not already exist
curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
~/.local/bin/ponder --version
~/.local/bin/sdlc --version
```
**Pass:** Both commands return `ponder 0.1.0` without error; `sdlc` is a symlink to `ponder`
**Fail:** Install errors, binary not executable, or version mismatch

---

### TC-6: Installed binary is the correct target architecture

**Type:** Automated
**Command (macOS Apple Silicon):**
```bash
file ~/.local/bin/ponder
```
**Pass:** Output contains `arm64` (or `x86_64` on Intel mac)
**Fail:** Wrong architecture (would result in "exec format error" anyway)

---

## Pass Criteria

All 6 test cases must pass. TC-5 and TC-6 require local execution on macOS. TC-3 and TC-4 are satisfied by CI passing and GitHub API response. If TC-5 fails due to PATH issues (install dir not in PATH), this is a warning, not a blocker — the binary still works when called with full path.

## Out of Scope

- Windows install.ps1 functional test (no Windows machine available; CI build is the proxy)
- Linux install test (CI musl build success is the proxy)
- Homebrew formula validation (future milestone)
