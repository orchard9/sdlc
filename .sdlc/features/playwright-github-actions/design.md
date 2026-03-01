# Design: Playwright CI Gate on PRs

## Workflow Structure

Single job named `playwright` in `.github/workflows/uat.yml`. No matrix, no parallelism across jobs — all steps run sequentially in one runner.

```
pull_request (targeting main)
    └── playwright (ubuntu-22.04)
            ├── Checkout
            ├── Install Rust (rust-toolchain@stable)
            ├── Cache Cargo (registry + git + target/)
            ├── Build sdlc binary (cargo build --bin sdlc --release)
            ├── Add ./target/release to PATH ($GITHUB_PATH)
            ├── Install Node.js 20 (setup-node@v4, npm cache)
            ├── npm ci (frontend/)
            ├── npx playwright install --with-deps chromium
            ├── npx playwright test (from frontend/)
            ├── Upload playwright-report/ (always, 30-day retention)
            └── Upload test-results/ (on failure, 7-day retention)
```

## Key Design Decisions

### Single job, not multi-job
The Playwright `webServer` config auto-starts `sdlc ui start` in-process, so the binary and the test runner must share the same filesystem and working directory. A matrix or separate jobs would complicate PATH propagation and working directory setup with no benefit for a single-browser suite.

### Build `sdlc` binary (not `sdlc-server`)
`playwright.config.ts` `webServer.command` calls `sdlc ui start --port 7777 --no-open`. The `sdlc` CLI binary embeds the server — `sdlc-server` is a library crate. The correct build target is `--bin sdlc`.

### PATH via `$GITHUB_PATH`
Appending `$GITHUB_WORKSPACE/target/release` to `$GITHUB_PATH` makes the `sdlc` binary visible to all subsequent steps, including the Playwright `webServer` subprocess that is launched by the test runner rather than by a shell step.

### `CI=true` at the workflow level
`playwright.config.ts` gates `reuseExistingServer` on `!process.env.CI`. Setting `CI=true` at the workflow `env` block ensures the web server is always started fresh and `retries: 2` / `workers: 4` are active.

### Cargo cache strategy
Cache key: `${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}`. Restore key: `${{ runner.os }}-cargo-`. Cached paths: `~/.cargo/registry`, `~/.cargo/git`, `target/`. The `target/` directory is included because incremental builds on a warm cache save the most time (sdlc has many crates).

### Node.js cache via `setup-node`
`actions/setup-node@v4` built-in cache with `cache: npm` and `cache-dependency-path: frontend/package-lock.json` is sufficient. No separate `actions/cache` step needed.

### Artifact retention
- `playwright-report/` — always uploaded (even on success) so HTML report is always available. 30-day retention matches GitHub's default.
- `test-results/` — only on failure (traces, screenshots, videos). 7-day retention because they are large and only needed for active debugging.

## Workflow YAML Sketch

```yaml
name: UAT

on:
  pull_request:
    branches: [main]

env:
  CI: "true"

jobs:
  playwright:
    name: Playwright E2E
    runs-on: ubuntu-22.04
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-toolchain@stable

      - name: Cache Cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          restore-keys: ${{ runner.os }}-cargo-

      - name: Build sdlc binary
        run: cargo build --bin sdlc --release

      - name: Add sdlc to PATH
        run: echo "$GITHUB_WORKSPACE/target/release" >> $GITHUB_PATH

      - name: Install Node.js
        uses: actions/setup-node@v4
        with:
          node-version: "20"
          cache: npm
          cache-dependency-path: frontend/package-lock.json

      - name: Install frontend dependencies
        run: cd frontend && npm ci

      - name: Install Playwright browsers
        run: npx playwright install --with-deps chromium
        working-directory: frontend

      - name: Run Playwright tests
        run: npx playwright test
        working-directory: frontend

      - name: Upload playwright-report
        uses: actions/upload-artifact@v4
        if: always()
        with:
          name: playwright-report
          path: frontend/playwright-report/
          retention-days: 30

      - name: Upload test-results (traces)
        uses: actions/upload-artifact@v4
        if: failure()
        with:
          name: test-results
          path: frontend/test-results/
          retention-days: 7
```

## File to Create

- `.github/workflows/uat.yml` — the full workflow YAML (implementation of the sketch above)
