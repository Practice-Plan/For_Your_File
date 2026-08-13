/**
 * Global Hotkey Hook
 *
 * Manages global hotkey functionality for window activation
 */

import { useEffect, useState } from 'react'
import { invoke } from '@tauri-apps/api/core'

/**
 * Hotkey configuration from backend
 */
export interface HotkeyConfig {
  modifiers: string
  key: string
  registered: boolean
}

/**
 * Hook return type
 */
export interface UseGlobalHotkeyReturn {
  /** Current hotkey configuration */
  config: HotkeyConfig | null
  /** Whether hotkey is currently registered */
  isRegistered: boolean
  /** Error message if any */
  error: string | null
  /** Register a new hotkey */
  registerHotkey: (hotkey: string) => Promise<void>
  /** Unregister current hotkey */
  unregisterHotkey: () => Promise<void>
  /** Update hotkey to new combination */
  updateHotkey: (hotkey: string) => Promise<void>
  /** Check if hotkey has a conflict */
  checkConflict: (hotkey: string) => Promise<boolean>
  /** Test if a hotkey can be registered */
  testHotkey: (hotkey: string) => Promise<boolean>
  /** Get suggested alternative hotkeys */
  getSuggestedHotkeys: () => Promise<string[]>
  /** Clear current error */
  clearError: () => void
}

/**
 * Hook for managing global hotkey functionality
 *
 * NOTE: This hook is for configuration management only (register, unregister,
 * update, checkConflict, etc.). The actual window toggle on hotkey-pressed is
 * handled exclusively by the backend listener in lib.rs. If the frontend also
 * toggles the window, the double-toggle cancels out (show then hide = no change).
 */
export function useGlobalHotkey(): UseGlobalHotkeyReturn {
  const [config, setConfig] = useState<HotkeyConfig | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Load initial configuration
  useEffect(() => {
    loadConfig()
  }, [])

  /**
   * Load hotkey configuration from backend
   */
  const loadConfig = async () => {
    try {
      const config = await invoke<HotkeyConfig>('get_hotkey_config')
      setConfig(config)
    } catch (err) {
      console.error('Failed to load hotkey config:', err)
      setError(`Failed to load hotkey configuration: ${err}`)
    }
  }

  /**
   * Register a global hotkey
   */
  const registerHotkey = async (hotkey: string) => {
    try {
      setError(null)
      await invoke('register_global_hotkey', { hotkey })
      await loadConfig()
    } catch (err) {
      const errorMsg = `Failed to register hotkey: ${err}`
      console.error(errorMsg)
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  /**
   * Unregister the global hotkey
   */
  const unregisterHotkey = async () => {
    try {
      setError(null)
      await invoke('unregister_global_hotkey')
      await loadConfig()
    } catch (err) {
      const errorMsg = `Failed to unregister hotkey: ${err}`
      console.error(errorMsg)
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  /**
   * Update hotkey to new combination
   */
  const updateHotkey = async (hotkey: string) => {
    try {
      setError(null)
      await invoke('update_global_hotkey', { hotkey })
      await loadConfig()
    } catch (err) {
      const errorMsg = `Failed to update hotkey: ${err}`
      console.error(errorMsg)
      setError(errorMsg)
      throw new Error(errorMsg)
    }
  }

  /**
   * Check if a hotkey has a conflict
   */
  const checkConflict = async (hotkey: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('check_hotkey_conflict', { hotkey })
    } catch (err) {
      console.error('Failed to check hotkey conflict:', err)
      return false
    }
  }

  /**
   * Test if a hotkey can be registered
   */
  const testHotkey = async (hotkey: string): Promise<boolean> => {
    try {
      return await invoke<boolean>('test_hotkey', { hotkey })
    } catch (err) {
      console.error('Failed to test hotkey:', err)
      return false
    }
  }

  /**
   * Get suggested alternative hotkeys
   */
  const getSuggestedHotkeys = async (): Promise<string[]> => {
    try {
      return await invoke<string[]>('get_suggested_hotkeys')
    } catch (err) {
      console.error('Failed to get suggested hotkeys:', err)
      return ['Alt+Space', 'Ctrl+Space', 'Alt+Q']
    }
  }

  /**
   * Clear current error
   */
  const clearError = () => {
    setError(null)
  }

  return {
    config,
    isRegistered: config?.registered ?? false,
    error,
    registerHotkey,
    unregisterHotkey,
    updateHotkey,
    checkConflict,
    testHotkey,
    getSuggestedHotkeys,
    clearError,
  }
}

/**
 * Parse hotkey string to components
 */
export function parseHotkey(hotkey: string): { modifiers: string[]; key: string } {
  const parts = hotkey.split('+').map(s => s.trim())

  if (parts.length === 0) {
    return { modifiers: [], key: '' }
  }

  const key = parts[parts.length - 1]
  const modifiers = parts.slice(0, -1)

  return { modifiers, key }
}

/**
 * Format hotkey from components
 */
export function formatHotkey(modifiers: string[], key: string): string {
  if (modifiers.length === 0) {
    return key
  }

  return [...modifiers, key].join('+')
}

/**
 * Validate hotkey format
 */
export function validateHotkey(hotkey: string): boolean {
  const validKeys = [
    'Space', 'Enter', 'Tab', 'Escape', 'Backspace', 'Delete', 'Insert',
    'Home', 'End', 'PageUp', 'PageDown',
    'Up', 'Down', 'Left', 'Right',
    'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12',
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M',
    'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z',
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9',
  ]

  const validModifiers = ['Alt', 'Ctrl', 'Shift', 'Win']

  const { modifiers, key } = parseHotkey(hotkey)

  // Validate key
  if (!validKeys.includes(key)) {
    return false
  }

  // Validate modifiers
  for (const mod of modifiers) {
    if (!validModifiers.includes(mod)) {
      return false
    }
  }

  // Must have at least one modifier for global hotkeys
  if (modifiers.length === 0) {
    return false
  }

  return true
}