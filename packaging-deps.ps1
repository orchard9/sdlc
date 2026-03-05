#!/usr/bin/env pwsh
# packaging-deps.ps1 — Install packaging tools needed for `just dist`
#
# Linux:   cargo-deb, cargo-generate-rpm
# macOS:   create-dmg
# Windows: cargo-wix
#
# Usage:
#   pwsh packaging-deps.ps1               # report + interactive prompt
#   pwsh packaging-deps.ps1 --auto-approve

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

# ── Detection ─────────────────────────────────────────────────────────────────

$needs = [System.Collections.Generic.List[string]]::new()

Write-Host ""
Write-Host "  Checking packaging dependencies..." -ForegroundColor White

if ($IsLinux) {
    Write-Header "cargo-deb (produces .deb packages)"
    if (Get-Command cargo-deb -ErrorAction SilentlyContinue) {
        $ver = (cargo deb --version 2>&1)
        Write-Found $ver
    } else {
        Write-Missing "cargo-deb not found"
        Write-Info "will install via: cargo install cargo-deb"
        $needs.Add("cargo-deb")
    }

    Write-Header "cargo-generate-rpm (produces .rpm packages)"
    if (Get-Command cargo-generate-rpm -ErrorAction SilentlyContinue) {
        $ver = (cargo generate-rpm --version 2>&1)
        Write-Found $ver
    } else {
        Write-Missing "cargo-generate-rpm not found"
        Write-Info "will install via: cargo install cargo-generate-rpm"
        $needs.Add("cargo-generate-rpm")
    }
}

if ($IsMacOS) {
    Write-Header "create-dmg (produces .dmg packages)"
    if (Get-Command create-dmg -ErrorAction SilentlyContinue) {
        $ver = (create-dmg --version 2>&1)
        Write-Found "create-dmg $ver"
    } else {
        Write-Missing "create-dmg not found"
        Write-Info "will install via: brew install create-dmg"
        $needs.Add("create-dmg")
    }
}

if ($IsWindows) {
    Write-Header "cargo-wix (produces .msi/.exe installer)"
    if (Get-Command cargo-wix -ErrorAction SilentlyContinue) {
        $ver = (cargo wix --version 2>&1)
        Write-Found $ver
    } else {
        Write-Missing "cargo-wix not found"
        Write-Info "will install via: cargo install cargo-wix"
        $needs.Add("cargo-wix")
    }
}

# ── Summary ───────────────────────────────────────────────────────────────────

Write-Host ""

if ($needs.Count -eq 0) {
    Write-Host "  All packaging dependencies are present. Run: just dist" -ForegroundColor Green
    Write-Host ""
    exit 0
}

Write-Host "  Missing: $($needs -join ', ')" -ForegroundColor Yellow
Write-Host ""

# ── Confirm ───────────────────────────────────────────────────────────────────

if (-not $AutoApprove) {
    $reply = Read-Host "  Install missing packaging dependencies? [y/N]"
    if ($reply -notmatch '^[Yy]') {
        Write-Host "  Aborted." -ForegroundColor DarkGray
        Write-Host ""
        exit 0
    }
}

# ── Install ───────────────────────────────────────────────────────────────────

if ($needs.Contains("cargo-deb")) {
    Write-Host ""
    Write-Host "  Installing cargo-deb..." -ForegroundColor Cyan
    cargo install cargo-deb
    Write-Host "  [OK]   cargo-deb installed" -ForegroundColor Green
}

if ($needs.Contains("cargo-generate-rpm")) {
    Write-Host ""
    Write-Host "  Installing cargo-generate-rpm..." -ForegroundColor Cyan
    cargo install cargo-generate-rpm
    Write-Host "  [OK]   cargo-generate-rpm installed" -ForegroundColor Green
}

if ($needs.Contains("create-dmg")) {
    Write-Host ""
    Write-Host "  Installing create-dmg..." -ForegroundColor Cyan
    brew install create-dmg
    Write-Host "  [OK]   create-dmg installed" -ForegroundColor Green
}

if ($needs.Contains("cargo-wix")) {
    Write-Host ""
    Write-Host "  Installing cargo-wix..." -ForegroundColor Cyan
    cargo install cargo-wix
    Write-Host "  [OK]   cargo-wix installed" -ForegroundColor Green
}

# ── Done ──────────────────────────────────────────────────────────────────────

Write-Host ""
Write-Host "  Done. Run: just dist" -ForegroundColor Green
Write-Host ""
