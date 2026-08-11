import { useEffect, useState } from 'react'
import { getCurrentWindow, PhysicalPosition, PhysicalSize } from '@tauri-apps/api/window'

interface WindowPosition {
  x: number
  y: number
  width: number
  height: number
  maximized: boolean
}

const STORAGE_KEY = 'lnk-window-position'

export function useWindowPosition() {
  const [windowState, setWindowState] = useState<WindowPosition>({
    x: 0,
    y: 0,
    width: 1000,
    height: 700,
    maximized: false,
  })

  useEffect(() => {
    async function loadWindowState() {
      try {
        const appWindow = getCurrentWindow()

        // Load from localStorage
        const saved = localStorage.getItem(STORAGE_KEY)
        if (saved) {
          const parsed = JSON.parse(saved)
          setWindowState(parsed)

          // Apply window position and size
          if (!parsed.maximized) {
            await appWindow.setPosition(new PhysicalPosition(parsed.x, parsed.y))
            await appWindow.setSize(new PhysicalSize(parsed.width, parsed.height))
          }
        }
      } catch (error) {
        console.error('Failed to load window state:', error)
      }
    }

    loadWindowState()
  }, [])

  useEffect(() => {
    async function saveWindowState() {
      try {
        // Save to localStorage
        localStorage.setItem(STORAGE_KEY, JSON.stringify(windowState))
      } catch (error) {
        console.error('Failed to save window state:', error)
      }
    }

    // Debounce saving
    const timeout = setTimeout(saveWindowState, 500)
    return () => clearTimeout(timeout)
  }, [windowState])

  return { windowState, setWindowState }
}