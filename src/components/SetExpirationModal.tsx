import { useState } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { Entry } from '../types'

interface SetExpirationModalProps {
  entry: Entry
  isOpen: boolean
  onClose: () => void
  onSuccess?: () => void
}

const PRESET_DAYS = [7, 14, 30, 60, 90]

/**
 * Modal for setting or modifying entry expiration date
 */
export function SetExpirationModal({ 
  entry, 
  isOpen, 
  onClose,
  onSuccess 
}: SetExpirationModalProps) {
  const [selectedDays, setSelectedDays] = useState<number>(30)
  const [customDate, setCustomDate] = useState<string>('')
  const [useCustomDate, setUseCustomDate] = useState(false)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const handlePresetClick = (days: number) => {
    setSelectedDays(days)
    setUseCustomDate(false)
  }

  const handleCustomDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setCustomDate(e.target.value)
    setUseCustomDate(true)
  }

  const handleSetExpiration = async () => {
    try {
      setLoading(true)
      setError(null)

      let expiresAt: number

      if (useCustomDate && customDate) {
        // Parse custom date
        const date = new Date(customDate)
        if (isNaN(date.getTime())) {
          throw new Error('Invalid date format')
        }
        expiresAt = Math.floor(date.getTime() / 1000)
      } else {
        // Use preset days
        const now = Math.floor(Date.now() / 1000)
        expiresAt = now + (selectedDays * 86400)
      }

      await invoke('set_expiration', {
        entryId: entry.id,
        expiresAt
      })

      onSuccess?.()
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleRemoveExpiration = async () => {
    try {
      setLoading(true)
      setError(null)

      await invoke('remove_expiration', { entryId: entry.id })

      onSuccess?.()
      onClose()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const formatCurrentExpiration = () => {
    if (!entry.expires_at) return null
    const date = new Date(entry.expires_at * 1000)
    return date.toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'long',
      day: 'numeric'
    })
  }

  const minDate = new Date()
  minDate.setDate(minDate.getDate() + 1)
  const minDateStr = minDate.toISOString().split('T')[0]

  return (
    <AnimatePresence>
      {isOpen && (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          className="fixed inset-0 z-50 flex items-center justify-center bg-black/50"
          onClick={(e) => e.target === e.currentTarget && onClose()}
        >
          <motion.div
            initial={{ scale: 0.95, opacity: 0 }}
            animate={{ scale: 1, opacity: 1 }}
            exit={{ scale: 0.95, opacity: 0 }}
            className="w-full max-w-md mx-4 bg-white dark:bg-dark-surface rounded-xl shadow-xl overflow-hidden"
          >
            {/* Header */}
            <div className="px-6 py-4 border-b border-gray-200 dark:border-dark-border">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-white">
                Set Expiration
              </h2>
              <p className="text-sm text-gray-500 dark:text-gray-400 mt-1 truncate">
                {entry.lnk_path.split(/[\\/]/).pop() || entry.lnk_path}
              </p>
            </div>

            {/* Content */}
            <div className="px-6 py-4 space-y-4">
              {/* Current expiration */}
              {entry.expires_at && (
                <div className="p-3 bg-blue-50 dark:bg-blue-900/20 rounded-lg">
                  <p className="text-sm text-blue-700 dark:text-blue-400">
                    Current expiration: <strong>{formatCurrentExpiration()}</strong>
                  </p>
                </div>
              )}

              {/* Preset options */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Quick presets
                </label>
                <div className="flex flex-wrap gap-2">
                  {PRESET_DAYS.map((days) => (
                    <button
                      key={days}
                      onClick={() => handlePresetClick(days)}
                      disabled={loading}
                      className={`px-3 py-1.5 text-sm font-medium rounded-lg transition-colors
                        ${!useCustomDate && selectedDays === days
                          ? 'bg-primary-500 text-white'
                          : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                        }`}
                    >
                      {days} days
                    </button>
                  ))}
                </div>
              </div>

              {/* Custom date */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Or set custom date
                </label>
                <input
                  type="date"
                  value={customDate}
                  onChange={handleCustomDateChange}
                  min={minDateStr}
                  disabled={loading}
                  className="w-full px-3 py-2 text-sm bg-gray-50 dark:bg-gray-800 
                    border border-gray-200 dark:border-gray-700 rounded-lg
                    focus:outline-none focus:border-primary-500"
                />
              </div>

              {/* Preview */}
              <div className="p-3 bg-gray-50 dark:bg-gray-800 rounded-lg">
                <p className="text-sm text-gray-600 dark:text-gray-400">
                  Entry will expire on:{' '}
                  <span className="font-medium text-gray-900 dark:text-white">
                    {useCustomDate && customDate
                      ? new Date(customDate).toLocaleDateString('en-US', {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric'
                        })
                      : new Date(Date.now() + selectedDays * 86400 * 1000).toLocaleDateString('en-US', {
                          year: 'numeric',
                          month: 'long',
                          day: 'numeric'
                        })}
                  </span>
                </p>
              </div>

              {/* Error */}
              {error && (
                <div className="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
                  <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
                </div>
              )}
            </div>

            {/* Actions */}
            <div className="px-6 py-4 bg-gray-50 dark:bg-gray-800/50 flex justify-between">
              <div>
                {entry.expires_at && (
                  <button
                    onClick={handleRemoveExpiration}
                    disabled={loading}
                    className="px-4 py-2 text-sm font-medium text-red-600 dark:text-red-400
                      hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg transition-colors"
                  >
                    Remove Expiration
                  </button>
                )}
              </div>
              <div className="flex gap-2">
                <button
                  onClick={onClose}
                  disabled={loading}
                  className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300
                    hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
                >
                  Cancel
                </button>
                <button
                  onClick={handleSetExpiration}
                  disabled={loading}
                  className="px-4 py-2 text-sm font-medium text-white bg-primary-500
                    hover:bg-primary-600 rounded-lg transition-colors disabled:opacity-50"
                >
                  {loading ? 'Setting...' : 'Set Expiration'}
                </button>
              </div>
            </div>
          </motion.div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}