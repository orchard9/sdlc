# Security Audit: dist-developer-docs

## Scope

Single file changed: `DEVELOPER.md` (documentation only, no code changes).

## Security Surface

This change has no runtime security surface. `DEVELOPER.md` is a Markdown file read by humans.
It is not parsed by the application, not embedded in any binary, and not served via any endpoint.

## Audit Findings

### A1 — Shell commands in documentation (INFORMATIONAL)

The new fallback section includes shell commands. These are instructions to contributors, not
commands executed by the application. No injection vectors exist.

Specific commands reviewed:
- `npm --prefix frontend ci` — standard npm install, no unusual flags
- `npm --prefix frontend run build` — standard frontend build, no unusual flags
- `cargo install --path crates/sdlc-cli --locked` — installs from local path with `--locked`,
  which is the safest form (pins to Cargo.lock)
- `ln -sf "$PONDER" "$(dirname "$PONDER")/sdlc"` — symlink creation using `$()` command
  substitution with `command -v ponder`. This is safe documentation of a shell pattern.

The Windows note directs users to create a hard link in `%USERPROFILE%\.cargo\bin\` — a standard
user-writable location.

### A2 — No secrets, credentials, or tokens (PASS)

No secrets, API keys, URLs, or credentials added to documentation.

### A3 — No external URL introduction (PASS)

No new external URLs added. Existing links (rustup.rs, nodejs.org) unchanged.

### A4 — Supply chain: cargo install --locked (PASS)

The fallback uses `--locked` flag which pins the build to the repository's `Cargo.lock`. This is
the recommended security-conscious install form. No supply chain concern.

## Decision

**APPROVE** — Documentation-only change with no security surface. All findings are informational
or passing. No action required.
