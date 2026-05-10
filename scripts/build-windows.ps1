# Build eilmeldung on Windows
#
# This script is self-contained: it will automatically install vcpkg and
# libxml2 if they are not already present.
#
# The only manual prerequisite is Perl, which is required to compile OpenSSL
# from source. Install it via scoop or Strawberry Perl:
#   scoop install perl
#   https://strawberryperl.com
#
# Usage:
#   .\scripts\build-windows.ps1
#   .\scripts\build-windows.ps1 -PerlPath "C:\custom\perl\bin\perl.exe"
#   .\scripts\build-windows.ps1 -VcpkgRoot "D:\my-vcpkg"
#   .\scripts\build-windows.ps1 -Debug

param(
    [string]$VcpkgRoot = "$env:LOCALAPPDATA\vcpkg",
    [string]$PerlPath  = "",
    [switch]$Debug
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Resolve Perl
# ---------------------------------------------------------------------------
if (-not $PerlPath) {
    $candidates = @(
        "C:\Strawberry\perl\bin\perl.exe",
        "$env:USERPROFILE\scoop\apps\perl\current\perl\bin\perl.exe",
        (Get-Command perl -ErrorAction SilentlyContinue)?.Source
    )
    foreach ($c in $candidates) {
        if ($c -and (Test-Path $c)) { $PerlPath = $c; break }
    }
}

if (-not $PerlPath -or -not (Test-Path $PerlPath)) {
    # Try to install via scoop if available
    if (Get-Command scoop -ErrorAction SilentlyContinue) {
        Write-Host "Perl not found -- installing via scoop..."
        scoop install perl
        # Re-detect via PATH after install
        $found = Get-Command perl -ErrorAction SilentlyContinue
        if ($found) {
            $PerlPath = $found.Source
            Write-Host "  Perl ready at $PerlPath"
        }
    }
}

if (-not $PerlPath -or -not (Test-Path $PerlPath)) {
    Write-Error @"
Perl not found and could not be installed automatically.
Install it manually via:
  scoop install perl
or download Strawberry Perl from https://strawberryperl.com
Then re-run this script or pass -PerlPath 'C:\path\to\perl.exe'.
"@
    exit 1
}

# ---------------------------------------------------------------------------
# Bootstrap vcpkg if not present
# ---------------------------------------------------------------------------
$vcpkgExe = Join-Path $VcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "vcpkg not found at '$VcpkgRoot' -- installing..."

    if (Test-Path $VcpkgRoot) {
        # Directory exists but no vcpkg.exe -- bootstrap only
        Write-Host "  Bootstrapping vcpkg..."
        & "$VcpkgRoot\bootstrap-vcpkg.bat" -disableMetrics
    } else {
        Write-Host "  Cloning vcpkg..."
        git clone https://github.com/microsoft/vcpkg $VcpkgRoot
        Write-Host "  Bootstrapping vcpkg..."
        & "$VcpkgRoot\bootstrap-vcpkg.bat" -disableMetrics
    }

    if (-not (Test-Path $vcpkgExe)) {
        Write-Error "vcpkg bootstrap failed. Check the output above for errors."
        exit 1
    }
    Write-Host "  vcpkg ready."
}

# ---------------------------------------------------------------------------
# Install libxml2 static if not present
# ---------------------------------------------------------------------------
$libxmlLib = Join-Path $VcpkgRoot "installed\x64-windows-static\lib\xml2.lib"

if (-not (Test-Path $libxmlLib)) {
    Write-Host "libxml2 static not found -- installing via vcpkg (this may take several minutes)..."
    & $vcpkgExe install libxml2:x64-windows-static

    if (-not (Test-Path $libxmlLib)) {
        Write-Error "libxml2 installation failed. Check the output above for errors."
        exit 1
    }
    Write-Host "  libxml2 ready."
}

# ---------------------------------------------------------------------------
# Set build environment
# ---------------------------------------------------------------------------
$env:VCPKG_ROOT           = $VcpkgRoot
$env:VCPKGRS_DYNAMIC      = "0"
$env:PKG_CONFIG_PATH      = "$VcpkgRoot\installed\x64-windows-static\lib\pkgconfig"
$env:CMAKE_TOOLCHAIN_FILE = "$VcpkgRoot\scripts\buildsystems\vcpkg.cmake"
$env:VCPKG_TARGET_TRIPLET = "x64-windows-static"
$env:RUSTFLAGS            = "-C target-feature=+crt-static"
$env:OPENSSL_SRC_PERL     = $PerlPath

$buildProfile = if ($Debug) { "debug" } else { "release" }
$cargoArgs = @("build", "--target", "x86_64-pc-windows-msvc")
if (-not $Debug) { $cargoArgs += "--release" }

Write-Host ""
Write-Host "Building eilmeldung ($buildProfile)..."
Write-Host "  vcpkg : $VcpkgRoot"
Write-Host "  perl  : $PerlPath"
Write-Host ""

cargo @cargoArgs

if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$binaryPath = "target\x86_64-pc-windows-msvc\$buildProfile\eilmeldung.exe"
Write-Host ""
Write-Host "Build successful: $binaryPath"
