# Audit: Document sdlc update as Update Mechanism

## Scope

Two file changes:
1. `README.md` — new `### Updating` section (~11 lines of documentation)
2. `crates/sdlc-cli/src/cmd/init/mod.rs` — one `println!` string changed

## Security Analysis

### Attack Surface

**None.** Both changes are purely presentational:
- The README change is static documentation. It contains no dynamic content, no external links that could be attacker-controlled, and no instructions that would cause users to run unsafe commands. The `sdlc update` command it documents already exists and already runs when users invoke it.
- The `println!` change replaces one user-visible string with another. There is no logic change, no new code path, no input parsing, and no file system or network interaction added.

### Trust Model

No change. The `sdlc update` command is already trusted by users who installed `sdlc`. Documenting it does not expand the trust boundary.

### Dependency Surface

No new dependencies introduced. No changes to `Cargo.toml` files.

### Data Handling

No user data is read, written, or transmitted by either change.

## Findings

None.

## Verdict

**APPROVED.** This feature has no meaningful security surface. Both changes are additive documentation and a one-line UX message fix.
