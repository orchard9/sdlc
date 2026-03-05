# ponder — just recipes
# Bootstrap deps: pwsh install-deps.ps1
# Install just:   cargo install just  |  brew install just  |  winget install just
# Usage:          just <recipe>

# Use PowerShell 7 on Windows
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

# Show available recipes
default:
    @just --list

# ─── Bootstrap ───────────────────────────────────────────────────────────────

# Print instructions for installing build and packaging dependencies
deps:
    @printf '\n  Build dependencies (Rust, Node.js, just):\n'
    @printf '    pwsh install-deps.ps1\n\n'
    @printf '  Packaging dependencies (cargo-deb, cargo-generate-rpm, create-dmg, cargo-wix):\n'
    @printf '    pwsh packaging-deps.ps1\n\n'
    @printf '  Requires pwsh (PowerShell 7+):\n'
    @printf '    Linux:   sudo apt install powershell  |  sudo snap install powershell\n'
    @printf '    macOS:   brew install --cask powershell\n'
    @printf '    Windows: built-in (pwsh.exe)\n\n'

# ─── Install ─────────────────────────────────────────────────────────────────

# Build frontend, install ponder + sdlc alias
install: _frontend
    cargo install --path crates/sdlc-cli --locked
    just _symlink
    @printf '\n  \033[36mOptional:\033[0m For remote UI tunneling, install orch-tunnel:\n'
    @printf '           https://github.com/orchard9/tunnel\n\n'

# Build without installing
build: _frontend
    cargo build --all

# ─── Distribution ────────────────────────────────────────────────────────────

# Build distributable packages for the host platform
[unix]
dist: _frontend _dist-unix

[windows]
dist: _frontend _dist-windows

[linux]
_dist-unix: _build-dist _pkg-targz _pkg-deb _pkg-rpm

[macos]
_dist-unix: _build-dist _pkg-targz
    @printf '\n  \033[33mNote:\033[0m DMG creation requires macOS CI (create-dmg). Skipping.\n'

[linux]
_build-dist:
    cargo build --profile dist --bin ponder

[macos]
_build-dist:
    cargo build --profile dist --bin ponder

[linux]
_pkg-targz:
    #!/usr/bin/env sh
    TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
    ARCHIVE="ponder-${TRIPLE}.tar.gz"
    tar czf "$ARCHIVE" -C target/dist ponder
    printf '  \033[32m✓\033[0m %s\n' "$ARCHIVE"

[macos]
_pkg-targz:
    #!/usr/bin/env sh
    TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
    ARCHIVE="ponder-${TRIPLE}.tar.gz"
    tar czf "$ARCHIVE" -C target/dist ponder
    printf '  \033[32m✓\033[0m %s\n' "$ARCHIVE"

[linux]
_pkg-deb:
    #!/usr/bin/env sh
    TRIPLE=$(rustc -vV | sed -n 's/^host: //p')
    cargo deb -p sdlc-cli --no-build --profile dist
    printf '  \033[32m✓\033[0m .deb built\n'

[linux]
_pkg-rpm:
    #!/usr/bin/env sh
    cd crates/sdlc-cli
    cargo generate-rpm --profile dist
    printf '  \033[32m✓\033[0m .rpm built\n'

[windows]
_dist-windows:
    #!powershell
    cargo build --profile dist --bin ponder
    $triple = (rustc -vV | Select-String 'host:').ToString().Split(': ')[1].Trim()
    $archive = "ponder-$triple.zip"
    Compress-Archive -Force -Path "target/$triple/dist/ponder.exe" -DestinationPath $archive
    Write-Host "  OK  $archive"
    if (Get-Command "cargo-wix" -ErrorAction SilentlyContinue) {
        cargo wix --nocapture --target $triple
        Write-Host "  OK  WiX installer built"
    } else {
        Write-Host "  !   cargo-wix not installed — skipping installer (cargo install cargo-wix)"
    }

# ─── Development ─────────────────────────────────────────────────────────────

# Run tests (skips npm build step)
[unix]
test:
    SDLC_NO_NPM=1 cargo test --all

[windows]
test:
    $env:SDLC_NO_NPM = "1"; cargo test --all

# Lint (clippy + TypeScript)
[unix]
lint:
    cargo clippy --all -- -D warnings
    cd frontend && npx tsc --noEmit

[windows]
lint:
    cargo clippy --all -- -D warnings
    Set-Location frontend; npx tsc --noEmit

# Remove build artifacts
[unix]
clean:
    cargo clean
    rm -rf frontend/dist frontend/node_modules

[windows]
clean:
    cargo clean
    if (Test-Path frontend/dist)         { Remove-Item -Recurse -Force frontend/dist }
    if (Test-Path frontend/node_modules) { Remove-Item -Recurse -Force frontend/node_modules }

# ─── Internal ────────────────────────────────────────────────────────────────

_frontend:
    npm --prefix frontend ci
    npm --prefix frontend run build

[unix]
_symlink:
    #!/usr/bin/env sh
    PONDER=$(command -v ponder)
    ln -sf "$PONDER" "$(dirname "$PONDER")/sdlc"
    printf '  \033[32m✓\033[0m sdlc -> ponder (alias)\n'

[windows]
_symlink:
    #!pwsh
    $ponder = (Get-Command ponder -ErrorAction Stop).Source
    $sdlc   = Join-Path (Split-Path $ponder) "sdlc.exe"
    if (Test-Path $sdlc) { Remove-Item $sdlc }
    New-Item -ItemType HardLink -Path $sdlc -Target $ponder | Out-Null
    Write-Host "  OK sdlc -> ponder (alias)"
