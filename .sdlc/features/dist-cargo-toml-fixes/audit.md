# Security Audit: cargo-dist Cargo.toml Configuration

## Change Summary

Added `[workspace.metadata.dist]` section to root `Cargo.toml` specifying five release targets and `install-path = ["~/.local/bin"]`. No Rust source files were modified.

## Threat Surface Assessment

**Attack surface introduced:** None.

This change modifies only build metadata in a TOML configuration file. It does not:

- Add new code paths or executable logic
- Introduce new network endpoints or connections
- Handle user input or external data
- Store, transmit, or process secrets or credentials
- Change authentication or authorization behavior
- Modify runtime behavior of the binary

## Findings

### F1: Supply chain — musl target toolchain

**Severity:** Informational
**Finding:** Building against `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` requires the musl toolchain at build time. These are standard Rust targets provided by the official Rust toolchain distribution.
**Action:** No action needed. Standard Rust toolchain supply chain controls apply. The existing CI already uses these targets.

### F2: Install-path scope

**Severity:** Informational
**Finding:** `install-path = ["~/.local/bin"]` installs the binary into the user's home directory without root. This is intentional and desirable — it avoids privilege escalation in the installer script.
**Action:** No action needed. This is the correct XDG user-local install path.

## Verdict

**APPROVED — no security concerns.**

The change is limited to build metadata. It introduces no new runtime behavior, attack surface, or security risk. The musl targets and `~/.local/bin` install path improve security posture by producing fully static binaries (no dynamic linking vulnerabilities) and avoiding root-level installation.
