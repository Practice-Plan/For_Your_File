# uninstall-context-menu.ps1
# PowerShell script to unregister context menu for LNK File Management Center

param()

# Error handling
$ErrorActionPreference = "Stop"

Write-Host "Uninstalling LNK File Management Center context menu..." -ForegroundColor Cyan

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

try {
    # Remove registry entries for files
    Write-Host "Removing file type registration..." -ForegroundColor Yellow
    $fileKey = "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter"
    if (Test-Path $fileKey) {
        Remove-Item -Path $fileKey -Recurse -Force
        Write-Host "  Removed: $fileKey" -ForegroundColor Green
    } else {
        Write-Host "  Not found: $fileKey" -ForegroundColor Gray
    }

    # Remove registry entries for folders
    Write-Host "Removing folder registration..." -ForegroundColor Yellow
    $folderKey = "HKLM:\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter"
    if (Test-Path $folderKey) {
        Remove-Item -Path $folderKey -Recurse -Force
        Write-Host "  Removed: $folderKey" -ForegroundColor Green
    } else {
        Write-Host "  Not found: $folderKey" -ForegroundColor Gray
    }

    # Remove registry entries for directories
    Write-Host "Removing directory registration..." -ForegroundColor Yellow
    $dirKey = "HKLM:\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter"
    if (Test-Path $dirKey) {
        Remove-Item -Path $dirKey -Recurse -Force
        Write-Host "  Removed: $dirKey" -ForegroundColor Green
    } else {
        Write-Host "  Not found: $dirKey" -ForegroundColor Gray
    }

    # Remove registry entries for drives
    Write-Host "Removing drive registration..." -ForegroundColor Yellow
    $driveKey = "HKLM:\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter"
    if (Test-Path $driveKey) {
        Remove-Item -Path $driveKey -Recurse -Force
        Write-Host "  Removed: $driveKey" -ForegroundColor Green
    } else {
        Write-Host "  Not found: $driveKey" -ForegroundColor Gray
    }

    Write-Host "`n✓ Context menu uninstallation completed successfully!" -ForegroundColor Green

} catch {
    Write-Error "Failed to uninstall context menu: $_"
    exit 1
}