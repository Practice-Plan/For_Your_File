# Build the For Your File launcher (Go, Windows).
#
# Icon + version resources are regenerated on every invocation from
# ..\src-tauri\icons\icon.ico and the version read from ..\package.json, so
# the launcher stays in lockstep with the main app. go-winres (installed via
# `go install github.com/tc-hib/go-winres@latest`) produces
# rsrc_windows_amd64.syso / rsrc_windows_386.syso that Go links in automatically.
#
# Usage:
#   .\build.ps1                    # 64-bit, writes fyf-launcher.exe here
#   .\build.ps1 -Arch 386          # 32-bit, writes fyf-launcher32.exe here
#   .\build.ps1 -Arch 386 -Name fyf-launcher.exe   # 32-bit under standard name
#   .\build.ps1 -OutDir <dir>      # write the exe into <dir> (e.g. next to the app)
param(
    [string]$OutDir = $PSScriptRoot,
    [ValidateSet('amd64', '386')]
    [string]$Arch = 'amd64',
    [string]$Name = ''
)

$ErrorActionPreference = 'Stop'

# ---------------------------------------------------------------------------
# Resolve shared version and icon
# ---------------------------------------------------------------------------
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
$packageJson = Join-Path $repoRoot 'package.json'
$version = (Get-Content $packageJson | ConvertFrom-Json).version
if (-not $version) { throw "Cannot read version from $packageJson" }

$icon = Resolve-Path (Join-Path $repoRoot 'src-tauri\icons\icon.ico')

# ---------------------------------------------------------------------------
# Ensure go-winres is available
# ---------------------------------------------------------------------------
$goWinres = Get-Command go-winres -ErrorAction SilentlyContinue
if (-not $goWinres) {
    Write-Host 'go-winres not found, installing...'
    Push-Location $env:TEMP
    try {
        go install github.com/tc-hib/go-winres@latest
    } finally {
        Pop-Location
    }
    $goBin = Join-Path (go env GOPATH) 'bin'
    $goWinres = Get-Command (Join-Path $goBin 'go-winres.exe')
}

# ---------------------------------------------------------------------------
# Generate .syso files (both arches so we can switch quickly)
# ---------------------------------------------------------------------------
Push-Location $PSScriptRoot
try {
    & $goWinres.Source simply `
        --icon $icon.Path `
        --manifest gui `
        --product-name 'For Your File Launcher' `
        --file-description 'For Your File Launcher' `
        --product-version $version `
        --file-version $version `
        --original-filename 'fyf-launcher.exe' `
        --arch 'amd64,386' | Out-Null
} finally {
    Pop-Location
}

# ---------------------------------------------------------------------------
# Build the requested architecture
# ---------------------------------------------------------------------------
$env:GOOS = 'windows'
$env:GOARCH = $Arch
$env:CGO_ENABLED = '0'

if (-not $Name) {
    $Name = if ($Arch -eq '386') { 'fyf-launcher32.exe' } else { 'fyf-launcher.exe' }
}

$output = Join-Path $OutDir $Name
Push-Location $PSScriptRoot
try {
    go build -trimpath -ldflags '-s -w -H windowsgui' -o $output .
} finally {
    Pop-Location
}

# Verify the embedded version matches.
$actual = (Get-Item $output).VersionInfo.FileVersion
if ($actual -ne $version) {
    throw "Launcher version mismatch: embedded='$actual', expected='$version'"
}

Write-Host "Built ($Arch, v$version): $output"
