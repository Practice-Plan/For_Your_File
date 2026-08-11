/**
 * Interface Shortcut Settings Component
 *
 * Allows users to view and customize keyboard shortcuts for different interfaces.
 * Shortcuts are stored in localStorage and read by the useKeyboardShortcuts hook.
 * This component is displayed exclusively within the Settings interface (Task 3).
 */

import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'

interface ShortcutConfig {
  id: string
  labelKey: string
  defaultKeys: string
  enabled: boolean
}

const STORAGE_KEY = 'interfaceShortcuts'

const DEFAULT_SHORTCUTS: ShortcutConfig[] = [
  { id: 'focusSearch', labelKey: 'settings.focusSearch', defaultKeys: 'Ctrl+K', enabled: true },
  { id: 'clearSearch', labelKey: 'settings.clearSearch', defaultKeys: 'Escape', enabled: true },
  { id: 'navigateUp', labelKey: 'settings.navigateUp', defaultKeys: 'ArrowUp', enabled: true },
  { id: 'navigateDown', labelKey: 'settings.navigateDown', defaultKeys: 'ArrowDown', enabled: true },
  { id: 'openSelected', labelKey: 'settings.openSelected', defaultKeys: 'Enter', enabled: true },
]

function loadShortcuts(): ShortcutConfig[] {
  try {
    const stored = localStorage.getItem(STORAGE_KEY)
    if (stored) {
      const parsed = JSON.parse(stored)
      return DEFAULT_SHORTCUTS.map(def => {
        const saved = parsed.find((s: ShortcutConfig) => s.id === def.id)
        return saved || def
      })
    }
  } catch {
    // Fall through to defaults
  }
  return DEFAULT_SHORTCUTS
}

function saveShortcuts(shortcuts: ShortcutConfig[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(shortcuts))
  } catch {
    // Ignore storage errors
  }
}

export function InterfaceShortcutSettings() {
  const { t } = useTranslation()
  const [shortcuts, setShortcuts] = useState<ShortcutConfig[]>(DEFAULT_SHORTCUTS)
  const [capturingId, setCapturingId] = useState<string | null>(null)

  useEffect(() => {
    setShortcuts(loadShortcuts())
  }, [])

  const updateShortcut = useCallback((id: string, updates: Partial<ShortcutConfig>) => {
    setShortcuts(prev => {
      const next = prev.map(s => (s.id === id ? { ...s, ...updates } : s))
      saveShortcuts(next)
      return next
    })
  }, [])

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!capturingId) return
      e.preventDefault()
      e.stopPropagation()

      const key = e.key

      // Ignore modifier-only presses
      if (['Control', 'Shift', 'Alt', 'Meta'].includes(key)) return

      const parts: string[] = []
      if (e.ctrlKey) parts.push('Ctrl')
      if (e.altKey) parts.push('Alt')
      if (e.shiftKey) parts.push('Shift')
      if (e.metaKey) parts.push('Win')

      // Map special keys to readable names
      const keyMap: Record<string, string> = {
        ' ': 'Space',
        ArrowUp: 'ArrowUp',
        ArrowDown: 'ArrowDown',
        ArrowLeft: 'ArrowLeft',
        ArrowRight: 'ArrowRight',
        Escape: 'Escape',
        Enter: 'Enter',
        Tab: 'Tab',
        Backspace: 'Backspace',
        Delete: 'Delete',
      }

      const mappedKey = keyMap[key] || key.toUpperCase()
      parts.push(mappedKey)

      updateShortcut(capturingId, { defaultKeys: parts.join('+') })
      setCapturingId(null)
    },
    [capturingId, updateShortcut]
  )

  useEffect(() => {
    if (capturingId) {
      window.addEventListener('keydown', handleKeyDown, true)
      return () => window.removeEventListener('keydown', handleKeyDown, true)
    }
  }, [capturingId, handleKeyDown])

  const resetAll = () => {
    setShortcuts(DEFAULT_SHORTCUTS)
    saveShortcuts(DEFAULT_SHORTCUTS)
  }

  return (
    <div className="p-6 space-y-4">
      <div>
        <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-2">
          {t('settings.interfaceShortcuts')}
        </h3>
        <p className="text-sm text-gray-500 dark:text-gray-400 mb-4">
          {t('settings.interfaceShortcutsDescription')}
        </p>
      </div>

      {/* Shortcut list */}
      <div className="space-y-2">
        {shortcuts.map(shortcut => (
          <div
            key={shortcut.id}
            className="flex items-center justify-between px-4 py-3 bg-gray-50 dark:bg-gray-700/50 rounded-lg"
          >
            <div className="flex items-center gap-3">
              {/* Enable/disable toggle */}
              <button
                onClick={() => updateShortcut(shortcut.id, { enabled: !shortcut.enabled })}
                className={`relative inline-flex h-5 w-9 items-center rounded-full transition-colors ${
                  shortcut.enabled ? 'bg-primary-500' : 'bg-gray-300 dark:bg-gray-600'
                }`}
                role="switch"
                aria-checked={shortcut.enabled}
              >
                <span
                  className={`inline-block h-3 w-3 transform rounded-full bg-white transition-transform ${
                    shortcut.enabled ? 'translate-x-5' : 'translate-x-1'
                  }`}
                />
              </button>
              <span className={`text-sm ${shortcut.enabled ? 'text-gray-900 dark:text-gray-100' : 'text-gray-400 dark:text-gray-500'}`}>
                {t(shortcut.labelKey)}
              </span>
            </div>
            <div className="flex items-center gap-2">
              {capturingId === shortcut.id ? (
                <span className="text-xs text-primary-600 dark:text-primary-400 animate-pulse">
                  {t('settings.pressKeys')}
                </span>
              ) : (
                <kbd className="px-2 py-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs font-mono text-gray-700 dark:text-gray-300">
                  {shortcut.defaultKeys}
                </kbd>
              )}
              <button
                onClick={() => setCapturingId(capturingId === shortcut.id ? null : shortcut.id)}
                disabled={!shortcut.enabled}
                className="px-2 py-1 text-xs bg-gray-200 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-300 dark:hover:bg-gray-500 transition-colors disabled:opacity-40 disabled:cursor-not-allowed"
              >
                {capturingId === shortcut.id ? t('settings.cancel') : t('settings.changeHotkey')}
              </button>
            </div>
          </div>
        ))}
      </div>

      {/* Reset button */}
      <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
        <button
          onClick={resetAll}
          className="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors text-sm"
        >
          {t('settings.resetToDefault')}
        </button>
      </div>

      {/* Info box */}
      <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
        <p className="text-sm text-blue-800 dark:text-blue-300">
          {t('settings.shortcutsInfo')}
        </p>
      </div>
    </div>
  )
}

export default InterfaceShortcutSettings
