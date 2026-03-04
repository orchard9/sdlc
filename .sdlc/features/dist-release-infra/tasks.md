# Tasks: dist-release-infra

## T1 — Create `orchard9/homebrew-tap` repository (Jordan)

Create a public, empty GitHub repository:
- Organization: `orchard9`
- Name: `homebrew-tap`
- Description: "Homebrew formulae for orchard9 tools"
- Visibility: Public
- No README, .gitignore, or license (release workflow bootstraps content)

URL after creation: `https://github.com/orchard9/homebrew-tap`

## T2 — Create `HOMEBREW_TAP_TOKEN` PAT (Jordan)

Create a fine-grained personal access token:
1. Go to `https://github.com/settings/tokens/new` (Fine-grained tokens)
2. Token name: `sdlc-homebrew-tap`
3. Repository access: Only selected → `orchard9/homebrew-tap`
4. Permissions: Contents → Read and Write
5. Copy the generated token value

## T3 — Add `HOMEBREW_TAP_TOKEN` as GitHub Actions secret (Jordan)

1. Go to `https://github.com/orchard9/sdlc/settings/secrets/actions`
2. Click "New repository secret"
3. Name: `HOMEBREW_TAP_TOKEN`
4. Value: the token from T2
5. Click "Add secret"

## T4 — Add Homebrew formula update job to release workflow

Edit `.github/workflows/release.yml` to add a `homebrew` job that runs after `release` completes:

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

This action:
- Fetches the new release tag and computes SHA256 checksums for the macOS `.tar.gz` assets
- Writes or updates `Formula/sdlc.rb` in `orchard9/homebrew-tap` by committing directly to `main`
- Requires `orchard9/homebrew-tap` to exist (T1) and `HOMEBREW_TAP_TOKEN` to be set (T3)

## T5 — Verify infrastructure end-to-end

After T1–T4 are complete:
1. Confirm `orchard9/homebrew-tap` repo exists and is public
2. Confirm `HOMEBREW_TAP_TOKEN` secret appears in `orchard9/sdlc` Actions secrets list (value hidden is expected)
3. Trigger a CI dry-run: push a test pre-release tag (e.g., `v0.0.1-infra-test`) and confirm the `homebrew` job passes
4. Delete the test tag after verification

## Notes

- T1–T3 are Jordan-only manual steps; no code review needed
- T4 is a code change to `.github/workflows/release.yml` — reviewed as part of the normal PR flow
- T5 is agent-verifiable via `gh run list` once the test tag triggers CI
- The `mislav/bump-homebrew-formula-action@v3` action is the standard community action used by most Rust/Go projects for Homebrew tap automation
