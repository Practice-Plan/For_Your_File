# Context Menu Integration

The context menu integration allows you to add shortcuts directly from Windows Explorer.

## Overview

![Context Menu](../images/features/context-menu.png)

Context menu integration provides:
- **Quick Addition**: Add shortcuts without opening the main app
- **Right-Click Access**: Available in Windows Explorer
- **Batch Addition**: Add multiple shortcuts at once
- **System Integration**: Native Windows feel

## Enabling Context Menu

### During Installation

The context menu can be installed during setup:

![Installation Options](../images/features/context-install.png)

Check "Install Shell Extension" to enable.

### Post-Installation

If not installed initially:

1. Open Settings > Integration
2. Click "Install Context Menu"
3. Accept the UAC prompt
4. Restart Windows Explorer or reboot

![Enable Context Menu](../images/features/context-enable.png)

### Verify Installation

1. Open Windows Explorer
2. Right-click any file or folder
3. Look for "LNK Management Center" in the menu

## Using the Context Menu

### Adding Shortcuts

To add a shortcut:

1. Right-click a `.lnk` file in Windows Explorer
2. Select "Add to LNK Management Center"

![Context Menu Add](../images/features/context-menu-add.png)

3. The Add Entry dialog appears:

![Context Menu Dialog](../images/features/context-dialog.png)

4. Configure the entry:
   - Add tags
   - Add notes
   - Select group
   - Set expiration
5. Click "Save"

### Adding Multiple Files

Batch add multiple shortcuts:

1. Select multiple `.lnk` files in Explorer
2. Right-click > "Add to LNK Management Center"
3. All files are added with default settings
4. Edit individually if needed

![Context Menu Batch](../images/features/context-batch.png)

### Quick Add (No Dialog)

For faster addition without configuration:

1. Right-click a `.lnk` file
2. Select "Quick Add to LNK Management Center"
3. Entry is added with default settings
4. Edit later in the main application

## Context Menu Options

### Available Actions

| Menu Item | Description |
|-----------|-------------|
| **Add to LNK Management Center** | Add with configuration dialog |
| **Quick Add** | Add without dialog |
| **Add and Open** | Add and open the main window |
| **Add with Tags** | Add with tag selection |

### Customizing Menu Items

Customize which items appear:

![Context Menu Settings](../images/features/context-settings.png)

| Option | Description |
|--------|-------------|
| **Show Quick Add** | Show/hide quick add option |
| **Show Add and Open** | Show/hide add and open option |
| **Default Action** | Choose default double-click action |
| **Menu Icon** | Show icon in context menu |

## Advanced Features

### Folder Monitoring

Automatically add shortcuts from specific folders:

![Folder Monitoring](../images/features/context-folder-monitor.png)

1. Right-click a folder
2. Select "Monitor for LNK Files"
3. New `.lnk` files are automatically added

### Custom Rules

Create rules for automatic tagging:

![Custom Rules](../images/features/context-rules.png)

Example rules:
- If added from Desktop → Tag as "Desktop"
- If added from Start Menu → Tag as "Start Menu"
- If file name contains "setup" → Tag as "Installer"

### Drag and Drop Integration

Drag `.lnk` files directly to:

- System tray icon
- Application window
- Specific group in sidebar

## Troubleshooting

### Context Menu Not Appearing

**Cause**: Shell extension not installed

**Solution**:
1. Open Settings > Integration
2. Click "Reinstall Context Menu"
3. Restart Explorer or reboot

### Context Menu Slow

**Cause**: Too many entries or large files

**Solution**:
- Reduce menu items in settings
- Use Quick Add instead of full dialog
- Check system performance

### "Access Denied" Error

**Cause**: Insufficient permissions

**Solution**:
- Run the application as administrator
- Reinstall with administrator rights
- Check UAC settings

### Menu Items Grayed Out

**Cause**: Selected files are not `.lnk` files

**Solution**:
- Ensure you're right-clicking `.lnk` files
- Check file extensions are visible
- Verify file type

### Uninstalling Context Menu

To remove the context menu:

1. Open Settings > Integration
2. Click "Uninstall Context Menu"
3. Accept the UAC prompt
4. Restart Explorer

## Context Menu Registry Locations

The shell extension registers at:

```
HKEY_CLASSES_ROOT\lnkfile\shell\LNK Management Center
HKEY_CLASSES_ROOT\*\shell\LNK Management Center
```

For advanced users, you can:
- Modify registry keys directly
- Add custom parameters
- Create additional menu items

## Best Practices

### When to Use Context Menu

Use context menu for:
- **One-off additions**: Quickly add a single shortcut
- **Batch operations**: Add multiple shortcuts at once
- **Folder organization**: Add from specific folders
- **External workflows**: Part of automated processes

### When to Use Main Application

Use the main application for:
- **Bulk editing**: Modify multiple entries
- **Advanced configuration**: Complex entry setup
- **Review and management**: Organize existing entries
- **Search operations**: Find and use shortcuts

### Efficiency Tips

1. **Use Quick Add**: Faster than full dialog for simple additions
2. **Set Defaults**: Configure default tags and groups in settings
3. **Keyboard Shortcuts**: Use keyboard to navigate context menu
4. **Batch Selection**: Select multiple files before right-clicking

---

See also:
- [Groups](./groups.md) - Organize added shortcuts
- [Tags](./) - Tagging system
- [Troubleshooting](../troubleshooting.md) - Context menu issues

*Last updated: 2026*