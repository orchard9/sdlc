# ponder — just recipes
# Install just: cargo install just  |  brew install just  |  winget install just
# Usage: just <recipe>

# Show available recipes
default:
    @just --list

# ─── Install ─────────────────────────────────────────────────────────────────

# Build frontend, install ponder + sdlc alias, install orch-tunnel
install: _frontend
    cargo install --path crates/sdlc-cli --locked
    just _symlink
    just _install-orch-tunnel

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
    #!powershell
    $ponder = (Get-Command ponder -ErrorAction Stop).Source
    $sdlc   = Join-Path (Split-Path $ponder) "sdlc.exe"
    if (Test-Path $sdlc) { Remove-Item $sdlc }
    New-Item -ItemType HardLink -Path $sdlc -Target $ponder | Out-Null
    Write-Host "  OK sdlc -> ponder (alias)"

[unix]
_install-orch-tunnel:
    #!/usr/bin/env sh
    if command -v orch-tunnel >/dev/null 2>&1; then
        printf '  \033[32m✓\033[0m orch-tunnel already installed\n'
    elif command -v brew >/dev/null 2>&1; then
        echo "  Installing orch-tunnel via Homebrew..."
        brew install orch-tunnel
    elif command -v gh >/dev/null 2>&1; then
        OS=$(uname -s | tr '[:upper:]' '[:lower:]')
        ARCH=$(uname -m)
        case "$ARCH" in arm64|aarch64) ARCH=arm64 ;; x86_64) ARCH=amd64 ;; esac
        DEST="$HOME/.local/bin"
        mkdir -p "$DEST"
        echo "  Installing orch-tunnel (${OS}-${ARCH}) to $DEST..."
        gh release download --repo orchard9/tunnel --pattern "orch-tunnel-${OS}-${ARCH}*" -D "$DEST"
        chmod +x "$DEST"/orch-tunnel-*
        ln -sf "$DEST"/orch-tunnel-* "$DEST/orch-tunnel"
    else
        printf '\n  \033[33m⚠\033[0m  orch-tunnel not installed\n'
        printf '     Needed only for: ponder ui --tunnel\n'
        printf '     Install: gh release download --repo orchard9/tunnel\n\n'
    fi

[windows]
_install-orch-tunnel:
    #!powershell
    if (Get-Command orch-tunnel -ErrorAction SilentlyContinue) {
        Write-Host "  OK orch-tunnel already installed"
    } elseif (Get-Command gh -ErrorAction SilentlyContinue) {
        $dest = "$env:USERPROFILE\.local\bin"
        New-Item -ItemType Directory -Force -Path $dest | Out-Null
        gh release download --repo orchard9/tunnel --pattern "orch-tunnel-windows-amd64*" -D $dest
        $bin = Get-Item "$dest\orch-tunnel-windows-amd64*"
        Rename-Item $bin.FullName "orch-tunnel.exe" -Force
        Write-Host "  OK orch-tunnel installed to $dest"
    } else {
        Write-Host ""
        Write-Host "  WARN orch-tunnel not installed"
        Write-Host "       Needed only for: ponder ui --tunnel"
        Write-Host "       Install: gh release download --repo orchard9/tunnel"
        Write-Host ""
    }
