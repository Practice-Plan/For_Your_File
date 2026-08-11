# Expiration Reminders

Expiration reminders help you track time-sensitive shortcuts and manage outdated entries.

## Overview

![Expiration Overview](../images/features/expiration-overview.png)

The expiration feature provides:
- **Automatic Tracking**: Monitor entry expiration dates
- **Notifications**: Get alerts before shortcuts expire
- **Visual Indicators**: See expiration status at a glance
- **Auto-Cleanup**: Optionally delete expired entries

## Setting Expiration Dates

### Set Expiration for an Entry

1. Select an entry from the list
2. Click the "Set Expiration" button or press `Ctrl + E`
3. Choose expiration options:

![Set Expiration Dialog](../images/features/expiration-set.png)

### Expiration Options

| Option | Description |
|--------|-------------|
| **No Expiration** | Entry never expires |
| **Specific Date** | Choose a specific expiration date |
| **Relative Time** | Set relative expiration (e.g., "30 days") |
| **Recurring** | Repeat expiration after opening |

### Quick Expiration Presets

Use presets for common expiration times:

| Preset | Duration |
|--------|----------|
| Tomorrow | 1 day |
| This Week | 7 days |
| This Month | 30 days |
| This Quarter | 90 days |
| This Year | 365 days |
| Custom | User-defined |

## Expiration Status Indicators

Entries display expiration status visually:

### Status Bar Indicators

![Expiration Status Bar](../images/features/expiration-status-bar.png)

| Status | Icon | Color | Meaning |
|--------|------|-------|---------|
| Active | ✅ | Green | Not expiring soon |
| Expiring Soon | ⚠️ | Yellow | Expiring within warning period |
| Expired | ❌ | Red | Already expired |

### Entry List Indicators

![Expiration in List](../images/features/expiration-list.png)

Each entry shows:
- Remaining days (if expiring soon)
- Expiration date
- Visual highlighting

## Expiration Notifications

### Notification Types

The system sends notifications for:

![Expiration Notifications](../images/features/expiration-notifications.png)

1. **Warning Notification**: Sent before expiration
2. **Expiration Alert**: Sent when entry expires
3. **Batch Summary**: Daily summary of expiring entries

### Configure Notifications

Settings > Expiration > Notifications:

| Setting | Description | Default |
|---------|-------------|---------|
| **Enable Notifications** | Turn notifications on/off | On |
| **Warning Days** | Days before expiration to warn | 7 days |
| **Check Interval** | How often to check (hours) | 1 hour |
| **Sound** | Play notification sound | On |
| **Windows Notifications** | Use Windows notification system | On |

### Notification Actions

When you receive a notification:

![Notification Actions](../images/features/expiration-notification-actions.png)

- **Open**: Open the expiring entry
- **Extend**: Add more time to expiration
- **Dismiss**: Clear the notification
- **Snooze**: Remind again later

## Managing Expired Entries

### View Expired Entries

Access expired entries from the sidebar:

![Expired Entries List](../images/features/expiration-expired-list.png)

The expired entries view shows:
- Entry name and path
- Expiration date
- Days since expired
- Quick actions

### Actions for Expired Entries

| Action | Description |
|--------|-------------|
| **Delete** | Remove the entry permanently |
| **Restore** | Remove expiration and reactivate |
| **Extend** | Add more time to expiration |
| **Open Anyway** | Open the entry despite expiration |
| **Ignore** | Mark as "do not delete" |

### Auto-Delete Expired Entries

Enable automatic deletion:

Settings > Expiration > Auto-Delete:

![Auto-Delete Settings](../images/features/expiration-auto-delete.png)

Options:
- **Enable Auto-Delete**: Automatically remove expired entries
- **Grace Period**: Days to wait after expiration before deletion
- **Protected Entries**: Never auto-delete tagged entries

## Expiration Dashboard

View all expiration information in the dashboard:

![Expiration Dashboard](../images/features/expiration-dashboard.png)

### Dashboard Sections

| Section | Content |
|---------|---------|
| **Summary** | Counts of expired/expiring entries |
| **Timeline** | Upcoming expirations |
| **Recent Expired** | Recently expired entries |
| **Protected** | Entries marked as protected |

## Bulk Expiration Operations

### Set Expiration for Multiple Entries

1. Select multiple entries (Ctrl+Click)
2. Right-click > "Set Expiration"
3. Choose expiration for all selected

### Remove Expiration from Multiple Entries

1. Select entries with expiration
2. Right-click > "Remove Expiration"
3. Confirm the action

## Best Practices

### When to Use Expiration

- **Temporary Projects**: Set expiration when project ends
- **Trial Software**: Match expiration to trial period
- **Time-Sensitive Links**: URLs that expire
- **Event-Related Shortcuts**: Conference, webinar links
- **Work-in-Progress**: Files being actively edited

### Expiration Workflow

1. **Plan**: Decide expiration date when creating entry
2. **Monitor**: Check expiration dashboard weekly
3. **Act**: Extend or delete as needed
4. **Clean**: Run cleanup monthly

### Using Grace Periods

Set appropriate grace periods:
- **Critical**: No grace period
- **Important**: 1-3 day grace period
- **Normal**: 7 day grace period
- **Low Priority**: 14 day grace period

## Troubleshooting

### Notifications Not Working

- Check Windows notification settings
- Verify app has notification permissions
- Check if notifications are enabled in settings
- Verify the check interval is configured

### Wrong Expiration Date

- Check system clock settings
- Verify timezone configuration
- Check if entry was modified
- Re-set the expiration date

### Entry Not Auto-Deleting

- Check if auto-delete is enabled
- Verify grace period hasn't passed
- Check if entry is protected
- Review auto-delete logs

---

See also:
- [Groups](./groups.md) - Organize entries by expiration
- [Search](./search.md) - Find expiring entries
- [Troubleshooting](../troubleshooting.md) - Common issues

*Last updated: 2026*