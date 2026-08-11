/**
 * Hotkey Settings Component
 *
 * UI for customizing global hotkey for window activation
 */

import React, { useState, useEffect, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import {
  useGlobalHotkey,
  formatHotkey,
  validateHotkey,
} from '../hooks/useGlobalHotkey'

/**
 * Hotkey Settings Component Props
 */
export interface HotkeySettingsProps {
  /** Optional className for styling */
  className?: string
}

/**
 * Hotkey Settings Component
 */
export const HotkeySettings: React.FC<HotkeySettingsProps> = ({ className = '' }) => {
  const { t } = useTranslation()
  
  const {
    config,
    isRegistered,
    error,
    updateHotkey,
    checkConflict,
    getSuggestedHotkeys,
    clearError,
  } = useGlobalHotkey()

  const [isCapturing, setIsCapturing] = useState(false)
  const [tempHotkey, setTempHotkey] = useState<string[]>([])
  const [hasConflict, setHasConflict] = useState(false)
  const [isValid, setIsValid] = useState(true)
  const [suggestions, setSuggestions] = useState<string[]>([])
  const [isSaving, setIsSaving] = useState(false)
  const [showSuccess, setShowSuccess] = useState(false)

  const inputRef = useRef<HTMLInputElement>(null)

  // Load suggestions on mount
  useEffect(() => {
    loadSuggestions()
  }, [])

  // Check conflict when temp hotkey changes
  useEffect(() => {
    if (tempHotkey.length > 1) {
      const hotkeyStr = formatHotkey(tempHotkey.slice(0, -1), tempHotkey[tempHotkey.length - 1])
      checkConflict(hotkeyStr).then(setHasConflict)
      setIsValid(validateHotkey(hotkeyStr))
    }
  }, [tempHotkey])

  /**
   * Load suggested hotkeys
   */
  const loadSuggestions = async () => {
    const suggested = await getSuggestedHotkeys()
    setSuggestions(suggested)
  }

  /**
   * Handle keyboard capture
   */
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!isCapturing) return

      e.preventDefault()
      e.stopPropagation()

      const key = e.key

      // Handle special keys
      const keyMap: Record<string, string> = {
        ' ': 'Space',
        Space: 'Space',
        ArrowUp: 'Up',
        ArrowDown: 'Down',
        ArrowLeft: 'Left',
        ArrowRight: 'Right',
        Escape: 'Escape',
        Enter: 'Enter',
        Tab: 'Tab',
        Backspace: 'Backspace',
        Delete: 'Delete',
        Insert: 'Insert',
        Home: 'Home',
        End: 'End',
        PageUp: 'PageUp',
        PageDown: 'PageDown',
      }

      const mappedKey = keyMap[key] || key.toUpperCase()

      // Add modifiers
      const modifiers: string[] = []
      if (e.altKey) modifiers.push('Alt')
      if (e.ctrlKey) modifiers.push('Ctrl')
      if (e.shiftKey) modifiers.push('Shift')
      if (e.metaKey) modifiers.push('Win')

      // Only include valid keys
      const validKeys = [
        'Space', 'Enter', 'Tab', 'Escape', 'Backspace', 'Delete', 'Insert',
        'Home', 'End', 'PageUp', 'PageDown',
        'Up', 'Down', 'Left', 'Right',
        'F1', 'F2', 'F3', 'F4', 'F5', 'F6', 'F7', 'F8', 'F9', 'F10', 'F11', 'F12',
      ]

      const isValidKey = validKeys.includes(mappedKey) || /^[A-Z0-9]$/.test(mappedKey)

      if (isValidKey) {
        setTempHotkey([...modifiers, mappedKey])
      }
    },
    [isCapturing]
  )

  // Setup keyboard listener
  useEffect(() => {
    if (isCapturing) {
      window.addEventListener('keydown', handleKeyDown)
    } else {
      window.removeEventListener('keydown', handleKeyDown)
    }

    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [isCapturing, handleKeyDown])

  /**
   * Start capturing hotkey
   */
  const startCapture = () => {
    setIsCapturing(true)
    setTempHotkey([])
    setHasConflict(false)
    clearError()
    inputRef.current?.focus()
  }

  /**
   * Cancel capturing
   */
  const cancelCapture = () => {
    setIsCapturing(false)
    setTempHotkey([])
    setHasConflict(false)
  }

  /**
   * Save new hotkey
   */
  const saveHotkey = async () => {
    if (tempHotkey.length < 2 || hasConflict || !isValid) return

    const hotkeyStr = formatHotkey(
      tempHotkey.slice(0, -1),
      tempHotkey[tempHotkey.length - 1]
    )

    setIsSaving(true)
    try {
      await updateHotkey(hotkeyStr)
      setIsCapturing(false)
      setTempHotkey([])
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 2000)
    } catch (err) {
      console.error('Failed to save hotkey:', err)
    } finally {
      setIsSaving(false)
    }
  }

  /**
   * Reset to default hotkey
   */
  const resetToDefault = async () => {
    setIsSaving(true)
    try {
      await updateHotkey('Alt+Space')
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 2000)
    } catch (err) {
      console.error('Failed to reset hotkey:', err)
    } finally {
      setIsSaving(false)
    }
  }

  /**
   * Apply suggestion
   */
  const applySuggestion = async (suggestion: string) => {
    setIsSaving(true)
    try {
      await updateHotkey(suggestion)
      setShowSuccess(true)
      setTimeout(() => setShowSuccess(false), 2000)
    } catch (err) {
      console.error('Failed to apply suggestion:', err)
    } finally {
      setIsSaving(false)
    }
  }

  const currentHotkey = config
    ? formatHotkey([config.modifiers], config.key)
    : 'Alt+Space'

  return (
    <div className={`p-6 bg-white dark:bg-gray-800 rounded-lg shadow ${className}`}>
      <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
        {t('settings.hotkey')}
      </h3>

      <div className="space-y-4">
        {/* Current Hotkey Display */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('settings.currentHotkey')}
          </label>
          <div className="flex items-center gap-3">
            <div className="flex-1 px-4 py-2 bg-gray-100 dark:bg-gray-700 rounded text-lg font-mono">
              {currentHotkey}
            </div>
            <span
              className={`px-3 py-1 rounded text-sm font-medium ${
                isRegistered
                  ? 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
                  : 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
              }`}
            >
              {isRegistered ? t('settings.hotkeyActive') : t('settings.hotkeyInactive')}
            </span>
          </div>
        </div>

        {/* Hotkey Input */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('settings.setNewHotkey')}
          </label>
          <div className="flex items-center gap-2">
            <input
              ref={inputRef}
              type="text"
              readOnly
              value={tempHotkey.length > 0 ? tempHotkey.join('+') : t('settings.hotkeyPlaceholder')}
              onClick={startCapture}
              className={`flex-1 px-4 py-2 bg-white dark:bg-gray-700 border rounded focus:outline-none focus:ring-2 ${
                isCapturing
                  ? 'border-primary-500 ring-2 ring-primary-500'
                  : 'border-gray-300 dark:border-gray-600'
              } ${hasConflict ? 'border-red-500' : ''}`}
              placeholder={t('settings.hotkeyPlaceholder')}
            />
            {isCapturing ? (
              <div className="flex gap-2">
                <button
                  onClick={saveHotkey}
                  disabled={tempHotkey.length < 2 || hasConflict || !isValid || isSaving}
                  className="px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                  {isSaving ? t('status.saving') : t('settings.save')}
                </button>
                <button
                  onClick={cancelCapture}
                  disabled={isSaving}
                  className="px-4 py-2 bg-gray-300 dark:bg-gray-600 text-gray-700 dark:text-gray-200 rounded hover:bg-gray-400 dark:hover:bg-gray-500 transition-colors"
                >
                  {t('settings.cancel')}
                </button>
              </div>
            ) : (
              <button
                onClick={startCapture}
                className="px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors"
              >
                {t('settings.changeHotkey')}
              </button>
            )}
          </div>

          {/* Validation Messages */}
          <AnimatePresence>
            {hasConflict && (
              <motion.p
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="mt-2 text-sm text-red-600 dark:text-red-400"
              >
                ⚠️ This hotkey is already registered by another application
              </motion.p>
            )}

            {!isValid && tempHotkey.length > 0 && (
              <motion.p
                initial={{ opacity: 0, y: -10 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -10 }}
                className="mt-2 text-sm text-red-600 dark:text-red-400"
              >
                ⚠️ Invalid hotkey combination. Must include at least one modifier (Alt, Ctrl, Shift, Win)
              </motion.p>
            )}
          </AnimatePresence>
        </div>

        {/* Suggestions */}
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
            {t('settings.suggestedHotkeys')}
          </label>
          <div className="flex flex-wrap gap-2">
            {suggestions.map(suggestion => (
              <button
                key={suggestion}
                onClick={() => applySuggestion(suggestion)}
                disabled={isSaving}
                className="px-3 py-1 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 text-sm font-mono transition-colors disabled:opacity-50"
              >
                {suggestion}
              </button>
            ))}
          </div>
        </div>

        {/* Reset Button */}
        <div className="pt-4 border-t border-gray-200 dark:border-gray-700">
          <button
            onClick={resetToDefault}
            disabled={isSaving}
            className="px-4 py-2 bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300 rounded hover:bg-gray-200 dark:hover:bg-gray-600 transition-colors"
          >
            {t('settings.resetToDefault')}
          </button>
        </div>

        {/* Error Display */}
        <AnimatePresence>
          {error && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="p-3 bg-red-100 dark:bg-red-900 border border-red-400 dark:border-red-700 rounded"
            >
              <p className="text-sm text-red-700 dark:text-red-200">{error}</p>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Success Message */}
        <AnimatePresence>
          {showSuccess && (
            <motion.div
              initial={{ opacity: 0, y: -10 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -10 }}
              className="p-3 bg-green-100 dark:bg-green-900 border border-green-400 dark:border-green-700 rounded"
            >
              <p className="text-sm text-green-700 dark:text-green-200">
                ✓ {t('settings.hotkeyUpdated')}
              </p>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Instructions */}
        <div className="mt-4 p-4 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
          <h4 className="text-sm font-medium text-blue-900 dark:text-blue-200 mb-2">
            {t('settings.howToUse')}
          </h4>
          <ul className="text-sm text-blue-800 dark:text-blue-300 space-y-1">
            <li>• {t('settings.hotkeyInstructions1')}</li>
            <li>• {t('settings.hotkeyInstructions2')}</li>
            <li>• {t('settings.hotkeyInstructions3')}</li>
            <li>• {t('settings.hotkeyInstructions4')}</li>
          </ul>
        </div>
      </div>
    </div>
  )
}

export default HotkeySettings