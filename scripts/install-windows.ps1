# Install eilmeldung on Windows from the latest main branch on GitHub.
#
# For end users and power users who want the latest unreleased code.
# Does not require cloning the repository. Run directly:
#
#   irm https://raw.githubusercontent.com/christo-auer/eilmeldung/main/scripts/install-windows.ps1 -OutFile install-eilmeldung.ps1
#   .\install-eilmeldung.ps1
#
# Prerequisites:
#   - Rust (https://rustup.rs) with the x86_64-pc-windows-msvc target
#   - Visual Studio Build Tools with the "Desktop development with C++"
#     workload (winget install Microsoft.VisualStudio.2022.BuildTools)
#
# All other dependencies (TLS, SQLite) are built from source by cargo.
#
# Usage:
#   .\install-windows.ps1

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
# Set build environment and install
# ---------------------------------------------------------------------------
$env:RUSTFLAGS = "-C target-feature=+crt-static"

Write-Host ""
Write-Host "Installing eilmeldung from latest main..."

cargo install --locked --git https://github.com/christo-auer/eilmeldung --target x86_64-pc-windows-msvc

if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

Write-Host ""
Write-Host "Installed successfully. Run 'eilmeldung' to start."
