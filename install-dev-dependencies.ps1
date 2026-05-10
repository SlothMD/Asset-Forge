[CmdletBinding()]
param(
    [string]$DevDrive = "G:",
    [switch]$SkipNpmInstall,
    [switch]$SkipVerification
)

$ErrorActionPreference = "Stop"

function Write-Step {
    param([string]$Message)
    Write-Host ""
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Write-Ok {
    param([string]$Message)
    Write-Host "OK: $Message" -ForegroundColor Green
}

function Write-Warn {
    param([string]$Message)
    Write-Host "WARN: $Message" -ForegroundColor Yellow
}

function Test-Command {
    param([string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Assert-Winget {
    if (-not (Test-Command "winget")) {
        throw @"
winget was not found.

Install "App Installer" from the Microsoft Store, then rerun this script:
https://apps.microsoft.com/detail/9nblggh4nns1
"@
    }
}

function Install-WingetPackage {
    param(
        [Parameter(Mandatory = $true)][string]$Id,
        [Parameter(Mandatory = $true)][string]$Name,
        [string[]]$OverrideArgs = @()
    )

    Write-Step "Checking $Name"
    $listOutput = winget list --id $Id --exact --accept-source-agreements 2>$null
    if ($LASTEXITCODE -eq 0 -and ($listOutput -match [regex]::Escape($Id))) {
        Write-Ok "$Name is already installed"
        return
    }

    Write-Step "Installing $Name"
    $args = @(
        "install",
        "--id", $Id,
        "--exact",
        "--source", "winget",
        "--accept-package-agreements",
        "--accept-source-agreements"
    )

    if ($OverrideArgs.Count -gt 0) {
        $args += $OverrideArgs
    }

    winget @args
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to install $Name with winget package id $Id."
    }
}

function Add-PathIfExists {
    param([string]$PathToAdd)

    if ((Test-Path $PathToAdd) -and (($env:Path -split ";") -notcontains $PathToAdd)) {
        $env:Path = "$PathToAdd;$env:Path"
    }
}

function Set-UserPathEntries {
    param([string[]]$Entries)

    $oldPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathParts = ($oldPath -split ";") | Where-Object {
        $_ -and $_ -notin @(
            "$env:USERPROFILE\.cargo\bin",
            "$env:APPDATA\npm"
        )
    }
    $newPath = (($Entries + $pathParts) | Select-Object -Unique) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    $env:Path = "$(($Entries | Where-Object { $_ }) -join ';');$env:Path"
}

function Copy-RustupProxyIfMissing {
    param(
        [string]$CargoBin,
        [string]$ProxyName
    )

    $rustupExe = Join-Path $CargoBin "rustup.exe"
    $proxyPath = Join-Path $CargoBin $ProxyName

    if ((Test-Path $rustupExe) -and -not (Test-Path $proxyPath)) {
        Copy-Item -LiteralPath $rustupExe -Destination $proxyPath -Force
    }
}

function Ensure-RustupProxies {
    param([string]$CargoBin)

    foreach ($proxy in @(
        "cargo.exe",
        "cargo-clippy.exe",
        "cargo-fmt.exe",
        "cargo-miri.exe",
        "clippy-driver.exe",
        "rls.exe",
        "rust-analyzer.exe",
        "rust-gdb.exe",
        "rust-gdbgui.exe",
        "rust-lldb.exe",
        "rustc.exe",
        "rustdoc.exe",
        "rustfmt.exe"
    )) {
        Copy-RustupProxyIfMissing -CargoBin $CargoBin -ProxyName $proxy
    }
}

function Configure-DevStorage {
    param([string]$DriveRoot)

    if (-not (Test-Path $DriveRoot)) {
        Write-Warn "$DriveRoot is not available. Falling back to standard per-user install paths."
        return
    }

    $devRoot = Join-Path $DriveRoot "DevTools"
    $cacheRoot = Join-Path $DriveRoot "DevCaches"
    $tempRoot = Join-Path $DriveRoot "Temp\User"
    $cargoHome = Join-Path $devRoot "rust\cargo"
    $rustupHome = Join-Path $devRoot "rust\rustup"
    $npmPrefix = Join-Path $devRoot "npm-global"
    $npmCache = Join-Path $cacheRoot "npm-cache"
    $nugetPackages = Join-Path $cacheRoot "nuget-home\packages"
    $pipCache = Join-Path $cacheRoot "pip\Cache"

    New-Item -ItemType Directory -Force `
        $cargoHome, $rustupHome, $npmPrefix, $npmCache, $nugetPackages, $pipCache, $tempRoot |
        Out-Null

    Move-DirectoryContents -Source "$env:USERPROFILE\.cargo" -Destination $cargoHome
    Move-DirectoryContents -Source "$env:USERPROFILE\.rustup" -Destination $rustupHome
    Move-DirectoryContents -Source "$env:APPDATA\npm" -Destination $npmPrefix
    Move-DirectoryContents -Source "$env:LOCALAPPDATA\npm-cache" -Destination $npmCache

    [Environment]::SetEnvironmentVariable("CARGO_HOME", $cargoHome, "User")
    [Environment]::SetEnvironmentVariable("RUSTUP_HOME", $rustupHome, "User")
    [Environment]::SetEnvironmentVariable("TEMP", $tempRoot, "User")
    [Environment]::SetEnvironmentVariable("TMP", $tempRoot, "User")
    [Environment]::SetEnvironmentVariable("NUGET_PACKAGES", $nugetPackages, "User")
    [Environment]::SetEnvironmentVariable("PIP_CACHE_DIR", $pipCache, "User")

    $env:CARGO_HOME = $cargoHome
    $env:RUSTUP_HOME = $rustupHome
    $env:TEMP = $tempRoot
    $env:TMP = $tempRoot
    $env:NUGET_PACKAGES = $nugetPackages
    $env:PIP_CACHE_DIR = $pipCache

    Set-UserPathEntries -Entries @((Join-Path $cargoHome "bin"), $npmPrefix)

    if (Test-Command "npm") {
        npm config set cache $npmCache --location=user
        npm config set prefix $npmPrefix --location=user
    }

    Ensure-RustupProxies -CargoBin (Join-Path $cargoHome "bin")
}

function Move-DirectoryContents {
    param(
        [string]$Source,
        [string]$Destination
    )

    if (-not (Test-Path -LiteralPath $Source -PathType Container)) {
        return
    }

    $sourceItem = Get-Item -LiteralPath $Source
    if ($sourceItem.LinkType -eq "Junction") {
        return
    }

    New-Item -ItemType Directory -Force $Destination | Out-Null
    robocopy $Source $Destination /E /MOVE /COPY:DAT /DCOPY:DAT /XJ /R:2 /W:2 /NFL /NDL /NJH /NJS /NP | Out-Host
    if ($LASTEXITCODE -gt 7) {
        throw "Failed to move $Source to $Destination with robocopy exit code $LASTEXITCODE."
    }

    if (Test-Path -LiteralPath $Source) {
        Remove-Item -LiteralPath $Source -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Refresh-DevPath {
    if ($env:CARGO_HOME) {
        Add-PathIfExists (Join-Path $env:CARGO_HOME "bin")
    }
    Add-PathIfExists "$env:USERPROFILE\.cargo\bin"
    Add-PathIfExists "G:\DevTools\npm-global"
    Add-PathIfExists "$env:ProgramFiles\nodejs"
    Add-PathIfExists "$env:ProgramFiles\Git\cmd"
}

function Ensure-RustStable {
    Refresh-DevPath

    if (-not (Test-Command "rustup")) {
        Write-Warn "rustup is not yet visible in this PowerShell session. Open a new terminal after this script finishes if Rust commands are still unavailable."
        return
    }

    Write-Step "Ensuring stable Rust toolchain"
    rustup default stable
    if ($LASTEXITCODE -ne 0) {
        throw "rustup default stable failed."
    }

    rustup update stable
    if ($LASTEXITCODE -ne 0) {
        throw "rustup update stable failed."
    }
}

function Run-VersionCheck {
    param(
        [string]$Command,
        [string[]]$Args
    )

    if (Test-Command $Command) {
        & $Command @Args
    } else {
        Write-Warn "$Command is not available in this terminal session"
    }
}

$repoRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $repoRoot

Write-Host "Asset Forge Windows development dependency installer" -ForegroundColor White
Write-Host "Repo: $repoRoot"
Write-Host ""
Write-Host "This installs or verifies:"
Write-Host "- Git"
Write-Host "- Node.js LTS and npm"
Write-Host "- Rustup, Rust, and Cargo"
Write-Host "- Microsoft Visual Studio 2022 Build Tools with C++ desktop workload"
Write-Host "- Microsoft Edge WebView2 Runtime"
Write-Host "- G-drive dev cache/tool paths when $DevDrive is available"
Write-Host ""

Assert-Winget
Configure-DevStorage -DriveRoot $DevDrive

Install-WingetPackage -Id "Git.Git" -Name "Git"
Install-WingetPackage -Id "OpenJS.NodeJS.LTS" -Name "Node.js LTS"
Install-WingetPackage -Id "Rustlang.Rustup" -Name "Rustup"
Install-WingetPackage -Id "Microsoft.EdgeWebView2Runtime" -Name "Microsoft Edge WebView2 Runtime"

$vsOverride = @(
    "--override",
    "--wait --quiet --norestart --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
)
Install-WingetPackage -Id "Microsoft.VisualStudio.2022.BuildTools" -Name "Visual Studio 2022 Build Tools" -OverrideArgs $vsOverride

Refresh-DevPath
Ensure-RustStable
if ($env:CARGO_HOME) {
    Ensure-RustupProxies -CargoBin (Join-Path $env:CARGO_HOME "bin")
    Refresh-DevPath
}

if (-not $SkipNpmInstall) {
    if (-not (Test-Command "npm")) {
        throw "npm is not available. Open a new terminal and rerun this script, or verify Node.js installation."
    }

    Write-Step "Installing project npm dependencies"
    npm install
    if ($LASTEXITCODE -ne 0) {
        throw "npm install failed."
    }
} else {
    Write-Warn "Skipping npm install because -SkipNpmInstall was provided"
}

if (-not $SkipVerification) {
    Write-Step "Version checks"
    Run-VersionCheck -Command "git" -Args @("--version")
    Run-VersionCheck -Command "node" -Args @("--version")
    Run-VersionCheck -Command "npm" -Args @("--version")
    Run-VersionCheck -Command "rustc" -Args @("--version")
    Run-VersionCheck -Command "cargo" -Args @("--version")

    if (Test-Command "npm") {
        Write-Step "Verifying frontend build"
        npm run build
        if ($LASTEXITCODE -ne 0) {
            throw "npm run build failed."
        }
    }
} else {
    Write-Warn "Skipping verification because -SkipVerification was provided"
}

Write-Step "Done"
Write-Ok "Asset Forge dependencies are installed or queued by their installers."
Write-Host ""
Write-Host "If Rust or build tools were installed for the first time, open a new PowerShell window before running:"
Write-Host "  npm run tauri:dev"
