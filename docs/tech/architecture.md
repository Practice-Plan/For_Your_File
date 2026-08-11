# System Architecture

This document provides a detailed overview of LNK File Management Center's system architecture, components, and design decisions.

## High-Level Architecture

```mermaid
graph TB
    subgraph "User Interface Layer"
        UI[React UI]
        Components[UI Components]
        State[State Management]
    end

    subgraph "IPC Layer"
        Invoke[Tauri Invoke]
        Events[Event System]
    end

    subgraph "Backend Services Layer"
        Hotkey[Hotkey Manager]
        Protocol[Protocol Handler]
        Expiration[Expiration Manager]
        Sync[Sync Manager]
        Groups[Group Manager]
        Shell[Shell Extension]
    end

    subgraph "Data Access Layer"
        DB[Database Layer]
        FileIO[File I/O]
        Registry[Windows Registry]
    end

    subgraph "Data Storage Layer"
        SQLite[(SQLite DB)]
        FileSystem[File System]
        Cloud[(Cloud Storage)]
    end

    UI --> Components
    Components --> State
    State --> Invoke
    Invoke --> Hotkey
    Invoke --> Protocol
    Invoke --> Expiration
    Invoke --> Sync
    Invoke --> Groups
    Invoke --> Shell

    Events --> State

    Hotkey --> Registry
    Protocol --> FileIO
    Expiration --> DB
    Sync --> FileIO
    Groups --> DB
    Shell --> Registry

    DB --> SQLite
    FileIO --> FileSystem
    FileIO --> Cloud
```

## Component Architecture

### Frontend Architecture

```mermaid
graph LR
    subgraph "React Application"
        App[App.tsx]
        Router[Router]
        Layout[Layout]
    end

    subgraph "Pages"
        EntryList[Entry List]
        EntryDetail[Entry Detail]
        GroupList[Group List]
        Settings[Settings]
        Search[Search]
    end

    subgraph "Components"
        Sidebar[Sidebar]
        Header[Header]
        EntryCard[Entry Card]
        GroupBadge[Group Badge]
        Modal[Modal]
        Notification[Notification]
    end

    subgraph "Hooks"
        UseEntries[useEntries]
        UseGroups[useGroups]
        UseHotkey[useHotkey]
        UseSync[useSync]
    end

    subgraph "Utils"
        Api[API Client]
        Helpers[Helper Functions]
        Types[Type Definitions]
    end

    App --> Router
    Router --> Layout
    Layout --> EntryList
    Layout --> EntryDetail
    Layout --> GroupList
    Layout --> Settings
    Layout --> Search

    EntryList --> EntryCard
    GroupList --> GroupBadge

    UseEntries --> Api
    UseGroups --> Api
    UseHotkey --> Api
    UseSync --> Api

    Api --> Types
```

### Backend Architecture

```mermaid
graph TB
    subgraph "Tauri Application"
        Main[main.rs]
        Lib[lib.rs]
    end

    subgraph "Core Modules"
        HotkeyMod[hotkey.rs]
        ProtocolMod[protocol.rs]
        ExpirationMod[expiration/]
        SyncMod[sync/]
        GroupMod[groups/]
        CLI[cli.rs]
        Models[models.rs]
        Commands[commands.rs]
        Notifications[notifications/]
    end

    subgraph "External Dependencies"
        Windows[Windows API]
        SQLiteLib[SQLite]
        FileSystem[File System]
    end

    Main --> Lib
    Lib --> HotkeyMod
    Lib --> ProtocolMod
    Lib --> ExpirationMod
    Lib --> SyncMod
    Lib --> GroupMod
    Lib --> CLI
    Lib --> Notifications

    Commands --> Models
    ExpirationMod --> Models
    GroupMod --> Models

    HotkeyMod --> Windows
    ExpirationMod --> SQLiteLib
    GroupMod --> SQLiteLib
    ProtocolMod --> FileSystem
```

## Data Flow Diagrams

### Entry Creation Flow

```mermaid
sequenceDiagram
    participant User
    participant UI as React UI
    participant Backend as Tauri Backend
    participant DB as SQLite Database
    participant FTS as FTS5 Index

    User->>UI: Add LNK file
    UI->>Backend: invoke('add_entry', path)
    Backend->>DB: INSERT INTO entries
    DB-->>Backend: entry_id
    Backend->>FTS: INSERT INTO entries_fts
    FTS-->>Backend: success
    Backend-->>UI: Entry object
    UI-->>User: Show new entry
```

### Global Hotkey Flow

```mermaid
sequenceDiagram
    participant User
    participant OS as Windows OS
    participant Hotkey as Hotkey Manager
    participant Backend as Tauri Backend
    participant UI as React UI

    User->>Hotkey: Register hotkey
    Hotkey->>OS: RegisterHotKey API
    OS-->>Hotkey: Success
    Hotkey->>Backend: Start listener thread
    Backend->>UI: Ready

    User->>OS: Press hotkey
    OS->>Hotkey: WM_HOTKEY message
    Hotkey->>Backend: emit('hotkey-pressed')
    Backend->>UI: Event received
    UI->>UI: Toggle window
```

### Deep Link Protocol Flow

```mermaid
sequenceDiagram
    participant User
    participant Browser
    participant OS as Windows OS
    participant Backend as Tauri Backend
    participant UI as React UI

    User->>Browser: Click filemgmt:// link
    Browser->>OS: Open URL
    OS->>Backend: Launch app with URL
    Backend->>Backend: parse_deep_link()
    Backend->>UI: emit('protocol-request')
    UI->>UI: Handle action (Add/Open/Search)
    UI-->>User: Show result
```

### Expiration Check Flow

```mermaid
sequenceDiagram
    participant Timer as System Timer
    participant Backend as Tauri Backend
    participant DB as SQLite Database
    participant Notif as Notification Service
    participant User

    Timer->>Backend: Check (every hour)
    Backend->>DB: SELECT expired entries
    DB-->>Backend: Expired entries
    Backend->>DB: SELECT expiring soon
    DB-->>Backend: Expiring entries

    alt Has expired
        Backend->>Notif: Show expired notification
        Notif->>User: Desktop notification
    end

    alt Has expiring soon
        Backend->>Notif: Show warning notification
        Notif->>User: Desktop notification
    end
```

### Cloud Sync Flow

```mermaid
sequenceDiagram
    participant User
    participant UI as React UI
    participant Backend as Tauri Backend
    participant LocalFS as Local FileSystem
    participant CloudFS as Cloud Storage

    User->>UI: Click "Sync to Cloud"
    UI->>Backend: invoke('sync_to_cloud')
    Backend->>LocalFS: Read database file
    LocalFS-->>Backend: Database data
    Backend->>Backend: Calculate checksums
    Backend->>CloudFS: Upload changes
    CloudFS-->>Backend: Upload results

    alt Conflicts detected
        Backend->>UI: Show conflicts
        UI->>User: Choose resolution
        User->>UI: Resolve strategy
        UI->>Backend: invoke('resolve_conflict')
        Backend->>CloudFS: Apply resolution
    end

    Backend-->>UI: Sync results
    UI-->>User: Show sync status
```

## Component Design

### Hotkey Manager

**Purpose**: Manage global Windows hotkeys for window activation.

**Design**:
- Single-threaded listener running in separate thread
- Mutex-protected state shared with main thread
- Windows API integration for hotkey registration
- Configuration persistence in registry

**Key Features**:
- Hotkey conflict detection
- Dynamic reconfiguration
- Thread-safe event emission

**Implementation**:
```rust
pub struct HotkeyManager {
    hotkey_id: Option<i32>,
    current_hotkey: Option<HotkeyConfig>,
    receiver: Option<Receiver<HotkeyEvent>>,
}

impl HotkeyManager {
    pub fn register(&mut self, modifiers: &[String], key: &str) -> Result<()>;
    pub fn unregister(&mut self) -> Result<()>;
    pub fn check_conflict(&self, modifiers: &[String], key: &str) -> Result<bool>;
    pub fn start_listener(&mut self, app_handle: AppHandle) -> Result<()>;
}
```

### Protocol Handler

**Purpose**: Handle deep link protocol (filemgmt://) for system integration.

**Design**:
- URL parsing with query parameter extraction
- Action routing to appropriate handlers
- Integration with Windows file associations

**Protocol Format**:
```
filemgmt://<action>?<param1>=<value1>&<param2>=<value2>
```

**Supported Actions**:
- `add` - Add a new entry
- `open` - Open an existing entry
- `search` - Search for entries
- `settings` - Open settings

**Implementation**:
```rust
pub enum ProtocolAction {
    Add,
    Open,
    Search,
    Settings,
}

pub struct ProtocolRequest {
    pub action: ProtocolAction,
    pub path: Option<String>,
    pub id: Option<String>,
    pub query: Option<String>,
}

pub fn parse_deep_link(url: &str) -> Result<ProtocolRequest>;
```

### Expiration Manager

**Purpose**: Track and manage entry expiration with notifications.

**Design**:
- Periodic background checks (configurable interval)
- Three-tier status tracking: Expired, Expiring Soon, Not Expiring
- Configurable warning threshold
- Notification integration

**Status Flow**:
```mermaid
stateDiagram-v2
    [*] --> Active: Entry created
    Active --> ExpiringSoon: Approaching expiration
    ExpiringSoon --> Expired: Expiration date passed
    Expired --> [*]: Deleted
    Active --> Active: Expiration removed
    ExpiringSoon --> Active: Expiration extended
```

**Implementation**:
```rust
pub struct ExpirationManager {
    conn: Connection,
    config: ExpirationConfig,
}

pub enum ExpirationStatus {
    Expired { expired_at: i64 },
    ExpiringSoon { expires_at: i64, days_remaining: i32 },
    NotExpiring,
}
```

### Sync Manager

**Purpose**: Synchronize database and settings with cloud storage.

**Design**:
- Provider-agnostic sync engine
- Bi-directional synchronization
- Conflict detection and resolution
- Sync history tracking

**Supported Providers**:
- OneDrive
- Jianguoyun (坚果云)
- Custom WebDAV (planned)

**Sync Strategy**:
- Last-modified-wins for automatic resolution
- Manual resolution for conflicts
- Atomic operations with rollback

**Implementation**:
```rust
pub struct SyncManager {
    provider: CloudProvider,
    sync_path: PathBuf,
    status: Arc<Mutex<SyncStatus>>,
}

pub enum CloudProvider {
    OneDrive,
    Jianguoyun,
    WebDAV,
}
```

### Group Manager

**Purpose**: Organize entries into user-defined groups.

**Design**:
- Many-to-many relationship model
- Color-coded visual identification
- Batch operations support
- Import/Export functionality

**Implementation**:
```rust
pub struct GroupManager {
    conn: Connection,
}

impl GroupManager {
    pub fn create_group(&self, name: &str, color: &str) -> Result<Group>;
    pub fn add_entry_to_group(&self, entry_id: i64, group_id: i64) -> Result<()>;
    pub fn batch_add(&self, entry_ids: &[i64], group_id: i64) -> Result<()>;
    pub fn export_group(&self, group_id: i64) -> Result<String>;
}
```

### Shell Extension

**Purpose**: Integrate with Windows Explorer context menu.

**Design**:
- Registry-based context menu registration
- PowerShell installation scripts
- Administrator privilege requirement

**Registry Structure**:
```
HKEY_LOCAL_MACHINE\SOFTWARE\Classes\*\shell\AddToFileManagementCenter
    (Default) = "Add to LNK Management Center"
    Icon = "path\to\app.exe"
    command
        (Default) = "path\to\app.exe" --add "%1"
```

## Design Decisions

### 1. Tauri 2.0 Over Electron

**Decision**: Use Tauri instead of Electron.

**Rationale**:
- Smaller bundle size (~3-10MB vs ~150MB)
- Better performance (native backend vs Node.js)
- Lower memory footprint
- Better Windows integration
- Rust's safety guarantees

**Trade-offs**:
- Rust learning curve for web developers
- Smaller ecosystem compared to Node.js
- Less third-party plugin availability

### 2. SQLite Over Server-Based Database

**Decision**: Use SQLite instead of PostgreSQL/MySQL.

**Rationale**:
- Zero configuration for end users
- Embedded database (no server process)
- Excellent performance for local applications
- Full ACID compliance
- Single file storage

**Trade-offs**:
- No concurrent write access from multiple processes
- Limited scalability for very large datasets
- No built-in replication

### 3. Global Hotkey System

**Decision**: Implement system-wide hotkey activation.

**Rationale**:
- Quick access without Alt-Tab
- Consistent user experience across Windows versions
- Native Windows API integration

**Trade-offs**:
- May conflict with other applications
- Requires user configuration
- Platform-specific implementation

### 4. Deep Link Protocol

**Decision**: Implement custom protocol handler (filemgmt://).

**Rationale**:
- Integration with web browsers
- Email/document linking support
- Cross-application integration
- CLI and batch script support

**Trade-offs**:
- Requires protocol registration in Windows
- URL length limitations
- Security considerations for untrusted input

### 5. Expiration Reminder System

**Decision**: Built-in expiration tracking instead of external reminders.

**Rationale**:
- Integrated user experience
- Automatic notifications
- Context-aware actions (extend, delete, ignore)
- Data lifecycle management

**Trade-offs**:
- Increased complexity
- Background resource usage
- Notification spam risk

### 6. Cloud Sync Architecture

**Decision**: File-based sync instead of direct API integration.

**Rationale**:
- Works with any cloud storage provider
- No OAuth/API key management
- User-controlled data location
- Offline capability

**Trade-offs**:
- No real-time sync
- Manual conflict resolution required
- Slower than direct API integration

## Performance Considerations

### Database Optimization

- **Indexes**: Created on frequently queried columns
- **FTS5**: Full-text search with optimized ranking
- **Connection Pooling**: Reuse connections to reduce overhead
- **Transactions**: Batch operations for performance

### Memory Management

- **Rust Ownership**: Automatic memory management without GC
- **RAII Pattern**: Resources automatically cleaned up
- **Smart Pointers**: Arc<Mutex> for shared state
- **Lazy Loading**: Load data on demand

### UI Performance

- **Virtual Scrolling**: Render only visible items
- **Memoization**: Cache expensive computations
- **Debouncing**: Delay rapid updates
- **Code Splitting**: Load components lazily

## Security Architecture

### Data Protection

- **Local Storage**: All data stored locally by default
- **No Telemetry**: No data sent to external servers
- **File Permissions**: User-only access to database
- **Path Validation**: Prevent directory traversal attacks

### Windows Integration

- **Registry Isolation**: User-specific registry keys
- **Administrator Privileges**: Required only for shell extension
- **Protocol Registration**: User-level protocol handler

### Network Security

- **HTTPS Only**: Cloud sync uses secure connections
- **No Hardcoded Secrets**: All credentials user-provided
- **Certificate Validation**: Verify SSL/TLS certificates

## Scalability

### Database Capacity

- **Entries**: Supports millions of entries
- **Groups**: No practical limit
- **Search**: Optimized FTS5 with ranking

### Performance at Scale

- **Indexing**: Maintains query performance
- **Pagination**: Load entries in chunks
- **Lazy Loading**: Load details on demand

## Extensibility

### Plugin System (Planned)

Future versions may support:
- Custom entry types
- External data sources
- Theme customization
- Workflow automation

### API Extension

Backend can be extended by:
- Adding new Tauri commands
- Creating new modules
- Integrating external libraries
- Adding new protocol actions

## Related Documentation

- **[Database Schema](./database-schema.md)** - Data layer details
- **[API Reference](./api.md)** - Backend interface
- **[Build & Deploy](./build-deploy.md)** - Deployment architecture