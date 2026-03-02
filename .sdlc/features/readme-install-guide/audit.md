# Security Audit: SSH and make install in README

## Surface

Documentation-only change. Three edits to `README.md` — no code, no config, no runtime behavior modified.

## Findings

### Supply chain / URL integrity

**Finding:** The SSH URL `ssh://git@github.com/orchard9/sdlc` and HTTPS URL `https://github.com/orchard9/sdlc` both point to the same GitHub repository. `cargo install --git` fetches and compiles from source — the same as before. The SSH variant does not change what is compiled or installed, only the transport mechanism.

**Action:** Accept. No new supply chain exposure. SSH is strictly equivalent to HTTPS for `cargo install --git` from the same repository.

### Clone URL (`git clone git@github.com:orchard9/sdlc.git`)

**Finding:** The `make install` subsection documents cloning via SSH SCP shorthand (`git@github.com:`). This is a standard Git SSH clone syntax and is equivalent to `git clone ssh://git@github.com/orchard9/sdlc.git`.

**Action:** Accept. No security concern. SSH clone is the correct documented path for users with SSH keys.

### `make install` command

**Finding:** `make install` runs `cargo install --path crates/sdlc-cli` which installs a binary from local source — exactly the same trust model as any `cargo install`. No network calls beyond what Cargo itself makes for dependency resolution.

**Action:** Accept. Makefile is in the repository under version control. No elevated privilege required.

### DEVELOPER.md link

**Finding:** The blockquote links to `DEVELOPER.md` — a relative path in the same repository. No external URLs, no redirect chains.

**Action:** Accept. Trivial internal link.

## Summary

No security findings require remediation. This is a documentation-only change with no new attack surface, no new network endpoints, and no code execution changes.

## Verdict

Approved.
