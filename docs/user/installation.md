# Installation Guide

This guide provides step-by-step instructions for installing LNK File Management Center on Windows.

## System Requirements

Before installing, ensure your system meets the following requirements:

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| Operating System | Windows 10 | Windows 10/11 (latest) |
| RAM | 4 GB | 8 GB or more |
| Disk Space | 100 MB | 500 MB |
| Display | 1024x768 | 1920x1080 or higher |

### Additional Requirements

- **Microsoft Edge WebView2 Runtime**: Required for the application to run. Usually pre-installed on Windows 11, but may need to be installed on Windows 10.
- **Administrator Rights**: Required for installation and shell extension setup

## Installation Steps

### Step 1: Download the Installer

1. Download the latest installer from the official release page
2. Choose the appropriate version:
   - `lnk-management-center-setup-x64.exe` for 64-bit systems
   - `lnk-management-center-setup-x86.exe` for 32-bit systems

### Step 2: Run the Installer

1. Locate the downloaded installer file
2. Right-click and select "Run as administrator"
3. If prompted by User Account Control, click "Yes"

### Step 3: Installation Wizard

The installation wizard will guide you through the setup process:

#### Welcome Screen
![Installation Welcome](./images/installation/welcome.png)

Click "Next" to continue.

#### License Agreement
![License Agreement](./images/installation/license.png)

1. Read the license agreement carefully
2. Select "I accept the agreement"
3. Click "Next"

#### Select Installation Location
![Installation Location](./images/installation/location.png)

- Default: `C:\Program Files\LNK Management Center`
- Click "Browse" to change if needed
- Click "Next"

#### Select Components
![Select Components](./images/installation/components.png)

Choose which components to install:

| Component | Description | Required |
|-----------|-------------|----------|
| Application Files | Main program files | Yes |
| Shell Extension | Right-click menu integration | Recommended |
| Start Menu Shortcut | Quick access from Start Menu | Optional |
| Desktop Shortcut | Icon on desktop | Optional |

#### Start Menu Folder
![Start Menu](./images/installation/start-menu.png)

- Select an existing folder or create a new one
- Click "Next"

#### Additional Tasks
![Additional Tasks](./images/installation/tasks.png)

Select additional tasks:

- [ ] Create a desktop shortcut
- [ ] Create a Quick Launch shortcut
- [ ] Launch application when Windows starts
- [ ] Register file associations

Click "Next" after making your selections.

#### Ready to Install
![Ready to Install](./images/installation/ready.png)

Review your settings and click "Install" to begin.

#### Installation Progress
![Installation Progress](./images/installation/progress.png)

Wait for the installation to complete.

#### Installation Complete
![Installation Complete](./images/installation/complete.png)

- [ ] Launch LNK File Management Center
- Click "Finish"

## Post-Installation Setup

### First Launch

When you first launch the application, you'll see the setup wizard:

#### 1. Welcome Screen
![First Launch Welcome](./images/installation/first-launch.png)

Click "Get Started" to begin.

#### 2. Choose Theme
![Theme Selection](./images/installation/theme.png)

Select your preferred theme:
- **Light**: Bright theme for daytime use
- **Dark**: Dark theme for low-light environments
- **System**: Follow Windows system theme

#### 3. Configure Global Hotkey
![Hotkey Configuration](./images/installation/hotkey.png)

Set your global hotkey for quick access:
- Default: `Alt + Space`
- Click "Change" to customize
- Ensure the key combination doesn't conflict with other applications

#### 4. Import Existing Shortcuts
![Import Shortcuts](./images/installation/import.png)

Optionally scan for existing .lnk files:
- Desktop shortcuts
- Start Menu shortcuts
- Custom folders

Click "Scan" to search, or "Skip" to manually add entries later.

#### 5. Setup Complete
![Setup Complete](./images/installation/setup-complete.png)

You're ready to use LNK File Management Center!

### Shell Extension Setup

The shell extension adds "Add to LNK Management Center" to the right-click context menu in Windows Explorer:

1. After installation, restart Windows Explorer or reboot your computer
2. Right-click any .lnk file to see the new context menu option

![Context Menu](./images/features/context-menu.png)

## Silent Installation (Advanced)

For enterprise deployment, use silent installation:

```powershell
# Basic silent install
lnk-management-center-setup.exe /SILENT

# Silent install with specific options
lnk-management-center-setup.exe /VERYSILENT /SUPPRESSMSGBOXES /NORESTART /DIR="C:\Custom\Path"
```

### Silent Install Parameters

| Parameter | Description |
|-----------|-------------|
| `/SILENT` | Silent installation with progress bar |
| `/VERYSILENT` | Silent installation without progress bar |
| `/SUPPRESSMSGBOXES` | Suppress message boxes |
| `/NORESTART` | Don't restart after installation |
| `/DIR="path"` | Specify installation directory |
| `/COMPONENTS="list"` | Select components to install |

## Uninstallation

### Standard Uninstall

1. Open Settings > Apps > Installed Apps
2. Find "LNK File Management Center"
3. Click "Uninstall"
4. Follow the uninstall wizard

### Complete Removal

To completely remove all application data:

1. Uninstall the application
2. Delete the data folder:
   ```
   %APPDATA%\wang.station\app\For_Your_File
   ```
3. Remove registry entries (optional):
   ```
   HKEY_CURRENT_USER\Software\LNK Management Center
   ```

## Troubleshooting Installation Issues

### Installer Won't Run

- Ensure you have administrator rights
- Temporarily disable antivirus software
- Check if the installer file is not corrupted (verify checksum)

### Shell Extension Not Working

- Restart Windows Explorer: Task Manager > Right-click "Windows Explorer" > Restart
- Reinstall the application with shell extension selected

### Application Won't Start

- Install Microsoft Edge WebView2 Runtime
- Check Windows Event Viewer for error details
- Try running as administrator

### Hotkey Not Working

- Check for conflicts with other applications
- Restart the application
- Try a different key combination

For more issues, see [troubleshooting.md](./troubleshooting.md).

---

*Last updated: 2026*