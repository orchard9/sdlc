# Distribution Architecture Options

## The Recommendation: cargo-dist

cargo-dist (https://opensource.axo.dev/cargo-dist/) is the purpose-built Rust release tool.

### What it handles automatically

| Concern | cargo-dist behavior |
|---------|---------------------|
| Cross-compilation | GitHub Actions matrix for all 6 targets |
| Install scripts | Auto-generated install.sh + install.ps1 |
| Checksums | SHA256 checksums.txt per release |
| GitHub Releases | Automatic upload of all artifacts |
| Homebrew tap | Optional, auto-generated formula |
| cargo-binstall | Metadata wired automatically |

### Platform targets

| OS | Arch | Notes |
|----|------|-------|
| Darwin | aarch64 (arm64) | Apple Silicon Mac |
| Darwin | x86_64 | Intel Mac |
| Linux | x86_64-musl | Most Linux, static binary |
| Linux | aarch64-musl | ARM Linux (Pi, AWS Graviton) |
| Windows | x86_64 | Standard Windows |
| Windows | aarch64 | Windows on ARM (Surface, Copilot+ PCs) |

### What cargo-dist does NOT handle

- Code signing (Phase 4 — separate CI steps required)
- SBOM generation (separate: cargo cyclonedx)
- Gitea releases (GitHub-native; needs assessment)
- APT/RPM packages (requires additional tooling, e.g. cargo-generate-rpm)

## Alternative: Custom pipeline (if Gitea is primary)

If Gitea is the release target:
- Use `cross` crate for cross-compilation in GitHub Actions (or Gitea Actions)
- Upload artifacts via Gitea Release API
- Ship envault-style install.sh + install.ps1 but ADD checksum verification
- Add: `sha256sum --check checksums.txt` step before binary execution

## Enterprise checklist (Phase 4)

### Code signing

**macOS (notarization)**
- Requires Apple Developer ID Application certificate ($99/year)
- CI step: sign binary → staple → notarize with Apple's servers
- Without this: Gatekeeper blocks execution on macOS 13+
- Tooling: rcodesign (Rust tool), or xcrun altool/notarytool

**Windows (Authenticode)**
- Requires EV (Extended Validation) Code Signing certificate (~$300-500/year from DigiCert, Sectigo)
- CI step: signtool.exe or osslsigncode
- Without this: Windows Defender SmartScreen warning on download

**Linux (GPG)**
- Less critical — Linux enterprise (RHEL, Ubuntu LTS) typically uses APT/RPM repo signing instead
- For standalone binary: detached GPG signature `.asc` file alongside binary

### SBOM

```bash
cargo install cargo-cyclonedx
cargo cyclonedx --format json
```

Generates `bom.json` (CycloneDX format) — publish alongside release artifacts.

### Telemetry opt-out

Any network calls should check:
```rust
if std::env::var("SDLC_NO_TELEMETRY").is_ok() {
    return;
}
```

Document in README and help text.
