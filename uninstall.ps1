#Requires -Version 5.1
<#
.SYNOPSIS
  Uninstall RepoHop (rhop) binary and optional PATH entry.

.DESCRIPTION
  One-liner:
    irm https://raw.githubusercontent.com/superdaobo/repohop/main/uninstall.ps1 | iex

  By default keeps config and database. To also remove data:
    $env:REPOPHOP_PURGE_DATA = '1'
    irm https://raw.githubusercontent.com/superdaobo/repohop/main/uninstall.ps1 | iex
#>

$ErrorActionPreference = 'Stop'

$InstallRoot = if ($env:REPOPHOP_INSTALL_DIR) {
    $env:REPOPHOP_INSTALL_DIR
} else {
    Join-Path $env:LOCALAPPDATA 'RepoHop\bin'
}
$ExePath = Join-Path $InstallRoot 'rhop.exe'
$PurgeData = $env:REPOPHOP_PURGE_DATA -eq '1'

function Write-Info([string]$Message) {
    Write-Host "[repohop] $Message"
}

function Remove-UserPath([string]$Dir) {
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    if (-not $userPath) { return }
    $normalized = $Dir.TrimEnd('\')
    $parts = $userPath -split ';' | Where-Object {
        $_ -and ($_.TrimEnd('\') -ine $normalized)
    }
    $newPath = ($parts -join ';').Trim(';')
    [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
    Write-Info "Removed from user PATH: $Dir"
}

Write-Info "RepoHop uninstaller"

if (Test-Path $ExePath) {
    Remove-Item -Force $ExePath
    Write-Info "Removed $ExePath"
} else {
    Write-Info "Binary not found at $ExePath"
}

if (Test-Path $InstallRoot) {
    $left = Get-ChildItem -Force $InstallRoot -ErrorAction SilentlyContinue
    if (-not $left -or $left.Count -eq 0) {
        Remove-Item -Force $InstallRoot -ErrorAction SilentlyContinue
    }
}

Remove-UserPath -Dir $InstallRoot

if ($PurgeData) {
    $configDir = Join-Path $env:APPDATA 'RepoHop'
    $dataDir = Join-Path $env:LOCALAPPDATA 'RepoHop'
    if (Test-Path $configDir) {
        Remove-Item -Recurse -Force $configDir
        Write-Info "Removed $configDir"
    }
    if (Test-Path $dataDir) {
        Remove-Item -Recurse -Force $dataDir
        Write-Info "Removed $dataDir"
    }
} else {
    Write-Info "Kept config/data (set REPOPHOP_PURGE_DATA=1 to remove)"
}

Write-Info "Done. Open a new terminal so PATH updates apply."
