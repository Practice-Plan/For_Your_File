# LNK File Management Center - Frontend

A modern desktop application for managing Windows LNK (shortcut) files, built with Tauri, React, and TypeScript.

## Tech Stack

- **Framework**: Tauri 2.0 (Rust backend + Web frontend)
- **Frontend**: React 18 + TypeScript
- **Build Tool**: Vite 5
- **Styling**: Tailwind CSS 3
- **Animations**: Framer Motion

## Project Structure

```
For_Your_File/
├── src/
│   ├── components/     # Reusable UI components
│   ├── hooks/          # Custom React hooks
│   ├── utils/          # Utility functions
│   ├── types/          # TypeScript type definitions
│   ├── App.tsx         # Main application component
│   ├── main.tsx        # Application entry point
│   └── index.css       # Global styles
├── public/             # Static assets
├── index.html          # HTML entry point
├── vite.config.ts      # Vite configuration
├── tailwind.config.js  # Tailwind CSS configuration
├── tsconfig.json       # TypeScript configuration
└── package.json        # Dependencies and scripts
```

## Features

- Dark/Light theme support
- Modern UI with smooth animations
- Window drag region for custom title bar
- Responsive layout with sidebar navigation

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Type check
npm run type-check
```

## Production Build (Tauri)

```bash
# Build frontend + Rust backend with embedded resources (recommended)
npx tauri build

# OR build manually (must pass --features custom-protocol for embedded frontend)
npm run build
cargo build --release --features custom-protocol
```

> **Important**: `cargo build --release` alone produces a **broken binary** that tries
> to connect to `http://localhost:1420` (dev server) and shows "Connection Refused".
> The `custom-protocol` feature tells Tauri to use the embedded frontend resources
> (`tauri://localhost`) instead of the dev URL.

## Integration with Tauri

This frontend will be integrated with a Rust backend using Tauri. The actual Tauri initialization should be done in the root project directory.

## License

GPL-3.0