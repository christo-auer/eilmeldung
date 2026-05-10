# Build eilmeldung on Windows
#
# Prerequisites (run once before using this script):
#   1. Install vcpkg:
#        git clone https://github.com/microsoft/vcpkg C:\vcpkg
#        C:\vcpkg\bootstrap-vcpkg.bat
#   2. Install libxml2 static:
#        C:\vcpkg\vcpkg install libxml2:x64-windows-static
#   3. Install Perl (required to compile OpenSSL from source).
#      Either Strawberry Perl (https://strawberryperl.com) or via scoop:
#        scoop install perl
#
# Usage:
#   .\scripts\build-windows.ps1
#   .\scripts\build-windows.ps1 -PerlPath "C:\custom\perl\bin\perl.exe"
#   .\scripts\build-windows.ps1 -VcpkgRoot "D:\vcpkg"
#   .\scripts\build-windows.ps1 -Debug

param(
    [string]$VcpkgRoot = "C:\vcpkg",
    [string]$PerlPath  = "",
    [switch]$Debug
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# ---------------------------------------------------------------------------
# Resolve Perl
# ---------------------------------------------------------------------------
if (-not $PerlPath) {
    # Try common locations
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
    Write-Error @"
Perl not found. Install it via:
  scoop install perl
or download Strawberry Perl from https://strawberryperl.com
Then re-run this script or pass -PerlPath 'C:\path\to\perl.exe'.
"@
    exit 1
}

# ---------------------------------------------------------------------------
# Verify vcpkg + libxml2
# ---------------------------------------------------------------------------
$vcpkgExe = Join-Path $VcpkgRoot "vcpkg.exe"
if (-not (Test-Path $vcpkgExe)) {
    Write-Error @"
vcpkg not found at '$VcpkgRoot'. Set it up with:
  git clone https://github.com/microsoft/vcpkg C:\vcpkg
  C:\vcpkg\bootstrap-vcpkg.bat
Then re-run this script or pass -VcpkgRoot 'D:\vcpkg'.
"@
    exit 1
}

$libxmlLib = Join-Path $VcpkgRoot "installed\x64-windows-static\lib\xml2.lib"
if (-not (Test-Path $libxmlLib)) {
    Write-Error @"
libxml2 static library not found. Install it with:
  $vcpkgExe install libxml2:x64-windows-static
"@
    exit 1
}

# ---------------------------------------------------------------------------
# Set build environment
# ---------------------------------------------------------------------------
$env:VCPKG_ROOT             = $VcpkgRoot
$env:VCPKGRS_DYNAMIC        = "0"
$env:PKG_CONFIG_PATH        = "$VcpkgRoot\installed\x64-windows-static\lib\pkgconfig"
$env:CMAKE_TOOLCHAIN_FILE   = "$VcpkgRoot\scripts\buildsystems\vcpkg.cmake"
$env:VCPKG_TARGET_TRIPLET   = "x64-windows-static"
$env:RUSTFLAGS              = "-C target-feature=+crt-static"
$env:OPENSSL_SRC_PERL       = $PerlPath

$profile = if ($Debug) { "debug" } else { "release" }
$cargoArgs = @("build", "--target", "x86_64-pc-windows-msvc")
if (-not $Debug) { $cargoArgs += "--release" }

Write-Host "Building eilmeldung ($profile)..."
Write-Host "  vcpkg : $VcpkgRoot"
Write-Host "  perl  : $PerlPath"
Write-Host ""

cargo @cargoArgs

if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

$binaryPath = "target\x86_64-pc-windows-msvc\$profile\eilmeldung.exe"
Write-Host ""
Write-Host "Build successful: $binaryPath"
