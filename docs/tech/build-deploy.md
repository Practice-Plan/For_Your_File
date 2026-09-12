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

### Production Build (Compile Only — No Installer)

v0.0.4+ disables Tauri bundling (`bundle.active: false`) to speed up CI
and avoid installer overhead. The workflow is: compile → run smoke tests → ship
the raw executable.

```powershell
# Build optimized production binary (compile only, no installer)
cargo build --manifest-path src-tauri/Cargo.toml --release --no-default-features
```

Build output:
```
src-tauri/target/release/
├── lnk-file-management-center.exe  # Main executable (self-contained)
```

Frontend production bundle (separate step, called by `beforeBuildCommand`):
```powershell
npm run build
# Output: dist/assets/ — 17 chunks, max 143 kB, zero 500 kB warnings
```

> **Note**: Full `npm run tauri build` still works but produces only the
> raw `.exe` (no MSI/NSIS). The `bundle.targets` is intentionally empty in
> `tauri.conf.json`. If an installer is needed, set `"active": true` and
> `"targets": ["nsis"]` locally — do NOT commit that change.

### CI Build Pipeline

`.github/workflows/ci.yml` runs a single `check` job on every push/PR:

| Step | Command | Purpose |
|------|---------|---------|
| Version sync | PowerShell assert | Verify package.json / Cargo.toml / tauri.conf.json all match |
| Type-check | `npm run type-check` | Zero TypeScript errors |
| Rust tests | `cargo test --verbose` | All unit tests pass |
| Lint | `cargo clippy -- -D warnings` | Zero clippy warnings |
| Format | `cargo fmt -- --check` | Code style correct |
| Compile | `cargo build --release --no-default-features` | Produce `.exe` only |

No artifact upload, no installer generation — CI is purely a quality gate.

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
// v0.0.4+ uses manualChunks + React.lazy for optimal code-splitting
export default defineConfig({
  build: {
    minify: 'esbuild',
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalized = id.replace(/\\/g, '/');
          if (!normalized.includes('node_modules/')) return undefined;

          if (normalized.includes('react') || normalized.includes('scheduler'))
            return 'vendor-react';
          if (normalized.includes('framer-motion') || normalized.includes('motion-'))
            return 'vendor-framer';
          if (normalized.includes('i18next'))
            return 'vendor-i18n';
          if (normalized.includes('@tauri-apps'))
            return 'vendor-tauri';
          return undefined; // let Rollup auto-group the rest
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
    "active": false,
    "targets": [],
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/icon.ico"
    ]
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

### Direct Executable (Current)

v0.0.4+ ships as a single `.exe` — no installer, no MSI, no NSIS.
Simply copy `lnk-file-management-center.exe` to the target machine and run.

```powershell
# Build the executable
cargo build --manifest-path src-tauri/Cargo.toml --release --no-default-features

# Find the output
dir src-tauri\target\release\lnk-file-management-center.exe
```

### If You Need an Installer (Future)

Installers are currently disabled in `tauri.conf.json`. To re-enable locally:

```json
{
  "bundle": {
    "active": true,
    "targets": ["nsis"]
  }
}
```

Then:
```powershell
npm run tauri build
# Output: src-tauri/target/release/bundle/nsis/*-setup.exe
```

> **Do not commit** installer-related changes to `tauri.conf.json`.
> The CI pipeline and default build both target compile-only.

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

**Location**: `%APPDATA%\wang.station\app\For_Your_File\hotkey_config.json`

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

**Default**: `%APPDATA%\wang.station\app\For_Your_File\lnk_management.db`

**Custom Location**:
```powershell
# Set environment variable
set LNK_DB_PATH=D:\Data\lnk_management.db

# Or modify configuration file
```

### Logging

**Log Level**: Configurable in development mode

**Log Location**: `%APPDATA%\wang.station\app\For_Your_File\logs\`

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
copy %APPDATA%\wang.station\app\For_Your_File\lnk_management.db %APPDATA%\wang.station\app\For_Your_File\backup\lnk_management_%date%.db

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
  copy backup\lnk_management_20260725.db %APPDATA%\wang.station\app\For_Your_File\lnk_management.db
   ```

3. **Deploy previous version**:
   ```powershell
   # v0.0.4+ ships as raw exe — just copy and run
   copy LNK_File_Management_Center_0.0.3.exe LNK_File_Management_Center.exe
   ```

### Configuration Migration

```powershell
# Backup configuration
copy %APPDATA%\wang.station\app\For_Your_File\hotkey_config.json config_backup.json

# Restore configuration
copy config_backup.json %APPDATA%\wang.station\app\For_Your_File\hotkey_config.json
```

## Monitoring and Logging

### Application Logs

**Location**: `%APPDATA%\wang.station\app\For_Your_File\logs\app.log`

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

**Location**: `%APPDATA%\wang.station\app\For_Your_File\crashes\`

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
type %APPDATA%\wang.station\app\For_Your_File\logs\app.log

# Verify database integrity
sqlite3 %APPDATA%\wang.station\app\For_Your_File\lnk_management.db "PRAGMA integrity_check;"

# Reset configuration
del %APPDATA%\wang.station\app\For_Your_File\hotkey_config.json
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