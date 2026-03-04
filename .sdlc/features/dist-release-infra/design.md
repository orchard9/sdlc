# Design: dist-release-infra

## Summary

Four infrastructure resources need to exist before the distribution milestone is complete. Three of the four (the GitHub repos and secrets) are manual Jordan actions. The fourth is a workflow change to publish a Homebrew formula on each release.

## Current State

The release pipeline already works end-to-end for install script distribution:

- `.github/workflows/release.yml` builds 5-platform binaries and uploads them as a GitHub Release on `orchard9/sdlc` using the built-in `GITHUB_TOKEN`
- `install.sh` and `install.ps1` download from `github.com/orchard9/sdlc/releases/latest` — this path is already valid
- The install scripts live in the repo root and are served via `raw.githubusercontent.com`

The one-liner install works today once a tag is pushed:
```sh
curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
```

## What Is Missing

### 1. Homebrew tap (new capability)

macOS users expect `brew install orchard9/tap/sdlc`. This requires:
- A public `orchard9/homebrew-tap` repo on GitHub
- A `Formula/sdlc.rb` formula file in that repo
- A step in the release workflow that updates the formula after each release

### 2. `HOMEBREW_TAP_TOKEN` secret

The release workflow needs write access to `orchard9/homebrew-tap`. GitHub Actions `GITHUB_TOKEN` scopes are limited to the current repo — a fine-grained PAT with `Contents: Write` on `homebrew-tap` is required.

### 3. `orchard9/sdlc-releases` repo (optional — scope decision)

The ponder prep checklist mentioned `sdlc-releases` as a separate releases repo. However, the current workflow already publishes to `orchard9/sdlc` releases satisfactorily. Creating a separate `sdlc-releases` repo adds complexity without user value. **Decision: skip `sdlc-releases` — release artifacts stay on `orchard9/sdlc`.**

## Implementation Plan

### Step 1 — Create `orchard9/homebrew-tap` (Jordan)

Create a public, empty GitHub repository at `github.com/organizations/orchard9`:
- Name: `homebrew-tap`
- Description: Homebrew formulae for orchard9 tools
- Visibility: Public
- No initial README, .gitignore, or license (the release workflow bootstraps content)

### Step 2 — Create `HOMEBREW_TAP_TOKEN` secret (Jordan)

1. Create a fine-grained PAT at `github.com/settings/tokens/new`:
   - Repository access: Only `orchard9/homebrew-tap`
   - Permission: Contents → Read and Write
2. Save the token as a repository secret on `orchard9/sdlc`:
   - Settings → Secrets and variables → Actions → New repository secret
   - Name: `HOMEBREW_TAP_TOKEN`
   - Value: the PAT

### Step 3 — Add Homebrew formula update step to release workflow

After the `Create GitHub Release` step, add a job that:
1. Generates the SHA256 checksums for the macOS binaries
2. Writes a `Formula/sdlc.rb` to `orchard9/homebrew-tap` using the `HOMEBREW_TAP_TOKEN`

The formula template:
```ruby
class Sdlc < Formula
  desc "Deterministic SDLC state machine for feature lifecycle management"
  homepage "https://github.com/orchard9/sdlc"
  version "VERSION"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/orchard9/sdlc/releases/download/VERSION/ponder-aarch64-apple-darwin.tar.gz"
      sha256 "AARCH64_SHA256"
    else
      url "https://github.com/orchard9/sdlc/releases/download/VERSION/ponder-x86_64-apple-darwin.tar.gz"
      sha256 "X86_64_SHA256"
    end
  end

  def install
    bin.install "ponder"
    bin.install_symlink "ponder" => "sdlc"
  end

  test do
    system "#{bin}/ponder", "--version"
  end
end
```

The workflow step uses `actions/github-script` or a simple `git` push to commit `Formula/sdlc.rb` to the tap repo.

### Workflow change (code — tracked as implementation task)

Add a `homebrew` job to `.github/workflows/release.yml`:

```yaml
homebrew:
  needs: release
  runs-on: ubuntu-latest
  steps:
    - name: Update Homebrew formula
      uses: mislav/bump-homebrew-formula-action@v3
      with:
        formula-name: sdlc
        homebrew-tap: orchard9/homebrew-tap
        base-branch: main
        create-pullrequest: false
      env:
        COMMITTER_TOKEN: ${{ secrets.HOMEBREW_TAP_TOKEN }}
```

## Verification

After all steps complete:

1. Push a test tag (or verify with existing tag) and confirm the `homebrew` job passes in CI
2. Run `brew tap orchard9/tap && brew install sdlc` on a macOS machine
3. Confirm `sdlc --version` works

## Files Changed

| File | Change |
|---|---|
| `.github/workflows/release.yml` | Add `homebrew` job after `release` |

## No Changes Needed

- `install.sh` / `install.ps1` — already correct, download from `orchard9/sdlc` releases
- `Cargo.toml` — no changes in this feature (musl/install-path changes are in `dist-cargo-toml-fixes`)
- `orchard9/sdlc-releases` — not created (releases stay on main repo)
