---
session: 2
timestamp: 2026-03-03T03:30:00Z
orientation:
  current: "cargo-dist identified as the right tool for Rust CLI distribution; enterprise-grade checklist defined; make vs. cmake is a developer concern, not an end-user concern"
  next: "Validate cargo-dist against Gitea release target (GitHub-only?), decide on code signing scope (v1 vs. v2), and fix the 4 TypeScript errors as a separate immediate PR"
  commit: "Decision on release pipeline (cargo-dist or custom), platform target list, and sequenced plan: immediate fix → developer install → end-user install → enterprise hardening"
---

## Session 2: Enterprise Grade Quality

**Focus**: User added "enterprise grade quality" to this ponder. Expanding from "fix make and copy envault" to a full distribution story.

**Team recruited this session:**
- Recruited: Lia Santangelo · Release Engineering & Distribution Expert
- Recruited: Ray Okafor · Enterprise IT Security & Gatekeeping

---

### The envault pattern dissected

Examined `/Users/jordanwashburn/Workspace/orchard9/envault/install.sh` and `install.ps1`.

**The envault distribution pattern:**
- `install.sh` — bash, platform+arch detection via `uname`, downloads pre-built binary from GitHub Releases (`.tar.gz`), installs to `$HOME/.local/bin`, falls back to `go install`
- `install.ps1` — PowerShell, downloads `.zip` from GitHub Releases, installs to `$env:USERPROFILE\.local\bin`, falls back to `go install`
- Releases: GitHub Releases with platform-specific binary archives
- No checksum verification (gap identified)

---

### Lia Santangelo · Release Engineering Expert

*The right answer for a Rust CLI in 2026 is cargo-dist, not hand-rolled install scripts.*

**cargo-dist capabilities:**
1. GitHub Actions CI — generates entire release workflow for darwin-aarch64, darwin-x86_64, linux-x86_64-musl, linux-aarch64-musl, windows-x86_64, windows-aarch64
2. Install scripts — auto-generated `install.sh` + `install.ps1`, kept in sync with release structure
3. Checksum generation — SHA256 checksums published as `checksums.txt` per release
4. Homebrew tap — optional, auto-generated on release
5. cargo-binstall compatibility — metadata in Cargo.toml tells cargo-binstall where to find pre-built binaries
6. Handles the `make install` problem — end users never need to compile; developers can `cargo install --path .`

**The Makefile problem is a developer UX problem, not an end-user problem.**
- Developer install: `cargo install --path .` or `just install` (Justfile, cross-platform unlike Make)
- End-user install: cargo-dist-generated install.sh / install.ps1

? Open: cargo-dist is GitHub Releases native. Does sdlc publish to GitHub, or Gitea? The Gitea fleet was noted in memory — this is a real question.

---

### Ray Okafor · Enterprise IT Security

*Enterprise grade means: signed binaries, non-root install, no auto-update, verifiable provenance.*

**Hard blockers for enterprise approval:**
1. Code signing — macOS notarization (Gatekeeper), Windows Authenticode (no "Unknown publisher" dialog). MDM systems (Jamf, Intune, SCCM) require signed binaries.
2. Non-root install — must default to user home directory. Never require sudo for normal install.
3. No auto-update — `sdlc update` as explicit opt-in only. Auto-update bypasses change management.

**Strong preferences (unlocks most enterprises):**
4. SBOM — `cargo cyclonedx` or `cargo sbom`. Some regulated contexts require this.
5. Checksum verification in install script — SHA256 verify before executing binary.
6. Proxy support — HTTP_PROXY/HTTPS_PROXY/NO_PROXY must be respected by any network call.
7. Telemetry opt-out — `SDLC_NO_TELEMETRY=1` env var. Must be documented.

**Nice to have:**
8. Version pinning: `SDLC_VERSION=1.2.3 curl ... | bash`
9. Air-gap install documentation
10. Enterprise distribution guide (1-page security assessment doc)

*make vs. cmake is irrelevant at enterprise scale. Enterprises download approved binaries; they don't compile.*

---

### Synthesis

⚑ Decided: Use **cargo-dist** as the release pipeline, not hand-rolled install scripts copied from envault. cargo-dist is purpose-built for exactly this problem in the Rust ecosystem.

⚑ Decided: Separate the developer install path from the end-user install path clearly. Developer = `cargo install --path .` or Justfile target. End-user = cargo-dist install scripts.

⚑ Decided: Checksum verification is non-negotiable in whatever install script ships.

⚑ Decided: Non-root install (`$HOME/.local/bin`) is the default. This is already the envault pattern — preserve it.

⚑ Decided: No auto-update. `sdlc update` as explicit command.

? Open: GitHub vs. Gitea as release target — cargo-dist is GitHub-native. This is the primary compatibility question before committing to cargo-dist.

? Open: Is code signing in scope for v1, or is it a v2 hardening step? (Requires: Apple Developer ID cert, Windows EV cert from a CA, significant CI complexity.)

? Open: Does sdlc currently phone home for anything? Version check? Telemetry? If yes, needs opt-out path.

---

### Immediate action item (separate from distribution work)

4 TypeScript errors are blocking `make install` on Ubuntu — these are bugs, not distribution problems, and should be fixed as a standalone commit:

| File | Error |
|------|-------|
| `frontend/src/components/runs/RunsHeatmap.tsx:40` | Unused variable `startMs` |
| `frontend/src/pages/ActionsPage.tsx:516` | `string | undefined` not assignable to `string` |
| `frontend/src/pages/ActionsPage.tsx:713` | `action_id` missing from `OrchestratorWebhookEvent` |
| `frontend/src/pages/ActionsPage.tsx:722` | `path` missing from `OrchestratorWebhookEvent` |

These should be fixed before the distribution work begins — they make the project look broken to new contributors.

---

### Proposed sequencing

**Phase 1 — Immediate (fix the broken, now)**
- Fix 4 TypeScript errors blocking `make install`
- Document developer install path in DEVELOPER.md

**Phase 2 — Foundation (make install cross-platform)**
- Add `cargo-dist` to the project (or evaluate vs. custom if Gitea-only)
- Set up GitHub Actions release workflow for multi-platform binaries
- Replace `make install` with a `just install` target (Justfile) or document `cargo install --path .`

**Phase 3 — End-user distribution**
- Publish GitHub Releases with binary archives for all 6 platforms
- Cargo-dist install scripts with checksum verification
- Homebrew tap (optional, v1)
- cargo-binstall metadata

**Phase 4 — Enterprise hardening**
- Code signing (macOS notarization + Windows Authenticode)
- SBOM generation
- Enterprise distribution guide
- Proxy support audit
- Version pinning in install scripts

---

## Product Summary

### What we explored
How to evolve sdlc's install experience from a broken `make`-based developer workflow into an enterprise-grade distribution pipeline that works on Windows, Mac, and Linux without requiring compilation.

### Key shifts
Previously we thought the fix was "copy the envault install scripts and adapt for Rust." We now know the right answer is cargo-dist — a purpose-built Rust release tool that handles cross-compilation CI, install scripts, checksums, Homebrew, and cargo-binstall integration automatically. The envault pattern is good shape, but hand-rolling it would mean missing checksums, no package manager integration, and ongoing maintenance burden.

We also separated the problem into four distinct phases: fix immediate bugs → fix developer install → build end-user distribution → add enterprise hardening. These were previously bundled together, which made the problem feel overwhelming.

### Implications
The 4 TypeScript errors should be fixed immediately — they're a quality signal problem independent of distribution. The cargo-dist decision is the core architectural choice for Phase 2 and should be validated against the Gitea question first (cargo-dist is GitHub-native; if Gitea is the release target, we may need a different approach). Code signing is Phase 4 and can wait until there are actual enterprise customers asking for it.

### Still open
1. Is GitHub Releases the publish target, or Gitea? This determines whether cargo-dist works or whether we need a custom pipeline.
2. Is code signing (Apple notarization + Windows Authenticode) in scope for v1, or treated as enterprise-hardening for a later phase?
