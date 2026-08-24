# Build and Deployment Documentation

This document provides comprehensive instructions for building, testing, and deploying LNK File Management Center.

## Prerequisites

### Development Environment

#### Required Software

| Software | Version | Purpose |
|----------|---------|---------|
| **Rust** | 1.77.2+ | Backend language |
| **Node.js** | 18.x+ | Frontend build tool |
| **npm** | 9.x+ | Package manager |
| **Windows SDK** | 10.0+ | Windows API access |
| **Visual Studio Build Tools** | 2022 | C++ toolchain |
| **Git** | 2.x | Version control |

#### Rust Installation

```powershell
# Install Rust using rustup
winget install Rustlang.Rustup

# Or download from https://rustup.rs/
# Verify installation
rustc --version
cargo --version
```

#### Node.js Installation

```powershell
# Install using winget
winget install OpenJS.NodeJS.LTS

# Or download from https://nodejs.org/
# Verify installation
node --version
npm --version
```

#### Windows SDK

Download and install from:
https://developer.microsoft.com/en-us/windows/downloads/windows-sdk/

Required components:
- Windows 10/11 SDK
- C++ build tools
- Windows Runtime libraries

### System Requirements

| Requirement | Minimum | Recommended |
|-------------|---------|-------------|
| **OS** | Windows 10 64-bit | Windows 11 64-bit |
| **RAM** | 4 GB | 8 GB |
| **Disk Space** | 5 GB | 10 GB |
| **CPU** | Dual-core 2.0 GHz | Quad-core 2.5 GHz+ |

## Development Setup

### 1. Clone Repository

```powershell
git clone https://github.com/your-org/lnk-file-management-center.git
cd lnk-file-management-center
```

### 2. Install Dependencies

```powershell
# Install frontend dependencies
npm install

# Rust dependencies are automatically installed by Cargo
```

### 3. Configure Environment

No additional configuration required for development. The application uses:
- Local SQLite database
- Default hotkey configuration
- No external API keys

### 4. Verify Setup

```powershell
# Check Rust toolchain
cargo check

# Check TypeScript
npm run type-check

# Run linting
npm run lint
```

## Building

### Development Build

```powershell
# Start development server with hot reload
npm run tauri dev
```

This command:
1. Starts Vite development server (frontend)
2. Builds and runs Tauri application (backend)
3. Opens application window
4. Enables hot module replacement

### Production Build

```powershell
# Build optimized production bundle
npm run tauri build
```

Build output locations:
```
src-tauri/target/release/
├── lnk-file-management-center.exe  # Main executable
└── bundle/
    ├── msi/
    │   └── LNK File Management Center_0.0.3_x64.msi
    └── nsis/
        └── LNK File Management Center_0.0.3_x64-setup.exe
```

### Build Configuration

#### Cargo.toml (Rust)

```toml
[profile.release]
opt-level = 3          # Maximum optimization
lto = true             # Link-time optimization
codegen-units = 1      # Single codegen unit
strip = true           # Strip symbols
```

#### vite.config.ts (Frontend)

```typescript
export default defineConfig({
  build: {
    target: 'esnext',
    minify: 'terser',
    sourcemap: false,
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
        },
      },
    },
  },
});
```

#### tauri.conf.json (Tauri)

```json
{
  "bundle": {
    "active": true,
    "targets": ["msi", "nsis"],
    "windows": {
      "wix": {
        "language": "zh-CN"
      }
    }
  }
}
```

## Build Variants

### Debug Build

```powershell
cargo build
```

Features:
- Debug symbols included
- Assertions enabled
- No optimization
- Larger binary size
- Slower performance

### Release Build

```powershell
cargo build --release
```

Features:
- Optimized binary
- Assertions disabled
- Symbols stripped
- Smaller binary size
- Maximum performance

## Testing

### Unit Tests

```powershell
# Run Rust tests
cargo test

# Run frontend tests
npm test
```

### Integration Tests

```powershell
# Run all tests
cargo test --all

# Run specific test
cargo test test_expiration_config_default
```

### End-to-End Tests

```powershell
# Build and run E2E tests
npm run test:e2e
```

### Code Coverage

```powershell
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## Deployment

### MSI Installer

**Location**: `src-tauri/target/release/bundle/msi/`

**Installation**:
```powershell
# Install MSI
msiexec /i "LNK File Management Center_0.0.3_x64.msi"

# Silent install
msiexec /i "LNK File Management Center_0.0.3_x64.msi" /quiet

# Uninstall
msiexec /x "LNK File Management Center_0.0.3_x64.msi"
```

### NSIS Installer

**Location**: `src-tauri/target/release/bundle/nsis/`

**Installation**:
```powershell
# Run installer
.\LNK File Management Center_0.0.3_x64-setup.exe

# Silent install
.\LNK File Management Center_0.0.3_x64-setup.exe /S

# Uninstall
"C:\Program Files\LNK File Management Center\uninstall.exe" /S
```

### Portable Version

Create a portable version by copying:
```powershell
# Copy executable and dependencies
xcopy src-tauri\target\release\lnk-file-management-center.exe .\portable\
```

## Release Process

### 1. Version Bump

Update version in:
- `package.json`
- `src-tauri/Cargo.toml`
- `src-tauri/tauri.conf.json`

```powershell
# Update version in package.json
npm version patch  # or minor, or major
```

### 2. Update Changelog

Create `CHANGELOG.md` entry:

```markdown
## [0.2.0] - 2026-07-26

### Added
- New feature X
- Support for Y

### Changed
- Improved performance of Z

### Fixed
- Bug in feature A

### Breaking Changes
- Removed deprecated API B
```

### 3. Build Release

```powershell
# Clean previous builds
cargo clean
npm run clean

# Build production release
npm run tauri build
```

### 4. Test Release

```powershell
# Install MSI on test machine
msiexec /i bundle\msi\*.msi

# Verify installation
# - Check shortcuts
# - Verify uninstall
# - Test functionality
```

### 5. Code Signing (Optional)

**Prerequisites**:
- Code signing certificate
- SignTool (Windows SDK)

**Signing**:
```powershell
# Sign executable
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com lnk-file-management-center.exe

# Sign installer
signtool sign /f certificate.pfx /p password /t http://timestamp.digicert.com installer.msi
```

### 6. Create Release

```powershell
# Tag release
git tag -a v0.2.0 -m "Release version 0.2.0"
git push origin v0.2.0

# Create GitHub release
# Upload artifacts:
# - MSI installer
# - NSIS installer
# - Portable ZIP
# - Source code
```

## Configuration

### Application Configuration

**Location**: `%APPDATA%\lnk-management\config.json`

**Default Configuration**:
```json
{
  "hotkey": {
    "modifiers": ["Alt"],
    "key": "Space"
  },
  "expiration": {
    "warning_days": 7,
    "enable_notifications": true,
    "auto_delete_expired": false,
    "check_interval_hours": 1
  },
  "sync": {
    "enabled": false,
    "provider": null,
    "sync_path": null
  },
  "ui": {
    "theme": "dark",
    "language": "zh-CN"
  }
}
```

### Database Location

**Default**: `%APPDATA%\lnk-management\lnk_management.db`

**Custom Location**:
```powershell
# Set environment variable
set LNK_DB_PATH=D:\Data\lnk_management.db

# Or modify configuration file
```

### Logging

**Log Level**: Configurable in development mode

**Log Location**: `%APPDATA%\lnk-management\logs\`

**Configuration**:
```rust
// Enable debug logging in development
if cfg!(debug_assertions) {
    app.handle().plugin(
        tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Debug)
            .build(),
    )?;
}
```

## Update Mechanism

### Automatic Updates (Planned)

```rust
// Using tauri-plugin-updater
app.plugin(tauri_plugin_updater::Builder::new().build());

// Check for updates
let update = app.updater().check().await?;
if let Some(update) = update {
    update.download_and_install().await?;
}
```

### Manual Updates

1. Download new installer
2. Run installer (uninstalls old version automatically)
3. Configure migration (if needed)

## Rollback Procedures

### Database Backup

```powershell
# Manual backup
copy %APPDATA%\lnk-management\lnk_management.db %APPDATA%\lnk-management\backup\lnk_management_%date%.db

# Automated backup (Windows Task Scheduler)
schtasks /create /tn "LNK Backup" /tr "cmd /c copy ..." /sc daily /st 02:00
```

### Rollback to Previous Version

1. **Uninstall current version**:
   ```powershell
   msiexec /x {product-code}
   ```

2. **Restore database backup**:
   ```powershell
   copy backup\lnk_management_20260725.db %APPDATA%\lnk-management\lnk_management.db
   ```

3. **Install previous version**:
   ```powershell
  msiexec /i LNK_File_Management_Center_0.0.3_x64.msi
   ```

### Configuration Migration

```powershell
# Backup configuration
copy %APPDATA%\lnk-management\config.json config_backup.json

# Restore configuration
copy config_backup.json %APPDATA%\lnk-management\config.json
```

## Monitoring and Logging

### Application Logs

**Location**: `%APPDATA%\lnk-management\logs\app.log`

**Log Format**:
```
[2026-07-26 10:30:45 INFO] Application started
[2026-07-26 10:30:46 INFO] Database initialized
[2026-07-26 10:30:47 DEBUG] Hotkey registered: Alt+Space
```

### Performance Monitoring

```rust
// Add performance metrics
use std::time::Instant;

let start = Instant::now();
// ... operation ...
let duration = start.elapsed();
log::info!("Operation completed in {:?}", duration);
```

### Crash Reports

**Location**: `%APPDATA%\lnk-management\crashes\`

**Report Contents**:
- Stack trace
- System information
- Application version
- Last operations

## Troubleshooting

### Build Fails

**Problem**: Rust compilation errors

**Solutions**:
```powershell
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cargo clean

# Check for missing dependencies
cargo check --verbose
```

### Missing Windows SDK

**Problem**: Link errors related to Windows API

**Solution**: Install Windows SDK
```powershell
# Install via winget
winget install Microsoft.WindowsSDK
```

### Node.js Version Mismatch

**Problem**: Incompatible Node.js version

**Solution**: Use nvm to manage Node.js versions
```powershell
# Install nvm-windows
winget install CoreyButler.NVMforWindows

# Install correct Node.js version
nvm install 18.17.0
nvm use 18.17.0
```

### MSI Installation Fails

**Problem**: MSI installer fails to install

**Solutions**:
```powershell
# Check Windows Installer logs
msiexec /i installer.msi /l*v install.log

# Run as administrator
# Disable antivirus temporarily
# Check for previous installation
```

### Application Won't Start

**Problem**: Application crashes on startup

**Solutions**:
```powershell
# Check logs
type %APPDATA%\lnk-management\logs\app.log

# Verify database integrity
sqlite3 %APPDATA%\lnk-management\lnk_management.db "PRAGMA integrity_check;"

# Reset configuration
del %APPDATA%\lnk-management\config.json
```

## CI/CD Pipeline

### GitHub Actions (Example)

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: 18

      - name: Install dependencies
        run: npm install

      - name: Build
        run: npm run tauri build

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: installers
          path: src-tauri/target/release/bundle/
```

## Best Practices

### Development

- Use feature branches for new features
- Write tests for new functionality
- Update documentation
- Run linters before committing
- Test on multiple Windows versions

### Build

- Use release builds for production
- Enable all optimizations
- Strip debug symbols
- Sign executables and installers
- Test installers thoroughly

### Deployment

- Use semantic versioning
- Create release notes
- Provide rollback instructions
- Monitor crash reports
- Have a communication plan

## Related Documentation

- **[Architecture](./architecture.md)** - System design
- **[API Reference](./api.md)** - Backend interface
- **[Contributing](./contributing.md)** - Development guidelines