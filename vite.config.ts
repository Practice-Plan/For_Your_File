import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react()],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  // Tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // Env variables starting with TAURI_ are exposed to the client
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: process.env.TAURI_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    // Don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    // Produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG,
    // Split vendor deps into dedicated chunks for better caching
    // and to keep the main entry chunk under the 500 kB warning threshold
    rollupOptions: {
      output: {
        manualChunks(id) {
          const normalizedId = id.replace(/\\/g, '/')

          // Skip non-node_modules (app code handled by dynamic import() below)
          if (!normalizedId.includes('node_modules/')) return undefined

          // Node_modules split — group by library family
          // Return undefined for unknown node_modules → Rollup auto-groups
          if (normalizedId.includes('react-dom') || normalizedId.includes('/react/') ||
              normalizedId.includes('scheduler') || normalizedId.includes('loose-envify') ||
              normalizedId.includes('object-assign') || normalizedId.includes('use-sync-external-store') ||
              normalizedId.includes('@types/react')) {
            return 'vendor-react'
          }

          // Framer Motion (biggest single dep after React)
          if (normalizedId.includes('framer-motion') || normalizedId.includes('motion-dom') ||
              normalizedId.includes('motion-utils') || normalizedId.includes('@motionone')) {
            return 'vendor-framer'
          }

          // i18n family
          if (normalizedId.includes('i18next') || normalizedId.includes('react-i18next') ||
              normalizedId.includes('i18next-browser-languagedetector')) {
            return 'vendor-i18n'
          }

          // Tauri API + plugins
          if (normalizedId.includes('@tauri-apps')) {
            return 'vendor-tauri'
          }

          // Everything else — let Rollup auto-assign (prevents empty chunk)
          return undefined
        },
      },
    },
  },
})