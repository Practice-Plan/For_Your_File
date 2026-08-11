import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { Entry, ExpirationCounts, ExpiringSoonEntry } from '../types'
import { CountdownBadge } from './ExpirationIndicator'

interface ExpiredEntriesListProps {
  onExtend?: (entryId: number, days: number) => void
  onDelete?: (entryId: number) => void
  onOpen?: (entry: Entry) => void
  className?: string
}

/**
 * List of expired and expiring entries with action buttons
 */
export function ExpiredEntriesList({ 
  onExtend, 
  onDelete, 
  onOpen,
  className = '' 
}: ExpiredEntriesListProps) {
  const [expiredEntries, setExpiredEntries] = useState<Entry[]>([])
  const [expiringSoonEntries, setExpiringSoonEntries] = useState<ExpiringSoonEntry[]>([])
  const [counts, setCounts] = useState<ExpirationCounts>({ expired: 0, expiring_soon: 0 })
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [showAll, setShowAll] = useState(false)

  useEffect(() => {
    loadData()
  }, [])

  const loadData = async () => {
    try {
      setLoading(true)
      setError(null)

      const [expired, expiringSoon, countsResult] = await Promise.all([
        invoke<Entry[]>('check_expired_entries'),
        invoke<ExpiringSoonEntry[]>('get_expiring_soon', { warningDays: 7 }),
        invoke<ExpirationCounts>('get_expiration_counts')
      ])

      setExpiredEntries(expired)
      setExpiringSoonEntries(expiringSoon)
      setCounts(countsResult)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setLoading(false)
    }
  }

  const handleExtend = async (entryId: number, days: number) => {
    try {
      await invoke('extend_expiration', { entryId, days })
      await loadData()
      onExtend?.(entryId, days)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  const handleDelete = async (entryId: number) => {
    try {
      await invoke('delete_expired_entries')
      await loadData()
      onDelete?.(entryId)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  const handleRemoveExpiration = async (entryId: number) => {
    try {
      await invoke('remove_expiration', { entryId })
      await loadData()
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    }
  }

  if (loading) {
    return (
      <div className={`flex items-center justify-center py-8 ${className}`}>
        <div className="animate-spin rounded-full h-6 w-6 border-b-2 border-primary-500" />
      </div>
    )
  }

  if (counts.expired === 0 && counts.expiring_soon === 0) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <div className="text-4xl mb-2">✅</div>
        <p className="text-sm text-gray-500 dark:text-gray-400">
          No expired or expiring entries
        </p>
      </div>
    )
  }

  return (
    <div className={className}>
      {error && (
        <div className="mb-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-700 dark:text-red-400">{error}</p>
        </div>
      )}

      {/* Summary */}
      <div className="flex gap-4 mb-4">
        {counts.expired > 0 && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-red-100 dark:bg-red-900/30 rounded-full">
            <span className="text-sm font-medium text-red-700 dark:text-red-400">
              {counts.expired} expired
            </span>
          </div>
        )}
        {counts.expiring_soon > 0 && (
          <div className="flex items-center gap-2 px-3 py-1.5 bg-yellow-100 dark:bg-yellow-900/30 rounded-full">
            <span className="text-sm font-medium text-yellow-700 dark:text-yellow-400">
              {counts.expiring_soon} expiring soon
            </span>
          </div>
        )}
      </div>

      {/* Expired Entries */}
      <AnimatePresence>
        {expiredEntries.length > 0 && (
          <motion.section
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
            className="mb-6"
          >
            <h3 className="text-sm font-medium text-red-700 dark:text-red-400 mb-2 flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              Expired
            </h3>
            <div className="space-y-2">
              {expiredEntries.slice(0, showAll ? undefined : 5).map((entry) => (
                <motion.div
                  key={entry.id}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="flex items-center justify-between p-3 bg-red-50 dark:bg-red-900/10 
                    border border-red-200 dark:border-red-800 rounded-lg"
                >
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                      {entry.lnk_path.split(/[\\/]/).pop() || entry.lnk_path}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {entry.target_path}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleExtend(entry.id!, 7)}
                      className="px-2 py-1 text-xs font-medium text-primary-600 dark:text-primary-400
                        hover:bg-primary-100 dark:hover:bg-primary-900/30 rounded transition-colors"
                    >
                      Extend 7 days
                    </button>
                    <button
                      onClick={() => handleRemoveExpiration(entry.id!)}
                      className="px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-400
                        hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
                    >
                      Dismiss
                    </button>
                    <button
                      onClick={() => handleDelete(entry.id!)}
                      className="px-2 py-1 text-xs font-medium text-red-600 dark:text-red-400
                        hover:bg-red-100 dark:hover:bg-red-900/30 rounded transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.section>
        )}
      </AnimatePresence>

      {/* Expiring Soon */}
      <AnimatePresence>
        {expiringSoonEntries.length > 0 && (
          <motion.section
            initial={{ opacity: 0, height: 0 }}
            animate={{ opacity: 1, height: 'auto' }}
            exit={{ opacity: 0, height: 0 }}
          >
            <h3 className="text-sm font-medium text-yellow-700 dark:text-yellow-400 mb-2 flex items-center gap-2">
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              Expiring Soon
            </h3>
            <div className="space-y-2">
              {expiringSoonEntries.slice(0, showAll ? undefined : 5).map(({ entry }) => (
                <motion.div
                  key={entry.id}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  className="flex items-center justify-between p-3 bg-yellow-50 dark:bg-yellow-900/10 
                    border border-yellow-200 dark:border-yellow-800 rounded-lg"
                >
                  <div className="flex-1 min-w-0">
                    <div className="flex items-center gap-2">
                      <p className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                        {entry.lnk_path.split(/[\\/]/).pop() || entry.lnk_path}
                      </p>
                      <CountdownBadge expiresAt={entry.expires_at!} />
                    </div>
                    <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
                      {entry.target_path}
                    </p>
                  </div>
                  <div className="flex items-center gap-2">
                    <button
                      onClick={() => handleExtend(entry.id!, 7)}
                      className="px-2 py-1 text-xs font-medium text-primary-600 dark:text-primary-400
                        hover:bg-primary-100 dark:hover:bg-primary-900/30 rounded transition-colors"
                    >
                      Extend 7 days
                    </button>
                    <button
                      onClick={() => onOpen?.(entry)}
                      className="px-2 py-1 text-xs font-medium text-gray-600 dark:text-gray-400
                        hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
                    >
                      Open
                    </button>
                  </div>
                </motion.div>
              ))}
            </div>
          </motion.section>
        )}
      </AnimatePresence>

      {/* Show More Button */}
      {(expiredEntries.length > 5 || expiringSoonEntries.length > 5) && (
        <button
          onClick={() => setShowAll(!showAll)}
          className="mt-3 text-sm text-primary-600 dark:text-primary-400 hover:underline"
        >
          {showAll ? 'Show less' : 'Show all'}
        </button>
      )}
    </div>
  )
}