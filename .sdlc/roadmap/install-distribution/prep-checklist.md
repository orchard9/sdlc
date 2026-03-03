# Release Prep Checklist

## What Jordan needs to do (4 steps)

### Step 1 — Create orchard9/sdlc-releases
Create a **public, empty** repository at github.com/organizations/orchard9/repositories/new
- Name: sdlc-releases
- Description: Binary releases for sdlc
- Visibility: Public
- No README, no .gitignore, no license (cargo-dist manages the repo content)

### Step 2 — Create orchard9/homebrew-tap
Create a **public, empty** repository at github.com/organizations/orchard9/repositories/new
- Name: homebrew-tap
- Description: Homebrew formulae for orchard9 tools
- Visibility: Public
- No README, no .gitignore

### Step 3 — Create GH_RELEASES_TOKEN secret
1. Go to github.com/settings/tokens (fine-grained PAT recommended)
2. Grant `Contents: Read and Write` on repository `orchard9/sdlc-releases`
3. Go to github.com/orchard9/sdlc/settings/secrets/actions
4. New secret: Name = GH_RELEASES_TOKEN, Value = the PAT

### Step 4 — Create HOMEBREW_TAP_TOKEN secret
1. Go to github.com/settings/tokens (fine-grained PAT recommended)
2. Grant `Contents: Read and Write` on repository `orchard9/homebrew-tap`
3. Go to github.com/orchard9/sdlc/settings/secrets/actions
4. New secret: Name = HOMEBREW_TAP_TOKEN, Value = the PAT

## What the code changes look like (no Jordan action needed)

- Cargo.toml: Change x86_64-unknown-linux-gnu to x86_64-unknown-linux-musl
- Cargo.toml: Change install-path = CARGO_HOME to install-path = [~/.local/bin]

## How to trigger first release

After all 4 prep steps:
git tag v0.1.0
git push origin v0.1.0

The GitHub Actions release workflow fires automatically on version tags.

## Verification

Install the released binary on a fresh machine using the install scripts.