# Global Hotkey

The global hotkey feature provides instant access to LNK File Management Center from anywhere in Windows.

## Overview

![Hotkey Demo](../images/features/hotkey-demo.png)

The global hotkey provides:
- **Instant Access**: Open the search window from any application
- **System-Wide**: Works regardless of which app is active
- **Customizable**: Choose your preferred key combination
- **Non-Intrusive**: Doesn't interfere with other applications

## Default Hotkey

By default, the application uses:

| Key Combination | Action |
|-----------------|--------|
| **Alt + Space** | Toggle search window |

This is similar to popular launchers like Spotlight (Mac) or Albert (Linux).

## Using the Global Hotkey

### Opening the Search Window

1. Press `Alt + Space` from anywhere in Windows
2. The search window appears centered on screen
3. Start typing your search query

![Hotkey Activation](../images/features/hotkey-activation.png)

### Closing the Search Window

- Press `Escape`
- Press `Alt + Space` again
- Click outside the window

### While Active

When the search window is open:

- The window is always on top
- Other applications remain visible behind
- Focus is automatically in the search box
- Results update as you type

## Configuring the Hotkey

### Access Hotkey Settings

1. Open Settings (gear icon)
2. Navigate to "Hotkey" section

![Hotkey Settings](../images/features/hotkey-settings.png)

### Change the Hotkey

1. Click the "Change Hotkey" button
2. Press your desired key combination
3. The application validates the combination
4. Click "Save" to apply

![Change Hotkey](../images/features/hotkey-change.png)

### Hotkey Validation

The application checks for:
- Conflicts with Windows system hotkeys
- Conflicts with common application hotkeys
- Invalid key combinations

If a conflict is detected, you'll see a warning:

![Hotkey Conflict Warning](../images/features/hotkey-conflict.png)

### Recommended Hotkeys

| Hotkey | Pros | Cons |
|--------|------|------|
| `Alt + Space` | Default, intuitive | May conflict with some apps |
| `Ctrl + Space` | Easy to reach | Common in other apps |
| `Win + S` | System-like | Reserved by Windows |
| `Ctrl + Alt + L` | Unlikely conflicts | Harder to reach |
| `F12` | Single key | May conflict with dev tools |

## Hotkey Behavior Settings

### Window Behavior

Configure how the window appears:

![Hotkey Behavior](../images/features/hotkey-behavior.png)

| Setting | Options |
|---------|---------|
| **Window Position** | Center, Cursor, Last Position |
| **Animation** | Fade, Slide, None |
| **Opacity** | 100%, 95%, 90% |
| **Always on Top** | On/Off |

### Auto-Hide Behavior

| Trigger | Action |
|---------|--------|
| **Escape** | Hide window |
| **Click Outside** | Hide window |
| **Lose Focus** | Hide window |
| **After Open** | Hide after opening shortcut |

### Startup Behavior

| Option | Description |
|--------|-------------|
| **Start with Windows** | Launch at system startup |
| **Start Minimized** | Start in system tray only |
| **Show on Startup** | Show window on first launch |

## Advanced Hotkey Features

### Multiple Hotkeys

Configure up to 3 different hotkeys for different actions:

![Multiple Hotkeys](../images/features/hotkey-multiple.png)

| Hotkey | Action |
|--------|--------|
| Primary | Toggle search window |
| Secondary | Quick add entry |
| Tertiary | Show expired entries |

### Hotkey with Modifiers

Use modifiers for additional actions:

| Key Combination | Action |
|-----------------|--------|
| `Hotkey` | Toggle window |
| `Hotkey + Shift` | Add new entry |
| `Hotkey + Ctrl` | Show settings |

### Hotkey Scripts

Create custom scripts for hotkey actions:

```javascript
// Example: Custom hotkey action
{
  "hotkey": "Ctrl+Alt+P",
  "action": "search",
  "query": "project:${selected}"
}
```

## Troubleshooting Hotkey Issues

### Hotkey Not Working

Common causes and solutions:

| Issue | Solution |
|-------|----------|
| **Conflict with another app** | Change to different key combination |
| **Need Admin rights** | Run as administrator |
| **Windows blocking** | Check Windows hotkey settings |
| **App not running** | Ensure app is running in tray |

### Checking for Conflicts

![Hotkey Conflict Checker](../images/features/hotkey-conflict-checker.png)

Use the built-in conflict checker:
1. Settings > Hotkey > Check Conflicts
2. The tool scans for potential conflicts
3. Results show conflicting applications

### Re-register Hotkey

If the hotkey stops working:

1. Settings > Hotkey
2. Click "Disable Hotkey"
3. Click "Enable Hotkey"
4. Test the hotkey

### Permissions Issues

Some hotkey combinations require:
- Administrator privileges
- Accessibility permissions (macOS)
- System permissions

Run as administrator if needed.

## Best Practices

### Choosing a Hotkey

1. **Easy to Reach**: Use keys near the home row
2. **Unique**: Avoid conflicts with common shortcuts
3. **Memorable**: Use a logical key combination
4. **Consistent**: Keep the same hotkey across devices

### Avoiding Conflicts

Avoid these common hotkeys:
- `Ctrl + C/V/X/Z` - Clipboard operations
- `Ctrl + S/O/P` - File operations
- `Alt + F4` - Close window
- `Alt + Tab` - Switch windows
- `Win + D/E/L/R` - Windows shortcuts

### Ergonomic Considerations

Choose a comfortable key combination:
- Avoid excessive stretching
- Consider one-handed vs two-handed
- Think about keyboard layout differences

---

See also:
- [Keyboard Shortcuts](../keyboard-shortcuts.md) - Complete shortcut reference
- [Search](./search.md) - Using the search window
- [Troubleshooting](../troubleshooting.md) - Hotkey issues

*Last updated: 2026*