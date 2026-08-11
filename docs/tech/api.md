# Backend API Documentation

This document provides comprehensive documentation for all Tauri backend commands (IPC APIs) in LNK File Management Center.

## Overview

The backend exposes functionality to the frontend through Tauri commands. These commands are invoked from the frontend using the `invoke` function from `@tauri-apps/api`.

**Frontend Invocation Example**:
```typescript
import { invoke } from '@tauri-apps/api';

// Invoke a command
const result = await invoke('get_hotkey_config');
```

## API Reference

### Hotkey Management

#### `register_global_hotkey`

Register a global hotkey for window activation.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hotkey` | String | Yes | Hotkey string (e.g., "Alt+Space") |

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('register_global_hotkey', { hotkey: 'Alt+Space' });
```

**Errors**:
- Hotkey registration failed
- Invalid hotkey format
- Hotkey already registered

---

#### `unregister_global_hotkey`

Unregister the current global hotkey.

**Parameters**: None

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('unregister_global_hotkey');
```

---

#### `update_global_hotkey`

Update to a new hotkey combination.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hotkey` | String | Yes | New hotkey string |

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('update_global_hotkey', { hotkey: 'Ctrl+Shift+Space' });
```

---

#### `check_hotkey_conflict`

Check if a hotkey conflicts with system or other applications.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hotkey` | String | Yes | Hotkey to check |

**Returns**: `Result<bool, String>` - `true` if conflict exists

**Example**:
```typescript
const hasConflict = await invoke('check_hotkey_conflict', { hotkey: 'Alt+F4' });
```

---

#### `get_hotkey_config`

Get current hotkey configuration.

**Parameters**: None

**Returns**:
```typescript
interface HotkeyConfig {
  modifiers: string[];  // e.g., ["Alt", "Shift"]
  key: string;          // e.g., "Space"
}
```

**Example**:
```typescript
const config = await invoke('get_hotkey_config');
console.log(config.modifiers, config.key);
```

---

#### `test_hotkey`

Test if a hotkey can be registered.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `hotkey` | String | Yes | Hotkey to test |

**Returns**: `Result<bool, String>` - `true` if available

---

#### `get_suggested_hotkeys`

Get a list of suggested hotkey combinations.

**Parameters**: None

**Returns**: `Vec<String>` - List of suggested hotkeys

**Example**:
```typescript
const suggestions = await invoke('get_suggested_hotkeys');
// ["Alt+Space", "Ctrl+Space", "Alt+Q", ...]
```

---

### Protocol Handler

#### `parse_protocol_url`

Parse a deep link protocol URL.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `url` | String | Yes | Protocol URL (filemgmt://...) |

**Returns**:
```typescript
interface ProtocolRequest {
  action: 'Add' | 'Open' | 'Search' | 'Settings';
  path?: string;
  id?: string;
  query?: string;
}
```

**Example**:
```typescript
const request = await invoke('parse_protocol_url', {
  url: 'filemgmt://add?path=C:\\test.lnk'
});
```

---

#### `handle_protocol_request`

Handle a protocol request from the frontend.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `request` | ProtocolRequest | Yes | Protocol request object |

**Returns**: `Result<(), String>`

**Example**:
```typescript
await invoke('handle_protocol_request', {
  request: {
    action: 'Open',
    id: '123'
  }
});
```

---

### Window Management

#### `show_window`

Show and focus the main window.

**Parameters**: None

**Returns**: `Result<(), String>`

---

#### `hide_window`

Hide the main window.

**Parameters**: None

**Returns**: `Result<(), String>`

---

#### `minimize_to_tray`

Minimize the window to system tray.

**Parameters**: None

**Returns**: `Result<(), String>`

---

### Shell Extension (Windows Only)

#### `register_shell_extension`

Register the Windows Explorer context menu extension.

**Parameters**: None

**Returns**: `Result<String, String>` - Success message

**Note**: Requires administrator privileges

**Example**:
```typescript
try {
  const message = await invoke('register_shell_extension');
  console.log(message);
} catch (error) {
  console.error('Failed to register:', error);
}
```

---

#### `unregister_shell_extension`

Unregister the Windows Explorer context menu extension.

**Parameters**: None

**Returns**: `Result<String, String>`

---

#### `is_shell_extension_registered`

Check if the shell extension is registered.

**Parameters**: None

**Returns**: `bool`

---

### Expiration Management

#### `check_expired_entries`

Get all entries that have expired.

**Parameters**: None

**Returns**: `Result<Vec<Entry>, String>`

**Entry Structure**:
```typescript
interface Entry {
  id?: number;
  lnk_path: string;
  target_path: string;
  target_type: 'File' | 'Folder' | 'Url' | 'Unknown';
  parameters?: string;
  working_dir?: string;
  description?: string;
  icon_location?: string;
  icon_index?: number;
  tags?: string;
  notes?: string;
  frequency: number;
  last_opened?: number;
  created_at: number;
  updated_at: number;
  group_id?: number;
  expires_at?: number;
}
```

---

#### `get_expiring_soon`

Get entries expiring within warning period.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `warning_days` | number | No | Days threshold (default: 7) |

**Returns**: `Result<Vec<[Entry, number]>, String>` - Entry with days remaining

---

#### `set_expiration`

Set expiration date for an entry.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |
| `expires_at` | number | Yes | Unix timestamp |

**Returns**: `Result<(), String>`

**Example**:
```typescript
const expiresAt = Math.floor(Date.now() / 1000) + (30 * 24 * 60 * 60); // 30 days
await invoke('set_expiration', { entryId: 123, expiresAt });
```

---

#### `remove_expiration`

Remove expiration from an entry.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |

**Returns**: `Result<(), String>`

---

#### `extend_expiration`

Extend expiration by N days.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |
| `days` | number | Yes | Days to extend |

**Returns**: `Result<(), String>`

---

#### `get_expiration_status`

Get expiration status for an entry.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry` | Entry | Yes | Entry object |

**Returns**: `Result<ExpirationStatus, String>`

```typescript
type ExpirationStatus =
  | { Expired: { expired_at: number } }
  | { ExpiringSoon: { expires_at: number, days_remaining: number } }
  | { NotExpiring: null };
```

---

#### `get_expiration_counts`

Get counts of expired and expiring entries.

**Parameters**: None

**Returns**:
```typescript
interface ExpirationCounts {
  expired: number;
  expiring_soon: number;
}
```

---

#### `delete_expired_entries`

Delete all expired entries.

**Parameters**: None

**Returns**: `Result<number, String>` - Number of deleted entries

---

#### `get_expiration_config`

Get expiration configuration.

**Parameters**: None

**Returns**:
```typescript
interface ExpirationConfig {
  warning_days: number;
  enable_notifications: boolean;
  auto_delete_expired: boolean;
  check_interval_hours: number;
}
```

---

#### `update_expiration_config`

Update expiration configuration.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `config` | ExpirationConfig | Yes | Configuration object |

**Returns**: `Result<(), String>`

---

#### `show_expiration_notification`

Manually show an expiration notification.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `notification_type` | string | Yes | Type: "expired", "expiring_soon", "batch", "extended" |
| `entry_name` | string | Yes | Entry name |
| `entry_id` | number | Yes | Entry ID |
| `days_remaining` | number | No | Days remaining |

**Returns**: `Result<(), String>`

---

### Cloud Sync

#### `get_sync_status`

Get current sync status.

**Parameters**: None

**Returns**:
```typescript
interface SyncStatusResponse {
  state: string;
  provider?: string;
  sync_path?: string;
  pending_count: number;
  conflict_count: number;
  last_sync?: string;
  last_error?: string;
}
```

---

#### `enable_sync`

Enable cloud synchronization.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `provider` | string | Yes | Provider: "OneDrive" or "Jianguoyun" |
| `sync_path` | string | Yes | Cloud sync path |

**Returns**: `Result<(), String>`

---

#### `disable_sync`

Disable cloud synchronization.

**Parameters**: None

**Returns**: `Result<(), String>`

---

#### `perform_full_sync`

Perform a full bi-directional sync.

**Parameters**: None

**Returns**: `Result<Vec<SyncResultResponse>, String>`

```typescript
interface SyncResultResponse {
  success: boolean;
  operation: string;
  local_path: string;
  cloud_path: string;
  error?: string;
}
```

---

#### `sync_to_cloud`

Upload local changes to cloud.

**Parameters**: None

**Returns**: `Result<Vec<SyncResultResponse>, String>`

---

#### `sync_from_cloud`

Download cloud changes to local.

**Parameters**: None

**Returns**: `Result<Vec<SyncResultResponse>, String>`

---

#### `detect_cloud_provider`

Auto-detect installed cloud providers.

**Parameters**: None

**Returns**: `Option<CloudProviderResponse>`

```typescript
interface CloudProviderResponse {
  provider: string;
  path: string;
}
```

**Example**:
```typescript
const provider = await invoke('detect_cloud_provider');
if (provider) {
  console.log(`Detected ${provider.provider} at ${provider.path}`);
}
```

---

#### `get_sync_history`

Get synchronization history.

**Parameters**: None

**Returns**: `Result<Vec<SyncHistoryResponse>, String>`

```typescript
interface SyncHistoryResponse {
  id: string;
  timestamp: number;
  operation: string;
  file_path: string;
  success: boolean;
  error?: string;
}
```

---

#### `clear_sync_history`

Clear sync history.

**Parameters**: None

**Returns**: `Result<(), String>`

---

#### `get_sync_conflicts`

Get unresolved sync conflicts.

**Parameters**: None

**Returns**: `Result<Vec<SyncConflictResponse>, String>`

```typescript
interface SyncConflictResponse {
  id: string;
  file_path: string;
  local_modified: number;
  cloud_modified: number;
  resolved: boolean;
}
```

---

#### `resolve_sync_conflict`

Resolve a sync conflict.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `conflict_id` | string | Yes | Conflict ID |
| `strategy` | string | Yes | Strategy: "local", "cloud", "merge" |

**Returns**: `Result<(), String>`

---

### Group Management

#### `create_group`

Create a new group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `name` | string | Yes | Group name |
| `color` | string | Yes | Hex color (e.g., "#FF5733") |

**Returns**: `Result<GroupResponse, String>`

```typescript
interface GroupResponse {
  id?: number;
  name: string;
  color: string;
  created_at: number;
  updated_at: number;
}
```

---

#### `list_groups`

List all groups with entry counts.

**Parameters**: None

**Returns**: `Result<Vec<GroupWithCountResponse>, String>`

```typescript
interface GroupWithCountResponse {
  id?: number;
  name: string;
  color: string;
  created_at: number;
  updated_at: number;
  entry_count: number;
}
```

---

#### `get_group`

Get a group by ID.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | number | Yes | Group ID |

**Returns**: `Result<Option<GroupWithCountResponse>, String>`

---

#### `update_group`

Update a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | number | Yes | Group ID |
| `name` | string | Yes | New name |
| `color` | string | Yes | New color |

**Returns**: `Result<GroupResponse, String>`

---

#### `delete_group`

Delete a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `id` | number | Yes | Group ID |

**Returns**: `Result<(), String>`

---

#### `add_entry_to_group`

Add an entry to a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<(), String>`

---

#### `remove_entry_from_group`

Remove an entry from a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<(), String>`

---

#### `get_group_entries`

Get all entries in a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<Vec<Entry>, String>`

---

#### `get_entry_groups`

Get all groups for an entry.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_id` | number | Yes | Entry ID |

**Returns**: `Result<Vec<GroupResponse>, String>`

---

#### `export_group`

Export a group to JSON.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<string, String>` - JSON string

---

#### `import_group`

Import a group from JSON.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `group_json` | string | Yes | JSON string |

**Returns**: `Result<GroupResponse, String>`

---

#### `batch_add_to_group`

Add multiple entries to a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_ids` | number[] | Yes | Array of entry IDs |
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<(), String>`

---

#### `batch_remove_from_group`

Remove multiple entries from a group.

**Parameters**:
| Name | Type | Required | Description |
|------|------|----------|-------------|
| `entry_ids` | number[] | Yes | Array of entry IDs |
| `group_id` | number | Yes | Group ID |

**Returns**: `Result<(), String>`

---

### CLI Support

#### `get_cli_args`

Get CLI arguments passed at startup.

**Parameters**: None

**Returns**:
```typescript
interface CliArgs {
  version: boolean;
  help: boolean;
  minimized: boolean;
  add?: string;
  open?: string;
  search?: string;
  deep_link?: string;
  files: string[];
}
```

---

## Error Handling

All commands return a `Result` type. Errors are returned as strings.

**Frontend Error Handling**:
```typescript
try {
  const result = await invoke('some_command', params);
  // Handle success
} catch (error) {
  console.error('Command failed:', error);
  // Handle error
}
```

**Common Errors**:
- Database connection errors
- Invalid parameters
- Permission denied
- Resource not found
- Operation timeout

## Authentication

No authentication required for local operations. Cloud sync may require OAuth tokens depending on provider.

## Rate Limiting

No rate limiting for local operations. Cloud sync operations may be limited by provider APIs.

## Events

The backend emits events to the frontend using Tauri's event system.

### `hotkey-pressed`

Emitted when the global hotkey is pressed.

```typescript
import { listen } from '@tauri-apps/api/event';

const unlisten = await listen('hotkey-pressed', (event) => {
  console.log('Hotkey pressed!');
});
```

### `protocol-request`

Emitted when a protocol request is received.

```typescript
const unlisten = await listen<ProtocolRequest>('protocol-request', (event) => {
  const request = event.payload;
  console.log('Protocol request:', request.action);
});
```

### `deep-link://filemgmt`

Emitted when a deep link is opened.

```typescript
const unlisten = await listen('deep-link://filemgmt', (event) => {
  const url = event.payload;
  console.log('Deep link:', url);
});
```

## TypeScript Type Definitions

Complete TypeScript definitions are available in the frontend codebase:

```typescript
// types/api.ts
export interface Entry { ... }
export interface Group { ... }
export interface SyncStatus { ... }
// ... etc
```

## Examples

### Complete Workflow Example

```typescript
import { invoke } from '@tauri-apps/api';

// Create a group
const group = await invoke<GroupResponse>('create_group', {
  name: 'Work Apps',
  color: '#3498DB'
});

// Add an entry to the group
await invoke('add_entry_to_group', {
  entryId: 123,
  groupId: group.id
});

// Set expiration
const expiresAt = Math.floor(Date.now() / 1000) + (30 * 24 * 60 * 60);
await invoke('set_expiration', {
  entryId: 123,
  expiresAt
});

// Get expiration status
const status = await invoke<ExpirationStatus>('get_expiration_status', {
  entry: entryObject
});

// Sync to cloud
if (await invoke('detect_cloud_provider')) {
  const results = await invoke<SyncResultResponse[]>('sync_to_cloud');
  console.log(`Synced ${results.length} files`);
}
```

## Next Steps

- See [Database Schema](./database-schema.md) for data structures
- See [Architecture](./architecture.md) for system design
- See [Build & Deploy](./build-deploy.md) for deployment