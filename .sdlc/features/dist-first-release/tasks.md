# Tasks: dist-first-release

## T1 — Pre-flight: verify Cargo workspace version is 0.1.0

Confirm `[workspace.package] version = "0.1.0"` in root `Cargo.toml`. If not, this must be addressed by `dist-cargo-toml-fixes` before proceeding.

**Acceptance:** `grep 'version = "0.1.0"' Cargo.toml` returns a match under `[workspace.package]`.

---

## T2 — Pre-flight: verify release.yml workflow is correct

Confirm `.github/workflows/release.yml` is present, has the correct 5 matrix targets, uses the `dist` profile for the build, and the `release` job uses `softprops/action-gh-release@v2`.

**Acceptance:** Workflow file passes manual inspection — all 5 targets present, `--profile dist`, `generate_release_notes: true`.

---

## T3 — Create and push annotated tag v0.1.0

```bash
git tag -a v0.1.0 -m "Release v0.1.0 — first official binary distribution"
git push origin v0.1.0
```

**Acceptance:** `git tag -l v0.1.0` returns `v0.1.0`; tag is visible on GitHub.

---

## T4 — Monitor GitHub Actions release workflow

Watch the `Release` workflow run triggered by the `v0.1.0` tag. Confirm all 5 build matrix jobs succeed and the release job creates the GitHub Release.

**Acceptance:** All 5 CI jobs green; GitHub Release `v0.1.0` exists with 5 attached assets at `https://github.com/orchard9/sdlc/releases/tag/v0.1.0`.

---

## T5 — Smoke-test install.sh on macOS

Run the install script against the newly published release and verify the installed binary works:

```bash
curl -sSfL https://raw.githubusercontent.com/orchard9/sdlc/main/install.sh | sh
~/.local/bin/ponder --version
~/.local/bin/sdlc --version
```

**Acceptance:** Both commands print `ponder 0.1.0` (or equivalent). No errors during install.

---

## T6 — Document results in review artifact

Record the outcomes of T1–T5: CI status per target, GitHub Release URL, install test output. Note any issues encountered and how they were resolved.

**Acceptance:** `review.md` written with pass/fail per step, CI job URLs, and `--version` output captured.
