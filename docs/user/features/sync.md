# Cloud Synchronization

Cloud synchronization keeps your shortcuts synchronized across multiple devices.

## Overview

![Sync Overview](../images/features/sync-overview.png)

The sync feature provides:
- **Cross-Device Sync**: Access shortcuts from any device
- **Automatic Sync**: Sync changes in real-time
- **Conflict Resolution**: Handle sync conflicts intelligently
- **Sync History**: Track all sync operations

## Setting Up Sync

### Initial Configuration

1. Open Settings > Synchronization
2. Click "Enable Sync"
3. Choose your sync provider:

![Sync Provider Selection](../images/features/sync-provider.png)

### Supported Sync Providers

| Provider | Storage | Features |
|----------|---------|----------|
| **Built-in Cloud** | LNK Cloud | Automatic, seamless sync |
| **Dropbox** | Your Dropbox | Uses existing account |
| **Google Drive** | Your Google Drive | Uses existing account |
| **OneDrive** | Your OneDrive | Uses existing account |
| **Custom Server** | Self-hosted | WebDAV or REST API |

### Configure Sync Settings

![Sync Settings](../images/features/sync-settings.png)

| Setting | Description | Default |
|---------|-------------|---------|
| **Sync Interval** | How often to sync | 5 minutes |
| **Sync on Startup** | Sync when app starts | On |
| **Sync on Exit** | Sync before closing | On |
| **Auto-Sync** | Sync automatically | On |
| **Conflict Resolution** | How to handle conflicts | Ask |

## Sync Operations

### Manual Sync

Force an immediate sync:

- Click the sync icon in the status bar
- Or press `Ctrl + Shift + S`
- Or use File > Sync Now

![Manual Sync](../images/features/sync-manual.png)

### Sync Status

Monitor sync status in the status bar:

| Icon | Status |
|------|--------|
| ✅ | Synced successfully |
| 🔄 | Syncing |
| ❌ | Sync failed |
| ⚠️ | Sync conflict |

### Sync Progress

View sync progress for large operations:

![Sync Progress](../images/features/sync-progress.png)

Shows:
- Current operation
- Progress percentage
- Items remaining
- Estimated time

## Conflict Resolution

### Understanding Conflicts

Conflicts occur when:
- Same entry modified on multiple devices
- Entry deleted on one device, modified on another
- Network issues during sync

### Conflict Dialog

When a conflict occurs:

![Sync Conflict Dialog](../images/features/sync-conflict.png)

Options:
- **Keep Local**: Use local version
- **Keep Remote**: Use remote version
- **Keep Both**: Rename and keep both
- **Merge**: Combine changes
- **Skip**: Ignore this conflict

### Automatic Conflict Resolution

Configure automatic resolution:

| Strategy | Use Case |
|----------|----------|
| **Always Local** | Local changes always win |
| **Always Remote** | Remote changes always win |
| **Most Recent** | Use most recent modification |
| **Ask** | Prompt for each conflict (default) |

## Sync History

View all sync operations:

![Sync History](../images/features/sync-history.png)

### History Details

Each entry shows:
- Timestamp
- Operation type (upload/download)
- Number of items
- Status
- Duration
- Any errors

### Filter History

Filter by:
- Date range
- Operation type
- Status (success/failed/conflict)

## Device Management

### View Connected Devices

See all devices linked to your account:

![Device List](../images/features/sync-devices.png)

Information shown:
- Device name
- Last sync time
- Status (active/offline)
- Number of entries

### Manage Devices

| Action | Description |
|--------|-------------|
| **Rename** | Change device display name |
| **Force Sync** | Trigger sync on device |
| **Remove** | Disconnect device |
| **Block** | Prevent device from syncing |

## Sync Data

### What Gets Synced

| Data Type | Synced |
|-----------|--------|
| Entries | ✅ Yes |
| Groups | ✅ Yes |
| Tags | ✅ Yes |
| Settings | ⚠️ Optional |
| Search History | ❌ No |
| Window Position | ❌ No |
| Local Paths | ⚠️ Path mapping |

### Path Mapping

Local paths are mapped for cross-device compatibility:

![Path Mapping](../images/features/sync-path-mapping.png)

Example:
- Device 1: `C:\Users\Alice\Documents`
- Device 2: `D:\Users\Alice\Docs`

## Offline Mode

### Working Offline

When offline:
- Continue using the application normally
- Changes are queued for sync
- Status bar shows offline indicator

![Offline Indicator](../images/features/sync-offline.png)

### Sync Queue

View pending changes:

![Sync Queue](../images/features/sync-queue.png)

Shows:
- Pending uploads
- Pending downloads
- Conflicts to resolve

## Sync Best Practices

### Initial Setup

1. **Choose Primary Device**: Designate one device as primary
2. **Initial Sync**: Let first sync complete before using other devices
3. **Verify Data**: Check all entries synced correctly

### Daily Usage

1. **Regular Sync**: Let auto-sync handle daily operations
2. **Check Conflicts**: Review and resolve conflicts promptly
3. **Monitor Storage**: Check cloud storage usage

### Backup Strategy

1. **Export Regularly**: Export entries as backup
2. **Version History**: Enable version history if available
3. **Multiple Providers**: Consider using multiple sync providers

## Troubleshooting Sync Issues

### Sync Not Working

- Check internet connection
- Verify sync is enabled
- Check cloud provider status
- Review sync logs

### Slow Sync

- Reduce sync interval
- Check network speed
- Compress sync data
- Use local network sync

### Data Missing After Sync

- Check sync history
- Review conflict resolution
- Restore from backup
- Contact support

### Storage Full

- Clean up old versions
- Delete unused entries
- Upgrade storage plan
- Switch provider

---

See also:
- [Settings](./) - Application settings
- [Troubleshooting](../troubleshooting.md) - Common sync issues
- [FAQ](../faq.md) - Frequently asked questions

*Last updated: 2026*