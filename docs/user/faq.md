# Frequently Asked Questions (FAQ)

This document answers the most common questions about LNK File Management Center.

## Getting Started

### How do I add a new entry?

**Method 1: From the Application**
1. Press `Ctrl + N` or click the "+" button
2. Click "Browse" to select a .lnk file
3. Add tags and notes (optional)
4. Click "Save"

**Method 2: From Windows Explorer**
1. Right-click a .lnk file
2. Select "Add to LNK Management Center"
3. Configure and save

**Method 3: Drag and Drop**
- Drag .lnk files directly into the application window

### How do I search for entries?

1. Press `Alt + Space` to open the search window
2. Type your search query
3. Results appear instantly
4. Use arrow keys to navigate
5. Press Enter to open

**Advanced Search:**
- Use `tag:name` to search by tag
- Use `group:name` to search by group
- Use `type:file/folder/url` to filter by type

### How do I organize my entries with groups?

**Create a Group:**
1. Click "Groups" in the sidebar
2. Click the "+" button
3. Enter group name and choose a color
4. Click "Create"

**Add Entry to Group:**
- Drag the entry to the group
- Or edit entry and select group from dropdown
- Or right-click > Move to Group

## Features

### How do I set an expiration date for an entry?

1. Select the entry
2. Click "Set Expiration" or press `Ctrl + Shift + E`
3. Choose expiration date or relative time
4. Click "Save"

Expiration notifications will appear based on your settings.

### How do I enable cloud synchronization?

1. Open Settings (`Ctrl + ,`)
2. Go to Synchronization
3. Click "Enable Sync"
4. Choose your sync provider
5. Sign in or configure
6. Click "Connect"

Sync will start automatically based on your interval settings.

### How do I change the global hotkey?

1. Open Settings
2. Go to Hotkey section
3. Click "Change Hotkey"
4. Press your new key combination
5. Click "Save"

**Recommended Hotkeys:**
- `Alt + Space` (default)
- `Ctrl + Space`
- `Ctrl + Alt + L`

Avoid conflicts with Windows system shortcuts.

### How do I use the context menu integration?

**Enable Context Menu:**
1. Settings > Integration
2. Click "Install Context Menu"
3. Restart Explorer or reboot

**Use Context Menu:**
1. Right-click any .lnk file in Explorer
2. Select "Add to LNK Management Center"
3. Configure and save

### How do I customize the sorting order?

1. Click the sort dropdown at the top of results
2. Choose your sort method:
   - Relevance (best match)
   - Most Used (frequency)
   - Recently Opened
   - Alphabetical
   - Custom

**Custom Sorting:**
Settings > Sorting > Select "Custom" > Adjust weights

## Configuration

### How do I change the theme?

1. Open Settings
2. Go to Appearance
3. Select theme:
   - Light
   - Dark
   - System (follows Windows)

Changes apply immediately.

### How do I change the language?

1. Settings > Appearance
2. Select Language
3. Choose from:
   - English
   - Chinese (Simplified)

Restart the application for changes to take effect.

### How do I configure notifications?

1. Settings > Notifications
2. Configure:
   - Enable/disable notifications
   - Warning days before expiration
   - Check interval
   - Sound settings

### How do I backup my data?

**Automatic Backup:**
- Enable cloud sync (see above)

**Manual Backup:**
1. Settings > Advanced
2. Click "Export All Entries"
3. Choose location and format
4. Save

**Database Backup:**
Copy `%APPDATA%\wang.station\app\For_Your_File\lnk_management.db`

## Troubleshooting

### The application won't start. What should I do?

1. **Check WebView2**: Install Microsoft Edge WebView2 Runtime
2. **Run as Admin**: Right-click > Run as administrator
3. **Check Antivirus**: Temporarily disable or add exception
4. **Reinstall**: Uninstall and reinstall fresh

See [troubleshooting.md](./troubleshooting.md) for detailed solutions.

### Search is not returning results. Why?

**Possible Causes:**
- No entries added yet
- Search index corrupted
- Filters are active
- Fuzzy search disabled

**Solutions:**
1. Add entries first
2. Settings > Advanced > Rebuild Search Index
3. Clear all filters
4. Enable fuzzy search in settings

### My global hotkey doesn't work. What's wrong?

**Check:**
- Application is running (check system tray)
- Hotkey is enabled in settings
- No other app is using the same hotkey
- Running with necessary permissions

**Fix:**
1. Change to a different hotkey
2. Run as administrator
3. Check for conflicts in Settings > Hotkey

### Context menu items don't appear. How do I fix this?

1. Reinstall shell extension:
   - Settings > Integration
   - Click "Uninstall Context Menu"
   - Click "Install Context Menu"
2. Restart Windows Explorer
3. Reboot your computer

### Sync is not working. What should I check?

**Check:**
- Internet connection is active
- Sync is enabled in settings
- Cloud provider is accessible
- No conflicts pending

**Fix:**
1. Click "Sync Now" manually
2. Check sync history for errors
3. Re-authenticate with provider
4. Contact support if issue persists

## Data Management

### How do I delete multiple entries at once?

1. Select entries using Ctrl+Click
2. Right-click selection
3. Choose "Delete"
4. Confirm deletion

### How do I export my entries?

1. Settings > Advanced
2. Click "Export All Entries"
3. Choose format:
   - JSON (full data)
   - CSV (spreadsheet)
   - HTML (viewable)
4. Select location
5. Save

### How do I import entries?

1. Settings > Advanced
2. Click "Import Entries"
3. Select file (JSON format)
4. Review and confirm
5. Import

### How do I transfer entries to another computer?

**Using Sync:**
1. Enable sync on first computer
2. Enable sync on second computer
3. Sign in with same account
4. Entries sync automatically

**Using Export/Import:**
1. Export entries from first computer
2. Copy export file to second computer
3. Import entries on second computer

## Uninstallation

### How do I uninstall LNK Management Center?

1. Open Settings > Apps
2. Find "LNK File Management Center"
3. Click "Uninstall"
4. Follow the wizard

**Complete Removal:**
1. Uninstall application
2. Delete `%APPDATA%\LNK Management Center`
3. (Optional) Remove registry entries

### Will I lose my data if I uninstall?

**Yes**, unless you:
- Enable cloud sync before uninstalling
- Export your entries manually
- Backup the database file

**To keep data:**
1. Export all entries before uninstalling
2. Or use cloud sync to restore on reinstall

### How do I completely reset the application?

1. Uninstall the application
2. Delete `%APPDATA%\LNK Management Center`
3. Delete registry key: `HKCU\Software\LNK Management Center`
4. Reinstall

⚠️ **Warning**: This removes all data permanently.

## Performance

### Why is the application slow?

**Possible Causes:**
- Large number of entries (10,000+)
- Too many results displayed
- Animations enabled on slow system
- Background sync running

**Solutions:**
1. Reduce max results in search settings
2. Disable animations
3. Reduce sync frequency
4. Clean up old entries

### How many entries can the application handle?

The application can handle **50,000+ entries** efficiently.

**For large databases:**
- Use groups to organize
- Regular cleanup of old entries
- Periodic index rebuild
- Disable unnecessary features

## Security

### Is my data secure?

- Data stored locally with standard Windows permissions
- Cloud sync uses encrypted connections
- No data sent to third parties
- Optional password protection for sync

### Can I password protect the application?

Not directly, but you can:
- Password protect sync account
- Use Windows user account protection
- Encrypt the data folder

### What data is sent to the cloud?

Only what you enable for sync:
- Entry metadata (paths, tags, notes)
- Group information
- Settings (if enabled)
- No file contents
- No personal information

## Advanced

### Can I use the application from the command line?

Yes. Available commands:

```bash
# Open search window
lnk-management-center.exe search

# Add entry
lnk-management-center.exe add "path/to/file.lnk"

# Search for query
lnk-management-center.exe search "query"

# Open settings
lnk-management-center.exe settings
```

### Can I create custom plugins?

Plugin support is planned for future versions. Currently, you can:
- Use the CLI for automation
- Create custom scripts
- Use the API (coming soon)

### Can I access the database directly?

The database is SQLite:
- Location: `%APPDATA%\wang.station\app\For_Your_File\lnk_management.db`
- Use any SQLite tool to access
- Modify at your own risk
- Backup before modifying

### How do I integrate with other applications?

**Deep Links:**
```
lnk-mc://add?path=C:\path\to\file.lnk
lnk-mc://search?query=work
lnk-mc://open?id=123
```

**API (coming soon):**
- REST API for external access
- Webhook support
- Third-party integrations

## Getting Help

### Where can I get more help?

- **Documentation**: This folder
- **Troubleshooting**: [troubleshooting.md](./troubleshooting.md)
- **Issue Tracker**: Report bugs on GitHub
- **Community**: Join our community forum

### How do I report a bug?

1. Check existing issues
2. Collect diagnostics:
   - Settings > Advanced > Export Diagnostics
3. Create a new issue with:
   - Description
   - Steps to reproduce
   - Expected vs actual behavior
   - System information
   - Diagnostic file

### How do I request a feature?

1. Check existing feature requests
2. Create a new feature request with:
   - Use case description
   - Proposed functionality
   - Mockups if applicable
   - Benefits

---

*Last updated: 2026*