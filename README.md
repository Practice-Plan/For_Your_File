# LNK File Management Center

<img src="src-tauri/icons/Square89x89Logo.png" alt="icon Logo" width="80" height="80" style="border-radius: 12px;">

> A modern Windows desktop app for managing LNK shortcut files, built with Tauri 2.0.

[中文](README-zh.md) | English | [Français](README-fr.md) | [Русский](README-ru.md) | [العربية](README-ar.md)

![License](https://img.shields.io/badge/License-GPL--3.0-blue.svg)
![Version](https://img.shields.io/badge/Version-0.0.2-green.svg)
![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg)
![Rust](https://img.shields.io/badge/Rust-1.77.2+-orange.svg)
![Node](https://img.shields.io/badge/Node-18%2B-339933.svg)

LNK File Management Center helps you manage Windows shortcut files (.lnk) in one place. With full-text search, smart grouping, expiration reminders, global hotkeys, and one-click launch, you can stop losing track of shortcuts spread across the desktop and Start menu.

## ✨ Features

- **🔍 Full-text search** — Real-time SQLite FTS5 search with prefix matching, pagination, search history, and keyword highlighting
- **📁 App scanning** — Automatically scans installed apps from the Start menu, with native Win32 icon extraction and disk-based cache (millisecond-level loading)
- **🚀 One-click launch** — Double-click to start apps, open files, folders, or URLs, while tracking usage frequency automatically
- **🖥️ Global hotkeys** — Customizable global shortcuts (default: `Alt+Space`), conflict detection, and suggestions
- **⏰ Expiration reminders** — Set expiration dates for temporary files and receive reminders or clean them in bulk when they expire
- **📚 Smart grouping** — 8-color grouping, drag-and-drop assignment, batch operations, and group import/export (JSON/CSV/HTML)
- **📦 Batch import** — Drag-and-drop or browse to import multiple items, with unified tag/parameter/open-mode configuration and live progress
- **🌐 Internationalization** — Supports Chinese, English, Français, Русский, and العربية
- **🎨 Theme switching** — Light/dark themes with user preference persistence
- **🖱️ Context menu integration** — Windows Explorer right-click menu integration for fast add actions
- **🔗 Deep links** — `filemgmt://` protocol support for add/open/search/settings
- **🧩 PPC integration** — Connects to the PPC central processing system (v0.0.7) with error-code mapping
- **🖥️ System tray** — Hide to tray on close, with hotkey/tray actions to bring the app back

## 🛠️ Tech stack

| Layer | Technology |
|----|------|
| Framework | Tauri 2.0 |
| Frontend | React 18 + TypeScript 5 |
| Build tool | Vite 5 |
| Styling | Tailwind CSS 3 |
| Animation | Framer Motion |
| Backend | Rust (edition 2021) |
| Database | SQLite (rusqlite, bundled) + FTS5 full-text search |
| Internationalization | i18next (en/zh/fr/ru/ar) |
| Icon extraction | Win32 API (SHGetFileInfoW + GDI) |

## 📋 Requirements

- **Operating system**: Windows 10 / Windows 11
- **Runtime**: Microsoft Edge WebView2 Runtime (preinstalled on Windows 11; required on Windows 10)
- **Development**: Rust 1.77.2+, Node.js 18+

## 🚀 Quick start

### Development mode

```bash
# 1. Install frontend dependencies
npm install

# 2. Start the dev server (default port 1420)
npm run dev
```

### Production build

```bash
# Recommended: build the full Tauri app (builds frontend + embeds resources automatically)
npx tauri build

# Generate installation package (64-bit)
npx tauri build --target x86_64-pc-windows-msvc

# Generate installation package (32-bit; run rustup target add i686-pc-windows-msvc first)
npx tauri build --target i686-pc-windows-msvc
```

Artifacts are generated under `src-tauri/target/<target>/release/bundle/` and include both `msi/` and `nsis/` installers.

> **⚠️ Important**: Running `cargo build --release` alone produces a broken binary because it tries to connect to the dev server and may fail with "Connection Refused". Always use `npx tauri build` or `cargo build --release --features custom-protocol` to embed frontend resources correctly.

## 📂 Project structure

```text
For_Your_File/
├── src/                          # React frontend
│   ├── components/               # UI components (30+)
│   │   ├── BatchImportModal.tsx  # Batch import (progress bar / error summary)
│   │   ├── AppSelectorModal.tsx  # App selector (icon cache / concurrency control)
│   │   └── ...
│   ├── hooks/                    # Custom hooks (useSearch, etc.)
│   ├── locales/                  # 5-language translations
│   ├── types/                    # TypeScript types
│   └── App.tsx                   # Main app
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── commands.rs           # 60+ Tauri commands
│   │   ├── db.rs                 # SQLite schema + FTS5 triggers
│   │   ├── hotkey.rs             # Global hotkey management
│   │   ├── lnk.rs                # LNK parsing (COM/IShellLinkW)
│   │   ├── app_scanner.rs        # Start menu scanning + native icon extraction
│   │   ├── expiration/           # Expiration reminder system
│   │   ├── ppc_linker.rs         # PPC integration
│   │   └── ...
│   ├── tests/                    # Rust integration tests
│   └── tauri.conf.json           # Tauri config
├── docs/                         # Documentation
│   ├── tech/                     # Technical docs
│   └── user/                     # User guide
├── .github/workflows/ci.yml      # CI pipeline (tests + 32/64-bit packaging)
├── package.json
└── LICENSE                       # GPL-3.0
```

## 🧪 Testing

```bash
# Frontend type-check
npm run type-check

# Rust unit and integration tests
cd src-tauri && cargo test

# Clippy check
cargo clippy -- -D warnings

# Format check
cargo fmt -- --check
```

## 🔧 Database

- **Location**: `%APPDATA%/lnk-management/lnk_management.db`
- **Tables**: `entries` (items), `groups` (groups), `entry_groups` (many-to-many), `entries_fts` (FTS5 full-text index)
- **Icon cache**: `%APPDATA%/lnk-management/icon_cache/` (hash key + modification-time invalidation)

## 📄 Documentation

- [Technical documentation](docs/tech/README.md) — architecture, API reference, database design, build and deployment
- [User guide](docs/user/README.md) — installation, feature usage, shortcuts, FAQ, troubleshooting

## 🤝 Contributing

Contributions are welcome via Issues and Pull Requests. Please make sure:

1. The code passes `cargo fmt` and `cargo clippy`
2. Tests pass (`cargo test`)
3. Commit messages clearly describe the change

## 📜 License

This project is open source under the [GNU General Public License v3.0](LICENSE).

Copyright © 2026 LNK File Management Center Contributors
