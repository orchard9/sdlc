# Security Audit: Playwright CI Gate on PRs

## Surface

A GitHub Actions workflow triggered on `pull_request`. No secrets, no external write operations, no deployments.

## Findings

### Supply Chain: Third-Party Actions
All actions are pinned to stable major version tags (`@v4`, `@stable`). These are widely used, maintained actions from trusted publishers (GitHub, dtolnay). Risk: low. Mitigation if desired: pin to SHA digests — not required for this use case.

### Fork PR Isolation
`pull_request` (not `pull_request_target`) is used. This means workflows from fork PRs run in an isolated context with read-only permissions and no access to repository secrets. The workflow does not request any permissions beyond the default read scope. This is correct and safe.

### No Secrets Accessed
The workflow does not reference `secrets.*`. No credentials are required to build or run Playwright tests.

### `cargo build` from Untrusted Code
A PR could introduce malicious Rust code that executes during `cargo build`. This is an inherent risk in any Rust CI workflow. Mitigations:
- Fork PRs receive no write permissions or secrets.
- The runner is ephemeral (GitHub-hosted `ubuntu-22.04`).
- No credentials are in scope for exfiltration.

### Playwright Browser Download
`npx playwright install --with-deps chromium` downloads a Chromium binary from the Playwright CDN over HTTPS. This is standard practice and the same binary version is pinned by `package-lock.json`. Risk: low.

## Verdict

No significant security concerns. The workflow follows GitHub Actions security best practices for PR CI gates (no `pull_request_target`, no secrets, ephemeral runners).
