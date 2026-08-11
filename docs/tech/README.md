# LNK File Management Center - Technical Documentation

Welcome to the technical documentation for LNK File Management Center, a modern desktop application for managing Windows LNK (shortcut) files.

## Overview

LNK File Management Center provides a safe and intelligent way to manage Windows shortcuts with features like:
- Global hotkey activation
- Deep link protocol support (filemgmt://)
- Expiration reminder system
- Cloud synchronization (OneDrive, Jianguoyun)
- Group organization
- Windows Explorer integration
- Full-text search

## Architecture

### Technology Stack

| Layer | Technology | Version |
|-------|-----------|---------|
| **Backend** | Rust (Tauri 2.0) | 2.11.3 |
| **Frontend** | React + TypeScript | React 18.3.1 |
| **Build Tool** | Vite | 5.3.4 |
| **Database** | SQLite (rusqlite) | 0.32 |
| **Styling** | Tailwind CSS | 3.4.6 |
| **Animations** | Framer Motion | 11.0.0 |

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                      User Interface                          │
│                   (React + TypeScript)                       │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐   │
│  │  Entry   │  │  Groups  │  │ Settings │  │  Search  │   │
│  │Management│  │Management│  │   Panel  │  │Interface │   │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘   │
└─────────────────────────────────────────────────────────────┘
                           ↕ IPC
┌─────────────────────────────────────────────────────────────┐
│                    Backend (Rust/Tauri)                      │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │  Hotkey      │  │  Protocol    │  │ Expiration   │     │
│  │  Manager     │  │  Handler     │  │ Manager      │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   Sync       │  │  Group       │  │ Shell        │     │
│  │  Manager     │  │  Manager     │  │ Extension    │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
                           ↕
┌─────────────────────────────────────────────────────────────┐
│                      Data Layer                               │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐     │
│  │   SQLite     │  │  File        │  │   Cloud      │     │
│  │  Database    │  │  System      │  │  Storage     │     │
│  └──────────────┘  └──────────────┘  └──────────────┘     │
└─────────────────────────────────────────────────────────────┘
```

## Core Modules

### 1. Hotkey Manager
- Global Windows hotkey registration
- Configurable hotkey combinations
- Window visibility toggling
- Conflict detection

### 2. Protocol Handler
- Deep link protocol support (`filemgmt://`)
- URL routing and parameter parsing
- Integration with Windows file associations
- CLI argument handling

### 3. Expiration Manager
- Automatic expiration tracking
- Warning notifications
- Scheduled cleanup tasks
- Configurable expiration policies

### 4. Sync Manager
- Cloud provider detection (OneDrive, Jianguoyun)
- Bi-directional synchronization
- Conflict resolution
- Sync history tracking

### 5. Group Manager
- Hierarchical organization
- Batch operations
- Import/Export functionality
- Color-coded visual identification

### 6. Shell Extension
- Windows Explorer context menu integration
- Registry management
- Installation/uninstallation scripts

## Documentation Index

### Core Documentation
- **[Database Schema](./database-schema.md)** - Complete database structure, ERD diagrams, and migration history
- **[Backend API](./api.md)** - Tauri command reference, request/response formats
- **[Architecture](./architecture.md)** - System design, components, and data flow

### Operations
- **[Build & Deployment](./build-deploy.md)** - Development setup, build procedures, release process
- **[Contributing](./contributing.md)** - Code style guide, PR process, testing requirements

## Quick Links

### For Developers
- [Getting Started](./build-deploy.md#development-setup)
- [API Reference](./api.md)
- [Database Schema](./database-schema.md)

### For DevOps
- [Build Procedures](./build-deploy.md#building)
- [Deployment Guide](./build-deploy.md#deployment)
- [Configuration](./build-deploy.md#configuration)

### For Contributors
- [Contribution Guidelines](./contributing.md)
- [Code Style Guide](./contributing.md#code-style)
- [Commit Message Format](./contributing.md#commit-messages)

## System Requirements

### Development
- **OS**: Windows 10/11 (64-bit)
- **Rust**: 1.77.2 or later
- **Node.js**: 18.x or later
- **Windows SDK**: 10.0 or later

### Production
- **OS**: Windows 10/11 (64-bit)
- **RAM**: 4 GB minimum, 8 GB recommended
- **Disk**: 100 MB for application, additional for database

## Security Considerations

- Local SQLite database with no network exposure
- Deep link protocol requires explicit user action
- Shell extension requires administrator privileges
- No telemetry or data collection
- All data stored locally by default

## Version History

| Version | Date | Changes |
|---------|------|---------|
| 0.1.0 | 2026-07 | Initial release |

## License

MIT License - See LICENSE file for details

## Support

For technical support or feature requests, please open an issue on the project repository.