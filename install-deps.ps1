#!/usr/bin/env pwsh
# install-deps.ps1 — Bootstrap build dependencies for ponder/sdlc
#
# Checks for Rust (rustup), Node.js (≥18), and just. Always prints a report
# of what was found and what needs installing, then prompts before acting.
#
# Usage:
#   pwsh install-deps.ps1               # report + interactive prompt
#   pwsh install-deps.ps1 --auto-approve  # report + install without prompting

param(
    [switch]$AutoApprove
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# ── Helpers ───────────────────────────────────────────────────────────────────

function Write-Header([string]$msg) {
    Write-Host "`n  $msg" -ForegroundColor Cyan
}

function Write-Found([string]$msg) {
    Write-Host "  [OK]   $msg" -ForegroundColor Green
}

function Write-Missing([string]$msg) {
    Write-Host "  [NEED] $msg" -ForegroundColor Yellow
}

function Write-Info([string]$msg) {
    Write-Host "         $msg" -ForegroundColor DarkGray
}

# Load nvm into the current session (unix only). Returns $true if nvm loaded.
function Import-Nvm {
    if ($IsWindows) { return $false }
    $nvmSh = "$HOME/.nvm/nvm.sh"
    if (-not (Test-Path $nvmSh)) { return $false }
    # Evaluate nvm.sh and capture env changes
    $envDump = bash -c ". '$nvmSh' && env"
    foreach ($line in $envDump) {
        if ($line -match '^(PATH|NVM_DIR|NVM_BIN)=(.+)$') {
            [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
        }
    }
    return $true
}

# ── Detection ─────────────────────────────────────────────────────────────────

$needs = [System.Collections.Generic.List[string]]::new()

Write-Host ""
Write-Host "  Checking build dependencies..." -ForegroundColor White

# Rust / rustup
Write-Header "Rust"
if (Get-Command rustup -ErrorAction SilentlyContinue) {
    $rustupVer = (rustup --version 2>&1) -replace 'rustup ',''
    $rustcVer  = (rustc --version 2>&1) -replace 'rustc ',''
    Write-Found "rustup $rustupVer"
    Write-Found "rustc  $rustcVer"
} else {
    Write-Missing "rustup not found"
    if ($IsWindows) {
        Write-Info "will install via: winget install Rustlang.Rustup"
    } else {
        Write-Info "will install via: curl https://sh.rustup.rs | sh"
    }
    $needs.Add("rust")
}

# Node.js — check nvm first, then PATH
Write-Header "Node.js (>=18)"
$nvmAvailable = Import-Nvm
$nodeOk = $false
if (Get-Command node -ErrorAction SilentlyContinue) {
    $nodeVerStr = (node --version)
    $nodeMajor  = [int]($nodeVerStr -replace 'v(\d+)\..*','$1')
    if ($nodeMajor -ge 18) {
        $via = if ($nvmAvailable) { " (via nvm)" } else { "" }
        Write-Found "node $nodeVerStr$via"
        $nodeOk = $true
    } else {
        Write-Missing "node $nodeVerStr found but <18"
    }
}
if (-not $nodeOk) {
    if ($nvmAvailable) {
        Write-Info "nvm found — will install Node.js 22 via: nvm install 22"
    } elseif ($IsWindows) {
        Write-Info "will install via: winget install OpenJS.NodeJS.LTS"
    } elseif ($IsMacOS -and (Get-Command brew -ErrorAction SilentlyContinue)) {
        Write-Info "will install via: brew install node@22"
    } else {
        Write-Info "will install nvm, then Node.js 22"
    }
    $needs.Add("node")
}

# just
Write-Header "just (task runner)"
if (Get-Command just -ErrorAction SilentlyContinue) {
    $justVer = (just --version 2>&1) -replace 'just ',''
    Write-Found "just $justVer"
} else {
    Write-Missing "just not found"
    if ($IsWindows) {
        Write-Info "will install via: winget install Casey.Just"
    } elseif ($IsMacOS -and (Get-Command brew -ErrorAction SilentlyContinue)) {
        Write-Info "will install via: brew install just"
    } else {
        Write-Info "will install via: cargo install just"
    }
    $needs.Add("just")
}

# ── Summary ───────────────────────────────────────────────────────────────────

Write-Host ""

if ($needs.Count -eq 0) {
    Write-Host "  All dependencies are present. Run: just install" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "  Missing: $($needs -join ', ')" -ForegroundColor Yellow
Write-Host ""

# ── Confirm ───────────────────────────────────────────────────────────────────

if (-not $AutoApprove) {
    $reply = Read-Host "  Install missing dependencies? [y/N]"
    if ($reply -notmatch '^[Yy]') {
        Write-Host "  Aborted." -ForegroundColor DarkGray
        Write-Host ""
        exit 0
    }
}

# ── Install ───────────────────────────────────────────────────────────────────

if ($needs.Contains("rust")) {
    Write-Host ""
    Write-Host "  Installing Rust..." -ForegroundColor Cyan
    if ($IsWindows) {
        winget install --id Rustlang.Rustup --silent --accept-package-agreements --accept-source-agreements
    } else {
        $rustupScript = (Invoke-WebRequest -Uri 'https://sh.rustup.rs' -UseBasicParsing).Content
        $tmp = [System.IO.Path]::GetTempFileName() + '.sh'
        Set-Content -Path $tmp -Value $rustupScript
        bash $tmp -y --no-modify-path
        Remove-Item $tmp
        # Pull cargo into this session's PATH
        $cargoEnv = "$HOME/.cargo/env"
        if (Test-Path $cargoEnv) {
            bash -c "source '$cargoEnv' && env" | ForEach-Object {
                if ($_ -match '^(PATH|CARGO_HOME|RUSTUP_HOME)=(.+)$') {
                    [System.Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], 'Process')
                }
            }
        }
    }
    Write-Host "  [OK]   Rust installed" -ForegroundColor Green
}

if ($needs.Contains("node")) {
    Write-Host ""
    Write-Host "  Installing Node.js..." -ForegroundColor Cyan
    if ($IsWindows) {
        winget install --id OpenJS.NodeJS.LTS --silent --accept-package-agreements --accept-source-agreements
    } elseif ($nvmAvailable) {
        bash -c ". '$HOME/.nvm/nvm.sh' && nvm install 22 && nvm alias default 22"
        Import-Nvm | Out-Null
    } elseif ($IsMacOS -and (Get-Command brew -ErrorAction SilentlyContinue)) {
        brew install node@22
        brew link node@22 --force --overwrite
    } else {
        # Install nvm then Node
        bash -c 'curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.1/install.sh | bash'
        bash -c ". '$HOME/.nvm/nvm.sh' && nvm install 22 && nvm alias default 22"
        Import-Nvm | Out-Null
    }
    Write-Host "  [OK]   Node.js installed" -ForegroundColor Green
}

if ($needs.Contains("just")) {
    Write-Host ""
    Write-Host "  Installing just..." -ForegroundColor Cyan
    if ($IsWindows) {
        winget install --id Casey.Just --silent --accept-package-agreements --accept-source-agreements
    } elseif ($IsMacOS -and (Get-Command brew -ErrorAction SilentlyContinue)) {
        brew install just
    } else {
        cargo install just
    }
    Write-Host "  [OK]   just installed" -ForegroundColor Green
}

# ── Done ──────────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  Done. Open a new shell, then run: just install" -ForegroundColor Green
Write-Host ""
