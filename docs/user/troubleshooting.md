# Troubleshooting Guide

This guide helps you resolve common issues with LNK File Management Center.

## Quick Diagnostics

Before troubleshooting, run these checks:

1. **Application Running**: Verify the app is running (check system tray)
2. **Version**: Ensure you have the latest version
3. **System Resources**: Check CPU and memory usage
4. **Logs**: Review application logs for errors

### Access Application Logs

Logs are located at:
```
%APPDATA%\wang.station\app\For_Your_File\logs\
```

To view logs:
1. Open Settings
2. Go to Advanced > Open Logs Folder
3. Open the most recent log file

## Common Errors and Solutions

### Application Won't Start

#### Symptom
Double-clicking the application icon has no effect, or the application crashes immediately.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Missing WebView2** | Install Microsoft Edge WebView2 Runtime |
| **Corrupted Install** | Reinstall the application |
| **Antivirus Blocking** | Add exception or temporarily disable |
| **Admin Rights Needed** | Run as administrator |

#### Detailed Steps

**1. Install WebView2 Runtime**

Download from Microsoft:
```
https://developer.microsoft.com/en-us/microsoft-edge/webview2/
```

**2. Verify Installation**

Run: `reg query "HKLM\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4B83-B028-D51E5CE4AA5A}"`

**3. Reinstall Application**

1. Uninstall via Settings > Apps
2. Delete `%APPDATA%\wang.station\app\For_Your_File`
3. Download and install fresh

---

### Database Problems

#### Symptom
"Database error" message, or entries not saving.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Corrupted Database** | Repair or reset database |
| **Insufficient Disk Space** | Free up disk space |
| **Permission Issues** | Check folder permissions |
| **Database Locked** | Close other instances |

#### Database Repair

1. Open Settings > Advanced
2. Click "Repair Database"
3. Wait for repair to complete
4. Restart the application

#### Reset Database

⚠️ **Warning**: This will delete all entries.

1. Close the application
2. Delete `%APPDATA%\wang.station\app\For_Your_File\lnk_management.db`
3. Restart the application

---

### Performance Issues

#### Symptom
Application is slow, laggy, or freezes.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Large Database** | Optimize or reduce entries |
| **Too Many Results** | Reduce max results setting |
| **Background Processes** | Disable unnecessary features |
| **System Resources** | Close other applications |

#### Performance Tuning

**1. Reduce Max Results**

Settings > Search > Max Results: Set to 25

**2. Disable Animations**

Settings > Appearance > Animations: Off

**3. Reduce Sync Frequency**

Settings > Sync > Interval: Set to 15 minutes

**4. Clear Search History**

Settings > Search > Clear Search History

---

### Search Not Working

#### Symptom
No results appear when searching, or search is slow.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Index Corrupted** | Rebuild search index |
| **No Entries** | Add entries first |
| **Fuzzy Search Issue** | Disable fuzzy search |
| **Filter Active** | Clear all filters |

#### Rebuild Search Index

1. Settings > Advanced
2. Click "Rebuild Search Index"
3. Wait for indexing to complete
4. Try searching again

---

### LNK File Issues

#### Symptom
Can't open shortcuts, or shortcuts open wrong target.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Target Moved** | Update entry with new path |
| **Target Deleted** | Remove entry or update path |
| **Broken Shortcut** | Repair the .lnk file |
| **Permission Issue** | Check target permissions |

#### Repair Broken Shortcuts

1. Select the entry
2. Click "Edit"
3. Click "Browse" to select new target
4. Save changes

---

### Sync Conflicts

#### Symptom
Sync fails, or data differs between devices.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Network Issue** | Check internet connection |
| **Conflict Unresolved** | Resolve pending conflicts |
| **Storage Full** | Free up cloud storage |
| **Version Mismatch** | Update app on all devices |

#### Resolve Conflicts

1. Open Settings > Sync
2. Click "View Conflicts"
3. Choose resolution for each:
   - Keep Local
   - Keep Remote
   - Keep Both
4. Click "Apply"

---

### Hotkey Conflicts

#### Symptom
Global hotkey doesn't work, or triggers wrong action.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Another App Using It** | Change hotkey |
| **Need Admin Rights** | Run as administrator |
| **Hotkey Disabled** | Enable in settings |
| **Windows Blocking** | Check Windows settings |

#### Change Hotkey

1. Settings > Hotkey
2. Click "Change Hotkey"
3. Press new key combination
4. Click "Save"

#### Check for Conflicts

1. Settings > Hotkey
2. Click "Check Conflicts"
3. Review listed applications
4. Choose a different hotkey if needed

---

### Context Menu Not Working

#### Symptom
Right-click menu items don't appear.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Shell Extension Not Installed** | Install shell extension |
| **Registry Corrupted** | Reinstall shell extension |
| **Need Restart** | Restart Explorer or reboot |
| **Not LNK File** | Ensure file is .lnk |

#### Reinstall Shell Extension

1. Settings > Integration
2. Click "Uninstall Context Menu"
3. Click "Install Context Menu"
4. Restart Explorer

#### Restart Explorer

1. Open Task Manager (Ctrl + Shift + Esc)
2. Find "Windows Explorer"
3. Right-click > Restart

---

### Notifications Not Working

#### Symptom
Expiration notifications don't appear.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **Notifications Disabled** | Enable in settings |
| **Windows Settings** | Check Windows notifications |
| **Focus Assist** | Disable Focus Assist |
| **Do Not Disturb** | Turn off Do Not Disturb |

#### Enable Notifications

1. Settings > Expiration
2. Enable "Show Notifications"
3. Set warning days
4. Check interval

#### Check Windows Settings

1. Open Windows Settings
2. Go to System > Notifications
3. Ensure notifications are enabled
4. Find "LNK Management Center" and enable

---

### Theme Not Applying

#### Symptom
Theme changes don't take effect.

#### Possible Causes and Solutions

| Cause | Solution |
|-------|----------|
| **System Theme** | Switch from System to specific theme |
| **Cache Issue** | Clear application cache |
| **Restart Needed** | Restart the application |
| **Corrupted Settings** | Reset application settings |

#### Reset Theme

1. Settings > Appearance
2. Select desired theme
3. Restart application

---

## Advanced Troubleshooting

### Enable Debug Mode

1. Settings > Advanced
2. Enable "Debug Mode"
3. Restart the application
4. Check logs for detailed information

### Safe Mode

Start in safe mode to bypass issues:

```powershell
lnk-management-center.exe --safe-mode
```

Safe mode disables:
- Shell extension
- Hotkeys
- Background sync
- Plugins

### Factory Reset

Reset all settings to defaults:

1. Settings > Advanced
2. Click "Reset to Defaults"
3. Confirm the action
4. Restart the application

### Check System Requirements

Verify your system meets requirements:

| Requirement | Check |
|-------------|-------|
| **Windows 10+** | Win + R, type `winver` |
| **4 GB RAM** | Task Manager > Performance |
| **100 MB Disk** | File Explorer, check C: drive |
| **WebView2** | Check installed programs |

### Collect Diagnostic Info

To help with support:

1. Settings > Advanced
2. Click "Export Diagnostics"
3. Save the ZIP file
4. Attach to support request

---

## Getting Help

### Support Channels

| Channel | Use For |
|---------|---------|
| **Documentation** | General questions |
| **FAQ** | Common questions |
| **Issue Tracker** | Bug reports |
| **Community Forum** | Discussion and help |

### Before Contacting Support

Prepare the following:

1. **Version Number**: Help > About
2. **Operating System**: Windows version
3. **Error Messages**: Exact text of errors
4. **Steps to Reproduce**: How to cause the issue
5. **Logs**: Export diagnostics

### Report a Bug

When reporting bugs, include:

- Description of the issue
- Steps to reproduce
- Expected behavior
- Actual behavior
- Screenshots if applicable
- Diagnostic export

---

## Prevention Tips

### Regular Maintenance

- **Weekly**: Check for updates
- **Monthly**: Review and clean entries
- **Quarterly**: Rebuild search index
- **Annually**: Archive old entries

### Backup Strategy

- **Daily**: Auto-sync enabled
- **Weekly**: Export entries manually
- **Monthly**: Full database backup

### System Health

- **Keep Updated**: Install updates promptly
- **Disk Space**: Maintain 10% free space
- **Security**: Keep antivirus updated
- **Cleanup**: Remove unused applications

---

*Last updated: 2026*