#Requires -Version 5.1
<#
.SYNOPSIS
  Install RepoHop (rhop) from the latest GitHub Release.

.DESCRIPTION
  Downloads rhop.exe into a per-user bin directory and adds it to the user PATH.

  One-liner (PowerShell 5.1 / 7):
    irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex

  Or pin a version:
    $env:REPOPHOP_VERSION = 'v0.1.0'
    irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex

.NOTES
  Does not require admin. Does not delete config/database on reinstall.
#>

$ErrorActionPreference = 'Stop'

$Repo = if ($env:REPOPHOP_REPO) { $env:REPOPHOP_REPO } else { 'superdaobo/repohop' }
$Version = $env:REPOPHOP_VERSION  # e.g. v0.1.0; empty = latest
$InstallRoot = if ($env:REPOPHOP_INSTALL_DIR) {
    $env:REPOPHOP_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA 'RepoHop\bin'
}
$ExeName = 'rhop.exe'
$AssetName = 'rhop-x86_64-pc-windows-msvc.exe'
$ChecksumName = 'SHA256SUMS.txt'

function Write-Info([string]$Message) {
    Write-Host "[repohop] $Message"
}

function Get-ApiHeaders {
    $headers = @{
        'User-Agent' = 'RepoHop-Installer'
        'Accept'     = 'application/vnd.github+json'
    }
    if ($env:GITHUB_TOKEN) {
        $headers['Authorization'] = "Bearer $($env:GITHUB_TOKEN)"
    }
    return $headers
}

function Get-ReleaseJson {
    $headers = Get-ApiHeaders
    if ($Version) {
        $tag = $Version
        if (-not $tag.StartsWith('v')) { $tag = "v$tag" }
        $url = "https://api.github.com/repos/$Repo/releases/tags/$tag"
        Write-Info "Fetching release $tag ..."
    } else {
        $url = "https://api.github.com/repos/$Repo/releases/latest"
        Write-Info "Fetching latest release ..."
    }
    return Invoke-RestMethod -Uri $url -Headers $headers -UseBasicParsing
}

function Get-AssetUrl($release, [string]$name) {
    $asset = $release.assets | Where-Object { $_.name -eq $name } | Select-Object -First 1
    if (-not $asset) {
        throw "Release asset not found: $name (tag $($release.tag_name))"
    }
    return $asset.browser_download_url
}

function Test-Sha256([string]$Path, [string]$Expected) {
    $hash = (Get-FileHash -Path $Path -Algorithm SHA256).Hash.ToLowerInvariant()
    $expected = $Expected.Trim().ToLowerInvariant()
    if ($hash -ne $expected) {
        throw "SHA-256 mismatch for $Path`n  expected: $expected`n  actual:   $hash"
    }
}

function Add-UserPath([string]$Dir) {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $userPath) { $userPath = '' }
    $parts = $userPath -split ';' | Where-Object { $_ -and $_.Trim() -ne '' }
    $normalized = $Dir.TrimEnd('\')
    foreach ($p in $parts) {
        if ($p.TrimEnd('\') -ieq $normalized) {
            Write-Info "PATH already contains $Dir"
            return
        }
    }
    $newPath = if ($userPath.Trim() -eq '') { $Dir } else { "$userPath;$Dir" }
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    # Update current session
    $env:Path = "$Dir;$env:Path"
    Write-Info "Added to user PATH: $Dir"
}

# --- main ---
Write-Info "RepoHop installer"
Write-Info "Install directory: $InstallRoot"

if (-not (Test-Path $InstallRoot)) {
    New-Item -ItemType Directory -Path $InstallRoot -Force | Out-Null
}

$release = Get-ReleaseJson
$tag = $release.tag_name
Write-Info "Using release $tag"

$exeUrl = Get-AssetUrl $release $AssetName
$tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("rhop-install-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null
$tmpExe = Join-Path $tmp $AssetName

try {
    Write-Info "Downloading $AssetName ..."
    Invoke-WebRequest -Uri $exeUrl -OutFile $tmpExe -UseBasicParsing

    # Optional checksum verification
    try {
        $sumUrl = Get-AssetUrl $release $ChecksumName
        $tmpSum = Join-Path $tmp $ChecksumName
        Invoke-WebRequest -Uri $sumUrl -OutFile $tmpSum -UseBasicParsing
        $line = Get-Content $tmpSum | Where-Object { $_ -match [regex]::Escape($AssetName) } | Select-Object -First 1
        if ($line) {
            $expected = ($line -split '\s+')[0]
            Write-Info "Verifying SHA-256 ..."
            Test-Sha256 -Path $tmpExe -Expected $expected
            Write-Info "Checksum OK"
        } else {
            Write-Info "Checksum file present but no line for $AssetName; skipping verify"
        }
    } catch {
        Write-Info "Checksum verification skipped: $($_.Exception.Message)"
    }

    $dest = Join-Path $InstallRoot $ExeName
    Copy-Item -Path $tmpExe -Destination $dest -Force
    Write-Info "Installed: $dest"

    Add-UserPath -Dir $InstallRoot

    Write-Host ""
    Write-Info "Installed RepoHop $tag"
    Write-Info "Run: rhop version"
    Write-Info "If 'rhop' is not found, open a new terminal (PATH refresh)."
    Write-Info "Config:  $env:APPDATA\RepoHop\config.toml"
    Write-Info "Data:    $env:LOCALAPPDATA\RepoHop\repohop.db"
    Write-Host ""

    # Best-effort version check in this session
    $rhop = Join-Path $InstallRoot $ExeName
    if (Test-Path $rhop) {
        & $rhop version
    }
} finally {
    if (Test-Path $tmp) {
        Remove-Item -Recurse -Force $tmp -ErrorAction SilentlyContinue
    }
}
