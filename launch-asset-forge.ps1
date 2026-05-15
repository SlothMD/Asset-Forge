[CmdletBinding()]
param(
    [switch]$BrowserOnly,
    [switch]$Dev,
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $repoRoot

$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath) {
    $env:Path = "$userPath;$env:Path"
}

$userCargoHome = [Environment]::GetEnvironmentVariable("CARGO_HOME", "User")
if ($userCargoHome) {
    $env:CARGO_HOME = $userCargoHome
    $cargoBin = Join-Path $userCargoHome "bin"
    if (Test-Path -LiteralPath $cargoBin) {
        $env:Path = "$cargoBin;$env:Path"
    }
}

$repoCargo = "D:\DevTools\rust\rustup\toolchains\stable-x86_64-pc-windows-msvc\bin"
if (Test-Path -LiteralPath (Join-Path $repoCargo "cargo.exe")) {
    $env:Path = "$repoCargo;$env:Path"
}

$msvcToolsRoot = "C:\Program Files\Microsoft Visual Studio\18\Community\VC\Tools\MSVC\14.50.35717"
$msvcBin = Join-Path $msvcToolsRoot "bin\HostX64\x64"
if (Test-Path -LiteralPath (Join-Path $msvcBin "link.exe")) {
    $env:Path = "$msvcBin;$env:Path"
}

$libPaths = @(
    (Join-Path $msvcToolsRoot "lib\onecore\x64"),
    "C:\Program Files\Microsoft Visual Studio\18\Community\SDK\ScopeCppSDK\vc15\VC\lib",
    "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\um\x64",
    "C:\Program Files (x86)\Windows Kits\10\Lib\10.0.19041.0\ucrt\x64"
) | Where-Object { Test-Path -LiteralPath $_ }
if ($libPaths.Count -gt 0) {
    $env:LIB = ($libPaths -join ";")
}

$includePaths = @(
    "C:\Program Files\Microsoft Visual Studio\18\Community\SDK\ScopeCppSDK\vc15\VC\include",
    "C:\Program Files (x86)\Windows Kits\10\include\10.0.19041.0\um",
    "C:\Program Files (x86)\Windows Kits\10\include\10.0.19041.0\shared",
    "C:\Program Files (x86)\Windows Kits\10\include\10.0.19041.0\ucrt"
) | Where-Object { Test-Path -LiteralPath $_ }
if ($includePaths.Count -gt 0) {
    $env:INCLUDE = ($includePaths -join ";")
}

if (-not (Test-Path -LiteralPath (Join-Path $repoRoot "node_modules"))) {
    npm install
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

if ($BrowserOnly) {
    npm run dev
    exit $LASTEXITCODE
}

if ($Dev -or -not $Release) {
    npm run tauri:dev
    exit $LASTEXITCODE
}

npm run tauri:build
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$releaseExe = Join-Path $repoRoot "apps\desktop\src-tauri\target\release\asset-forge-desktop.exe"
if (-not (Test-Path -LiteralPath $releaseExe)) {
    throw "Tauri release executable was not found at $releaseExe"
}

Start-Process -FilePath $releaseExe -WorkingDirectory (Split-Path -Parent $releaseExe)
