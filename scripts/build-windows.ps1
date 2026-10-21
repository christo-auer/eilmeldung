# Build eilmeldung on Windows from a local clone.
#
# For contributors and developers who want to build local changes.
# For installing the latest code from GitHub without cloning, use
# scripts/install-windows.ps1 instead.
#
# Prerequisites:
#   - Rust (https://rustup.rs) with the x86_64-pc-windows-msvc target
#   - Visual Studio Build Tools with the "Desktop development with C++"
#     workload (winget install Microsoft.VisualStudio.2022.BuildTools)
#
# All other dependencies (TLS, SQLite) are built from source by cargo.
#
# Usage:
#   .\scripts\build-windows.ps1
#   .\scripts\build-windows.ps1 -Debug

param(
    [switch]$Debug
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Check for MSVC C++ build tools (needed to compile bundled C dependencies)
# ---------------------------------------------------------------------------
$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
$vsPath = $null
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
}

if (-not $vsPath) {
    Write-Error @"
Could not find the MSVC C++ build tools.
Make sure Visual Studio Build Tools are installed with the C++ workload:
  winget install Microsoft.VisualStudio.2022.BuildTools
"@
    exit 1
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error @"
cargo not found. Install Rust from https://rustup.rs and re-run this script.
"@
    exit 1
}

# ---------------------------------------------------------------------------
# Set build environment and build
# ---------------------------------------------------------------------------
$env:RUSTFLAGS = "-C target-feature=+crt-static"

$buildProfile = if ($Debug) { "debug" } else { "release" }
$cargoArgs = @("build", "--target", "x86_64-pc-windows-msvc")
if (-not $Debug) { $cargoArgs += "--release" }

Write-Host ""
Write-Host "Building eilmeldung ($buildProfile)..."
cargo @cargoArgs
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$binaryPath = "target\x86_64-pc-windows-msvc\$buildProfile\eilmeldung.exe"
Write-Host ""
Write-Host "Build successful: $binaryPath"
