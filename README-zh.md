# LNK File Management Center

<img src="src-tauri/icons/Square89x89Logo.png" alt="icon Logo" width="80" height="80" style="border-radius: 12px;">

> 一个现代化的 Windows LNK 快捷方式管理桌面应用，基于 Tauri 2.0 构建。

中文 | [English](README.md) | [Français](README-fr.md) | [Русский](README-ru.md) | [العربية](README-ar.md)

![License](https://img.shields.io/badge/License-GPL--3.0-blue.svg)
![Version](https://img.shields.io/badge/Version-0.0.3-green.svg)
![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg)
![Rust](https://img.shields.io/badge/Rust-1.77.2+-orange.svg)
![Node](https://img.shields.io/badge/Node-18%2B-339933.svg)

LNK File Management Center 帮助你集中管理 Windows 快捷方式（.lnk）文件。通过全文搜索、智能分组、过期提醒、全局热键和一键打开，告别散落桌面与开始菜单的快捷方式混乱。

## ✨ 功能特性

- **🔍 全文搜索** — 基于 SQLite FTS5 实时搜索，支持前缀匹配、分页加载、搜索历史与关键词高亮
- **📁 应用扫描** — 自动扫描开始菜单中的应用，原生 Win32 图标提取 + 磁盘缓存（毫秒级加载）
- **🚀 一键打开** — 双击即启动应用/打开文件/文件夹/URL，自动追踪使用频率
- **🖥️ 全局热键** — 支持自定义全局热键（默认 `Alt+Space`），冲突检测与建议
- **⏰ 过期提醒** — 为临时文件设置过期时间，到期自动提醒、批量清理
- **📚 智能分组** — 8 色分组、拖拽分配、批量操作、组导入/导出（JSON/CSV/HTML）
- **📦 批量导入** — 拖放或浏览批量导入，统一配置标签/参数/打开方式，实时进度条
- **🗃️ 数据库预览** — 在独立窗口查看完整本地数据库，支持真实进度和分页加载
- **🌐 国际化** — 支持中文、English、Français、Русский、العربية 五种语言
- **🎨 主题切换** — 深色/浅色主题，记忆用户偏好
- **🖱️ 右键菜单** — Windows Explorer 右键菜单集成，一键添加
- **🔗 深度链接** — `filemgmt://` 协议支持 add/open/search/settings
- **🧩 PPC 集成** — 连接 PPC 中央处理系统（v0.0.7），错误码映射
- **🖥️ 系统托盘** — 关闭隐藏到托盘，热键/托盘一键唤起

## 🛠️ 技术栈

| 层 | 技术 |
|----|------|
| 框架 | Tauri 2.0 |
| 前端 | React 18 + TypeScript 5 |
| 构建工具 | Vite 5 |
| 样式 | Tailwind CSS 3 |
| 动画 | Framer Motion |
| 后端 | Rust (edition 2021) |
| 数据库 | SQLite (rusqlite, bundled) + FTS5 全文搜索 |
| 国际化 | i18next (en/zh/fr/ru/ar) |
| 图标提取 | Win32 API (SHGetFileInfoW + GDI) |

## 📋 系统要求

- **操作系统**: Windows 10 / Windows 11
- **运行时**: Microsoft Edge WebView2 Runtime（Windows 11 预装，Windows 10 需安装）
- **开发**: Rust 1.77.2+、Node.js 18+

## 🚀 快速开始

### 开发模式

```bash
# 1. 安装前端依赖
npm install

# 2. 启动开发服务器（默认端口 1420）
npm run dev
```

### 生产构建

```bash
# 推荐方式：构建完整 Tauri 应用（自动构建前端 + 嵌入资源）
npx tauri build

# 生成安装包（64 位）
npx tauri build --target x86_64-pc-windows-msvc

# 生成安装包（32 位，需先 rustup target add i686-pc-windows-msvc）
npx tauri build --target i686-pc-windows-msvc
```

产物位于 `src-tauri/target/<target>/release/bundle/`，包含 `msi/` 与 `nsis/` 两种安装包。

> **⚠️ 重要**：单独执行 `cargo build --release` 会产生损坏的二进制文件（会尝试连接 dev server 并报 "Connection Refused"）。必须通过 `npx tauri build` 或 `cargo build --release --features custom-protocol` 构建以嵌入前端资源。

## 📂 项目结构

```
For_Your_File/
├── src/                          # React 前端
│   ├── components/               # UI 组件（30+）
│   │   ├── BatchImportModal.tsx  # 批量导入（进度条/错误汇总）
│   │   ├── AppSelectorModal.tsx  # 应用选择器（图标缓存/并发控制）
│   │   └── ...
│   ├── hooks/                    # 自定义 Hooks（useSearch 等）
│   ├── locales/                  # 5 国语言翻译
│   ├── types/                    # TypeScript 类型定义
│   └── App.tsx                   # 主应用
├── src-tauri/                    # Rust 后端
│   ├── src/
│   │   ├── commands.rs           # 60+ Tauri 命令
│   │   ├── db.rs                 # SQLite schema + FTS5 触发器
│   │   ├── hotkey.rs             # 全局热键管理
│   │   ├── lnk.rs                # LNK 解析（COM/IShellLinkW）
│   │   ├── app_scanner.rs        # 开始菜单扫描 + 原生图标提取
│   │   ├── expiration/           # 过期提醒系统
│   │   ├── ppc_linker.rs         # PPC 集成
│   │   └── ...
│   ├── tests/                    # Rust 集成测试
│   └── tauri.conf.json           # Tauri 配置
├── docs/                         # 文档
│   ├── tech/                     # 技术文档
│   └── user/                     # 用户指南
├── .github/workflows/ci.yml      # CI（测试 + 32/64 位打包）
├── package.json
└── LICENSE.md                    # GPL-3.0
```

## 🧪 测试

```bash
# 前端类型检查
npm run type-check

# Rust 单元测试 + 集成测试
cd src-tauri && cargo test

# Clippy 检查
cargo clippy -- -D warnings

# 格式检查
cargo fmt -- --check
```

## 🔧 数据库

- **位置**: `%APPDATA%/wang.station/app/For_Your_File/lnk_management.db`
- **表**: `entries`（条目）、`groups`（分组）、`entry_groups`（多对多）、`entries_fts`（FTS5 全文索引）
- **图标缓存**: `%APPDATA%/wang.station/app/For_Your_File/icon_cache/`（hash 键 + 修改时间失效）

## 📄 文档

- [技术文档](docs/tech/README.md) — 架构、API 参考、数据库设计、构建部署
- [用户指南](docs/user/README.md) — 安装、功能使用、快捷键、FAQ、故障排查

## 🤝 贡献

欢迎提交 Issue 与 Pull Request。请确保：

1. 代码通过 `cargo fmt` 与 `cargo clippy`
2. 测试全部通过（`cargo test`）
3. 提交信息清晰描述变更

## 📜 许可证

本项目基于 [GNU General Public License v3.0](LICENSE.md) 开源。

Copyright © 2026 LNK File Management Center Contributors
