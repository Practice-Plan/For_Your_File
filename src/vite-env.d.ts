/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_APP_TITLE: string
  readonly TAURI_PLATFORM: 'linux' | 'darwin' | 'win32' | undefined
  readonly TAURI_ARCH: 'x86_64' | 'arm64' | undefined
}

interface ImportMeta {
  readonly env: ImportMetaEnv
}