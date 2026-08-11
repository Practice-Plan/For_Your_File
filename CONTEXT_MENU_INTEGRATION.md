# Windows Context Menu Integration for LNK File Management Center

This document describes how to install, use, and manage the Windows Explorer context menu integration.

## Overview

The context menu integration allows users to right-click on files, folders, or drives in Windows Explorer and select "Add to File Management Center" to quickly add them to the application.

## Architecture

The implementation uses a **registry-based approach** which is:
- ✅ **Simpler** - No COM programming required
- ✅ **More reliable** - Less prone to errors
- ✅ **Easier to debug** - Registry entries are easy to inspect
- ✅ **Better compatibility** - Works on Windows 10 and 11
- ✅ **No code signing required** - No DLL to sign

### Components

1. **Registry Entries**
   - `HKLM\SOFTWARE\Classes\*\shell\AddToFileManagementCenter` - All files
   - `HKLM\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter` - Folders
   - `HKLM\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter` - Directories
   - `HKLM\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter` - Drives

2. **PowerShell Scripts**
   - `install-context-menu.ps1` - Registers context menu
   - `uninstall-context-menu.ps1` - Unregisters context menu

3. **Application Command-Line Interface**
   - `--add <path>` - Add file/folder to application

## Installation

### Prerequisites

- Windows 10 or Windows 11
- PowerShell 5.1 or later
- Administrator privileges (for registration)

### Method 1: Automated Installation (Recommended)

Run the installation script as Administrator:

```powershell
# Navigate to the application directory
cd "k:\Practice Plan\For_Your_File"

# Run the installation script
.\install-context-menu.ps1
```

### Method 2: Manual Installation

1. **Find the executable path:**
   ```powershell
   $exePath = "k:\Practice Plan\For_Your_File\src-tauri\target\release\LNK File Management Center.exe"
   ```

2. **Run the installation script with path:**
   ```powershell
   .\install-context-menu.ps1 -ExePath $exePath
   ```

### Method 3: From the Application

1. Open LNK File Management Center
2. Go to Settings
3. Click "Install Context Menu Integration"
4. Confirm the UAC prompt

## Testing

### Verify Registration

Check if the registry entries exist:

```powershell
# Check for files
Test-Path "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter"

# Check for folders
Test-Path "HKLM:\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter"

# Check for directories
Test-Path "HKLM:\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter"

# Check for drives
Test-Path "HKLM:\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter"
```

### Test the Context Menu

1. Open Windows Explorer (`Win + E`)
2. Navigate to any folder
3. Right-click on a file
4. Verify "Add to File Management Center" appears in the context menu
5. Click the menu item
6. Verify the application opens with the file added

### Test Different Types

Test with different item types:
- ✅ Files (any type)
- ✅ Folders
- ✅ Directories
- ✅ Drives (C:, D:, etc.)

## Uninstallation

### Method 1: Automated Uninstallation

```powershell
# Run as Administrator
.\uninstall-context-menu.ps1
```

### Method 2: From the Application

1. Open LNK File Management Center
2. Go to Settings
3. Click "Uninstall Context Menu Integration"
4. Confirm the UAC prompt

### Method 3: Manual Registry Cleanup

```powershell
# Remove all registry entries
Remove-Item -Path "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter" -Recurse -Force
Remove-Item -Path "HKLM:\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter" -Recurse -Force
Remove-Item -Path "HKLM:\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter" -Recurse -Force
Remove-Item -Path "HKLM:\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter" -Recurse -Force
```

## Troubleshooting

### Context Menu Not Appearing

1. **Check if registered:**
   ```powershell
   Test-Path "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter"
   ```

2. **Verify executable path:**
   ```powershell
   $key = Get-Item "HKLM:\SOFTWARE\Classes\*\shell\AddToFileManagementCenter\command"
   $command = $key.GetValue("")
   Write-Host "Command: $command"
   ```

3. **Restart Windows Explorer:**
   ```powershell
   Stop-Process -Name explorer -Force
   Start-Process explorer
   ```

### Registration Failed

1. **Check Administrator privileges:**
   ```powershell
   ([Security.Principal.WindowsPrincipal] [Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
   ```

2. **Check execution policy:**
   ```powershell
   Get-ExecutionPolicy
   ```

3. **Run with verbose output:**
   ```powershell
   .\install-context-menu.ps1 -ExePath "path\to\exe" -Verbose
   ```

### Application Not Opening

1. **Verify executable exists:**
   ```powershell
   Test-Path "path\to\LNK File Management Center.exe"
   ```

2. **Check command-line parameters:**
   - The application should support `--add <path>` parameter
   - Check application logs for errors

3. **Test manually:**
   ```powershell
   & "path\to\LNK File Management Center.exe" --add "C:\test.txt"
   ```

## Registry Structure

### Files (All Types)
```
HKEY_LOCAL_MACHINE\SOFTWARE\Classes\*\shell\AddToFileManagementCenter
(Default) = "Add to File Management Center"
Icon = "path\to\exe,0"

HKEY_LOCAL_MACHINE\SOFTWARE\Classes\*\shell\AddToFileManagementCenter\command
(Default) = "path\to\exe" --add "%1"
```

### Folders
```
HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter
(Default) = "Add to File Management Center"
Icon = "path\to\exe,0"

HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Folder\shell\AddToFileManagementCenter\command
(Default) = "path\to\exe" --add "%1"
```

### Directories
```
HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter
(Default) = "Add to File Management Center"
Icon = "path\to\exe,0"

HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Directory\shell\AddToFileManagementCenter\command
(Default) = "path\to\exe" --add "%1"
```

### Drives
```
HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter
(Default) = "Add to File Management Center"
Icon = "path\to\exe,0"

HKEY_LOCAL_MACHINE\SOFTWARE\Classes\Drive\shell\AddToFileManagementCenter\command
(Default) = "path\to\exe" --add "%1"
```

## Security Considerations

1. **Administrator Privileges**: Required for installation/uninstallation
2. **Registry Access**: HKLM requires admin rights
3. **User Permissions**: Works with limited user accounts after registration
4. **Performance**: Minimal impact (no background process)
5. **Code Signing**: Not required for this approach

## Advantages of Registry-Based Approach

| Aspect | Registry Approach | COM Shell Extension |
|--------|------------------|---------------------|
| **Complexity** | ✅ Low | ❌ High |
| **Reliability** | ✅ High | ❌ Medium |
| **Debugging** | ✅ Easy | ❌ Difficult |
| **Code Signing** | ✅ Not required | ❌ Required |
| **Performance** | ✅ Good | ✅ Good |
| **Compatibility** | ✅ Windows 10/11 | ✅ Windows 10/11 |
| **Maintenance** | ✅ Easy | ❌ Complex |

## Best Practices

1. **Always test** after installation
2. **Keep installation logs** for troubleshooting
3. **Document the executable path** used during installation
4. **Verify registry entries** after installation
5. **Test with different file types**
6. **Include uninstallation** instructions in documentation

## Frequently Asked Questions

### Q: Do I need to restart Windows Explorer?
A: Sometimes. In most cases, the context menu appears immediately. If not, restart Explorer.

### Q: Does this work on Windows 11?
A: Yes, fully compatible with Windows 11.

### Q: Can users uninstall this without admin rights?
A: No, removing registry entries requires admin privileges.

### Q: Will this slow down Explorer?
A: No, the registry approach has negligible performance impact.

### Q: Can I customize the menu text?
A: Yes, edit the `(Default)` value in the registry.

### Q: Can I add an icon?
A: Yes, set the `Icon` value to `"path\to\exe,0"`.

## Future Improvements

1. Add support for multiple languages
2. Add custom icon for different file types
3. Add sub-menus for additional actions
4. Add keyboard shortcut support
5. Add support for batch operations