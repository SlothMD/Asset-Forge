[CmdletBinding()]
param(
    [switch]$BrowserOnly,
    [switch]$Dev
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

if ($BrowserOnly) {
    npm run dev
    exit $LASTEXITCODE
}

if ($Dev) {
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
