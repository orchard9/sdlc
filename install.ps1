# Install ponder — feature lifecycle CLI
# Binary: ponder.exe  |  Alias: sdlc.cmd -> ponder
#
# Usage:
#   irm https://raw.githubusercontent.com/orchard9/sdlc/main/install.ps1 | iex
#
# Override install directory:
#   $env:PONDER_INSTALL = "C:\tools"; irm ... | iex

$ErrorActionPreference = "Stop"

$Repo       = "orchard9/sdlc"
$InstallDir = if ($env:PONDER_INSTALL) { $env:PONDER_INSTALL } else { "$env:USERPROFILE\.local\bin" }

# ── Detect architecture ───────────────────────────────────────────────────────

$Arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture
$Target = switch ($Arch) {
    "X64"   { "x86_64-pc-windows-msvc" }
    "Arm64" { "x86_64-pc-windows-msvc" }  # fallback to x64 until arm64 build is available
    default { throw "Unsupported architecture: $Arch" }
}

# ── Resolve latest version ────────────────────────────────────────────────────

$Release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest"
$Version = $Release.tag_name

if (-not $Version) {
    throw "Could not determine latest version"
}

# ── Download and install ──────────────────────────────────────────────────────

$Url     = "https://github.com/$Repo/releases/download/$Version/ponder-$Target.zip"
$TmpDir  = Join-Path ([System.IO.Path]::GetTempPath()) ([System.IO.Path]::GetRandomFileName())
$ZipPath = Join-Path $TmpDir "ponder.zip"

New-Item -ItemType Directory -Path $TmpDir | Out-Null

Write-Host "Installing ponder $Version ($Target)..."

Invoke-WebRequest -Uri $Url -OutFile $ZipPath
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

Move-Item -Force (Join-Path $TmpDir "ponder.exe") (Join-Path $InstallDir "ponder.exe")

# Create sdlc.cmd alias (cmd.exe + PowerShell compatible)
$AliasContent = "@echo off`r`n`"%~dp0ponder.exe`" %*"
Set-Content -Path (Join-Path $InstallDir "sdlc.cmd") -Value $AliasContent

Remove-Item -Recurse -Force $TmpDir

Write-Host "  OK  ponder -> $InstallDir\ponder.exe"
Write-Host "  OK  sdlc   -> $InstallDir\ponder.exe  (alias via sdlc.cmd)"

# ── Add to PATH ───────────────────────────────────────────────────────────────

$UserPath = [System.Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [System.Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
    Write-Host ""
    Write-Host "Added $InstallDir to your PATH."
    Write-Host "Restart your terminal for the change to take effect."
}

Write-Host ""
Write-Host "Run 'ponder --version' or 'sdlc --version' to verify."
