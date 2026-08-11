# install-context-menu.ps1
# PowerShell script to register context menu for LNK File Management Center
# This uses a registry-based approach which is simpler and more reliable

param(
    [string]$ExePath
)

# Error handling
$ErrorActionPreference = "Stop"

Write-Host "Installing LNK File Management Center context menu..." -ForegroundColor Cyan

# Determine EXE path if not provided
if (-not $ExePath) {
    # Try to find the executable
    $scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Path

    # Check common locations
    $possiblePaths = @(
        (Join-Path $scriptPath "target\release\LNK File Management Center.exe"),
        (Join-Path $scriptPath "target\debug\LNK File Management Center.exe"),
        (Join-Path $scriptPath "src-tauri\target\release\LNK File Management Center.exe"),
        (Join-Path $scriptPath "src-tauri\target\debug\LNK File Management Center.exe")
    )

    foreach ($path in $possiblePaths) {
        if (Test-Path $path) {
            $ExePath = $path
            break
        }
    }

    if (-not $ExePath) {
        Write-Error "Could not find executable. Please specify the path using -ExePath parameter."
        exit 1
    }
}

Write-Host "Using executable: $ExePath"

# Check if EXE exists
if (-not (Test-Path $ExePath)) {
    Write-Error "Executable not found at: $ExePath"
    exit 1
}

# Check if running as administrator
$isAdmin = ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)

if (-not $isAdmin) {
    Write-Error "This script must be run as Administrator"
    exit 1
}

try {
    # Create registry entries for files (all file types)
    Write-Host "`nRegistering for all file types..." -ForegroundColor Yellow
    $fileKey = "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter"
    New-Item -Path $fileKey -Force | Out-Null
    Set-ItemProperty -Path $fileKey -Name "(Default)" -Value "Add to File Management Center"
    Set-ItemProperty -Path $fileKey -Name "Icon" -Value "`"$ExePath`",0"

    $commandKey = "$fileKey\command"
    New-Item -Path $commandKey -Force | Out-Null
    # Use the --add parameter to pass file path
    Set-ItemProperty -Path $commandKey -Name "(Default)" -Value "`"$ExePath`" --add `"%1`""

    # Create registry entries for folders
    Write-Host "Registering for folders..." -ForegroundColor Yellow
    $folderKey = "HKLM:\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter"
    New-Item -Path $folderKey -Force | Out-Null
    Set-ItemProperty -Path $folderKey -Name "(Default)" -Value "Add to File Management Center"
    Set-ItemProperty -Path $folderKey -Name "Icon" -Value "`"$ExePath`",0"

    $folderCommandKey = "$folderKey\command"
    New-Item -Path $folderCommandKey -Force | Out-Null
    Set-ItemProperty -Path $folderCommandKey -Name "(Default)" -Value "`"$ExePath`" --add `"%1`""

    # Create registry entries for directories
    Write-Host "Registering for directories..." -ForegroundColor Yellow
    $dirKey = "HKLM:\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter"
    New-Item -Path $dirKey -Force | Out-Null
    Set-ItemProperty -Path $dirKey -Name "(Default)" -Value "Add to File Management Center"
    Set-ItemProperty -Path $dirKey -Name "Icon" -Value "`"$ExePath`",0"

    $dirCommandKey = "$dirKey\command"
    New-Item -Path $dirCommandKey -Force | Out-Null
    Set-ItemProperty -Path $dirCommandKey -Name "(Default)" -Value "`"$ExePath`" --add `"%1`""

    # Create registry entries for drives (optional)
    Write-Host "Registering for drives..." -ForegroundColor Yellow
    $driveKey = "HKLM:\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter"
    New-Item -Path $driveKey -Force | Out-Null
    Set-ItemProperty -Path $driveKey -Name "(Default)" -Value "Add to File Management Center"
    Set-ItemProperty -Path $driveKey -Name "Icon" -Value "`"$ExePath`",0"

    $driveCommandKey = "$driveKey\command"
    New-Item -Path $driveCommandKey -Force | Out-Null
    Set-ItemProperty -Path $driveCommandKey -Name "(Default)" -Value "`"$ExePath`" --add `"%1`""

    Write-Host "`n✓ Context menu installation completed successfully!" -ForegroundColor Green
    Write-Host "`nRegistry locations:" -ForegroundColor Cyan
    Write-Host "  Files:     HKLM\SOFTWARE\Classes\*\shell\AddToFileManagementCenter" -ForegroundColor White
    Write-Host "  Folders:   HKLM\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter" -ForegroundColor White
    Write-Host "  Directories: HKLM\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter" -ForegroundColor White
    Write-Host "  Drives:    HKLM\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter" -ForegroundColor White

    Write-Host "`nTo test:" -ForegroundColor Yellow
    Write-Host "  1. Open Windows Explorer" -ForegroundColor White
    Write-Host "  2. Right-click on any file, folder, or drive" -ForegroundColor White
    Write-Host "  3. Select 'Add to File Management Center'" -ForegroundColor White

} catch {
    Write-Error "Failed to install context menu: $_"
    exit 1
}