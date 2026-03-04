# Design: cargo-dist Cargo.toml Configuration

## Overview

This is a config-only change. No code changes are needed — only `Cargo.toml` is modified.

## Change

Add a `[workspace.metadata.dist]` section to `/Cargo.toml`:

```toml
[workspace.metadata.dist]
targets = [
    "aarch64-apple-darwin",
    "x86_64-apple-darwin",
    "x86_64-unknown-linux-musl",
    "aarch64-unknown-linux-musl",
    "x86_64-pc-windows-msvc",
]
install-path = ["~/.local/bin"]
```

### Why `[workspace.metadata.dist]`

cargo-dist reads configuration from `[workspace.metadata.dist]` in the root `Cargo.toml`. This is the standard location for project-wide cargo-dist settings. It controls which targets are built and where the installer script places the binary.

### Why musl Linux targets

`x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` produce fully statically linked binaries with no glibc dependency. This means the binary runs on any Linux distribution regardless of glibc version — ideal for a CLI tool distributed to heterogeneous environments. The existing `.github/workflows/release.yml` already uses these targets; the `[workspace.metadata.dist]` section makes this explicit in the source tree so cargo-dist tooling agrees.

### Why `~/.local/bin`

`~/.local/bin` is the XDG-standard user-local binary path present on virtually all modern Linux distros and included in the default `$PATH` for most desktop environments (Fedora, Ubuntu 20+, Arch, etc.). Installing here requires no root access and does not pollute system directories. The Windows and macOS targets are unaffected by this setting (they use platform-native install paths).

## Files Changed

| File | Change |
|---|---|
| `Cargo.toml` | Add `[workspace.metadata.dist]` section |

No other files change.
