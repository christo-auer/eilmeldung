# Build eilmeldung on Windows
#
# This script is self-contained: it will automatically install all required
# dependencies (Perl, LLVM, vcpkg, libxml2) if they are not already present.
#
# Usage:
#   .\scripts\build-windows.ps1                  # build local source
#   .\scripts\build-windows.ps1 -Install         # install latest main from GitHub
#   .\scripts\build-windows.ps1 -Debug           # build debug binary
#   .\scripts\build-windows.ps1 -PerlPath "C:\custom\perl\bin\perl.exe"
#   .\scripts\build-windows.ps1 -LlvmBinPath "C:\custom\llvm\bin"
#   .\scripts\build-windows.ps1 -VcpkgRoot "D:\my-vcpkg"

param(
    [string]$VcpkgRoot   = "$env:LOCALAPPDATA\vcpkg",
    [string]$PerlPath    = "",
    [string]$LlvmBinPath = "",
    [switch]$Debug,
    [switch]$Install
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Install-ViaScoop($pkg) {
    if (Get-Command scoop -ErrorAction SilentlyContinue) {
        Write-Host "$pkg not found -- installing via scoop..."
        scoop install $pkg
        return $true
    }
    return $false
}

# ---------------------------------------------------------------------------
# Resolve Perl
# ---------------------------------------------------------------------------
if (-not $PerlPath) {
    $perlSource = (Get-Command perl -ErrorAction SilentlyContinue)?.Source
    $candidates = @(
        "C:\Strawberry\perl\bin\perl.exe",
        "$env:USERPROFILE\scoop\apps\perl\current\perl\bin\perl.exe",
        $(if ($perlSource) { $perlSource } else { $null })
    )
    foreach ($c in $candidates) {
        if ($c -and (Test-Path $c)) { $PerlPath = $c; break }
    }
}

if (-not $PerlPath -or -not (Test-Path $PerlPath)) {
    if (Install-ViaScoop "perl") {
        $found = Get-Command perl -ErrorAction SilentlyContinue
        if ($found) { $PerlPath = $found.Source }
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
Write-Host "  perl  : $PerlPath"

# ---------------------------------------------------------------------------
# Resolve LLVM (required by bindgen to generate libxml2 bindings)
# ---------------------------------------------------------------------------
if (-not $LlvmBinPath) {
    $clangSource = (Get-Command clang -ErrorAction SilentlyContinue)?.Source
    $candidates = @(
        "$env:USERPROFILE\scoop\apps\llvm\current\bin",
        "C:\Program Files\LLVM\bin",
        $(if ($clangSource) { Split-Path $clangSource } else { $null })
    )
    foreach ($c in $candidates) {
        if ($c -and (Test-Path "$c\clang.exe")) { $LlvmBinPath = $c; break }
    }
}

if (-not $LlvmBinPath -or -not (Test-Path "$LlvmBinPath\clang.exe")) {
    if (Install-ViaScoop "llvm") {
        $found = Get-Command clang -ErrorAction SilentlyContinue
        if ($found) { $LlvmBinPath = Split-Path $found.Source }
    }
}

if (-not $LlvmBinPath -or -not (Test-Path "$LlvmBinPath\clang.exe")) {
    Write-Error @"
LLVM/clang not found and could not be installed automatically.
Install it manually via:
  scoop install llvm
Then re-run this script or pass -LlvmBinPath 'C:\path\to\llvm\bin'.
"@
    exit 1
}
Write-Host "  llvm  : $LlvmBinPath"

# ---------------------------------------------------------------------------
# Bootstrap vcpkg if not present
# ---------------------------------------------------------------------------
$vcpkgExe = Join-Path $VcpkgRoot "vcpkg.exe"

if (-not (Test-Path $vcpkgExe)) {
    Write-Host "vcpkg not found at '$VcpkgRoot' -- installing..."

    if (Test-Path $VcpkgRoot) {
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
$libxmlLib = Join-Path $VcpkgRoot "installed\x64-windows-static\lib\libxml2.lib"

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
# Resolve MSVC + Windows SDK include paths for bindgen
# ---------------------------------------------------------------------------
$bindgenIncludes = @()

$vswhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vswhere) {
    $vsPath = & $vswhere -latest -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath 2>$null
    if ($vsPath) {
        $msvcVersionDir = Get-ChildItem "$vsPath\VC\Tools\MSVC" -ErrorAction SilentlyContinue |
            Sort-Object Name | Select-Object -Last 1
        if ($msvcVersionDir) {
            $bindgenIncludes += "$($msvcVersionDir.FullName)\include"
        }
    }
}

$sdkRoot = "${env:ProgramFiles(x86)}\Windows Kits\10\Include"
if (Test-Path $sdkRoot) {
    $sdkVersion = Get-ChildItem $sdkRoot -ErrorAction SilentlyContinue |
        Sort-Object Name | Select-Object -Last 1
    if ($sdkVersion) {
        $bindgenIncludes += "$($sdkVersion.FullName)\ucrt"
        $bindgenIncludes += "$($sdkVersion.FullName)\um"
        $bindgenIncludes += "$($sdkVersion.FullName)\shared"
    }
}

if ($bindgenIncludes.Count -eq 0) {
    Write-Error @"
Could not find MSVC or Windows SDK include paths.
Make sure Visual Studio Build Tools are installed:
  winget install Microsoft.VisualStudio.2022.BuildTools
"@
    exit 1
}

# ---------------------------------------------------------------------------
# Set build environment
# ---------------------------------------------------------------------------
$env:VCPKG_ROOT              = $VcpkgRoot
$env:VCPKGRS_DYNAMIC         = "0"
$env:PKG_CONFIG_PATH         = "$VcpkgRoot\installed\x64-windows-static\lib\pkgconfig"
$env:CMAKE_TOOLCHAIN_FILE    = "$VcpkgRoot\scripts\buildsystems\vcpkg.cmake"
$env:VCPKG_TARGET_TRIPLET    = "x64-windows-static"
$env:RUSTFLAGS               = "-C target-feature=+crt-static"
$env:OPENSSL_SRC_PERL        = $PerlPath
$env:LIBCLANG_PATH           = $LlvmBinPath
$env:BINDGEN_EXTRA_CLANG_ARGS = ($bindgenIncludes | ForEach-Object { "-I`"$_`"" }) -join " "

Write-Host "  vcpkg : $VcpkgRoot"
Write-Host ""

if ($Install) {
    Write-Host "Installing eilmeldung from main branch..."
    cargo install --locked --git https://github.com/christo-auer/eilmeldung --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    Write-Host ""
    Write-Host "Installed successfully. Run 'eilmeldung' to start."
} else {
    $buildProfile = if ($Debug) { "debug" } else { "release" }
    $cargoArgs = @("build", "--target", "x86_64-pc-windows-msvc")
    if (-not $Debug) { $cargoArgs += "--release" }

    Write-Host "Building eilmeldung ($buildProfile)..."
    cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    $binaryPath = "target\x86_64-pc-windows-msvc\$buildProfile\eilmeldung.exe"
    Write-Host ""
    Write-Host "Build successful: $binaryPath"
}
