# Code Review: Playwright CI Gate on PRs

## Summary

Created `.github/workflows/uat.yml` — a GitHub Actions workflow that runs the Playwright e2e suite on every PR targeting `main`.

## Files Changed

- `.github/workflows/uat.yml` — new file (49 lines)

## Review

### Correctness

- **Binary target**: builds `--bin sdlc` (not `sdlc-server`). The `playwright.config.ts` `webServer.command` is `sdlc ui start`, which is a subcommand of the CLI binary — this is correct.
- **PATH setup**: `$GITHUB_WORKSPACE/target/release` is appended to `$GITHUB_PATH` before the Node.js steps so the `sdlc` binary is visible to the Playwright `webServer` subprocess.
- **`CI=true`**: set at the workflow `env` block. `playwright.config.ts` uses `!process.env.CI` for `reuseExistingServer`, so the server always starts fresh in CI. Retries and worker counts are also gated on this variable.
- **Artifact paths**: `frontend/playwright-report/` and `frontend/test-results/` match `playwright.config.ts` defaults for `outputDir` and reporter output.
- **`if: always()` on report upload**: ensures the HTML report is available even when tests fail, which is the primary debugging artifact.
- **`if: failure()` on traces**: traces/videos are large; only uploading on failure avoids wasting storage quota on successful runs.

### Caching

- Cargo cache covers `~/.cargo/registry`, `~/.cargo/git`, and `target/`. Key is `Cargo.lock` hash — any dependency change invalidates the cache, which is correct. Restore key falls back to any same-OS Cargo cache.
- Node.js cache via `setup-node` built-in covers `frontend/node_modules` keyed on `frontend/package-lock.json`.

### Action Versions

All actions pinned to stable major versions (`@v4`):
- `actions/checkout@v4`
- `dtolnay/rust-toolchain@stable`
- `actions/cache@v4`
- `actions/setup-node@v4`
- `actions/upload-artifact@v4`

### No Issues Found

The workflow is structurally correct, matches the spec and design, and correctly reflects the Playwright config's expectations. No security concerns (no secrets accessed, no external write permissions needed).

## Verdict: Approved
