# Project Groups

Groups allow you to organize your shortcuts into logical collections for better management and faster access.

## Overview

![Groups Interface](../images/features/groups-interface.png)

Groups provide:
- **Organization**: Keep related shortcuts together
- **Quick Access**: Filter search by group
- **Visual Identification**: Color-coded groups
- **Bulk Operations**: Manage multiple entries at once

## Creating Groups

### Create a New Group

1. Click the "Groups" icon in the sidebar
2. Click the "+" button or press `Ctrl + G`
3. Enter group details:

![Create Group Dialog](../images/features/groups-create.png)

#### Group Properties

| Property | Description |
|----------|-------------|
| **Name** | Group name (required) |
| **Color** | Choose a color for visual identification |
| **Description** | Optional description |
| **Icon** | Choose an icon (optional) |

### Group Colors

Choose from preset colors or create custom colors:

![Group Colors](../images/features/groups-colors.png)

Available preset colors:
- 🔴 Red - Important/Urgent
- 🟠 Orange - Work
- 🟡 Yellow - Personal
- 🟢 Green - Active/In Progress
- 🔵 Blue - Information
- 🟣 Purple - Projects
- ⚫ Gray - Archive
- ⚪ White - Miscellaneous

## Managing Groups

### View Groups

Access groups from the sidebar:

![Groups List](../images/features/groups-list.png)

The sidebar shows:
- Group name and color
- Entry count
- Quick actions menu

### Edit a Group

1. Right-click the group in the sidebar
2. Select "Edit Group"
3. Modify the properties
4. Click "Save"

![Edit Group](../images/features/groups-edit.png)

### Delete a Group

1. Right-click the group
2. Select "Delete Group"
3. Choose what to do with entries:
   - Move to "Ungrouped"
   - Delete all entries
   - Cancel

⚠️ **Warning**: Deleting a group with entries will prompt for confirmation.

## Adding Entries to Groups

### Method 1: During Entry Creation

When adding a new shortcut, select the group from the dropdown:

![Select Group During Creation](../images/features/groups-add-entry.png)

### Method 2: Drag and Drop

Drag entries from the main list to a group in the sidebar:

![Drag to Group](../images/features/groups-drag-drop.png)

### Method 3: Right-Click Menu

1. Right-click an entry
2. Select "Move to Group"
3. Choose the target group

### Method 4: Bulk Assignment

1. Select multiple entries (Ctrl+Click)
2. Right-click > "Move to Group"
3. Select the group

## Viewing Group Contents

### Open Group View

Click on a group to view its entries:

![Group View](../images/features/groups-view.png)

The group view shows:
- Group header with name and description
- Entry count
- All entries in the group
- Quick search within group

### Group Statistics

View group statistics in the group header:

| Statistic | Description |
|-----------|-------------|
| **Total Entries** | Number of shortcuts |
| **Active** | Entries not expired |
| **Expired** | Entries past expiration |
| **Most Used** | Most frequently opened entry |

## Group Operations

### Export Group

Export a group to share with others or backup:

1. Right-click the group
2. Select "Export Group"
3. Choose export format:
   - JSON (for importing)
   - CSV (for spreadsheets)
   - HTML (for viewing)

![Export Group](../images/features/groups-export.png)

### Import Group

Import a previously exported group:

1. Click the "+" button in Groups
2. Select "Import Group"
3. Choose the file to import
4. Review and confirm

### Merge Groups

Combine two groups into one:

1. Right-click a group
2. Select "Merge Into..."
3. Choose the target group
4. Confirm the merge

## Group Settings

### Default Group

Set a default group for new entries:

1. Settings > Groups
2. Select "Default Group"
3. Choose from existing groups

### Group Shortcuts

Assign keyboard shortcuts to groups:

| Group | Shortcut |
|-------|----------|
| Work | `Ctrl + 1` |
| Personal | `Ctrl + 2` |
| Projects | `Ctrl + 3` |

### Auto-Grouping Rules

Create rules to automatically group new entries:

![Auto-Grouping Rules](../images/features/groups-auto-rules.png)

Example rules:
- If path contains "work" → Move to "Work" group
- If tag is "urgent" → Move to "Important" group
- If type is URL → Move to "Bookmarks" group

## Best Practices

### Group Organization Strategies

1. **By Project**: Create groups for each project
2. **By Category**: Personal, Work, Entertainment
3. **By Priority**: Urgent, Normal, Low Priority
4. **By Time**: Daily, Weekly, Monthly tasks

### Naming Conventions

Use consistent naming:
- Start with emoji for visual identification: `📁 Documents`
- Use prefixes for sorting: `01. Work`, `02. Personal`
- Keep names short and descriptive

### Group Maintenance

Regular maintenance tips:
- Review groups monthly
- Merge similar groups
- Archive unused groups
- Keep entry count balanced

## Troubleshooting

### Group Not Appearing

- Refresh the sidebar (F5)
- Check if the group was deleted
- Verify database integrity

### Cannot Add Entry to Group

- Ensure the group exists
- Check if entry is already in another group
- Verify no database lock

### Group Color Not Showing

- Check if color is properly saved
- Try changing and re-saving the color
- Restart the application

---

See also:
- [Search](./search.md) - Find entries within groups
- [Expiration](./expiration.md) - Set expirations for group entries
- [FAQ](../faq.md) - Common questions

*Last updated: 2026*