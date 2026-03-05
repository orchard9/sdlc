# ponder — just recipes
# Install just: cargo install just  |  brew install just  |  winget install just
# Usage: just <recipe>

# Use PowerShell 7 on Windows
set windows-shell := ["pwsh.exe", "-NoLogo", "-Command"]

# Show available recipes
default:
    @just --list

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

