import { motion } from 'framer-motion'
import {
  highlightText,
  formatRelativeTime,
  formatFrequency,
  parseTags,
  truncatePath,
} from '../utils/search'
import { ExpirationIndicator, CountdownBadge, ExpirationTooltip } from './ExpirationIndicator'
import { ExpirationStatus } from '../types'

interface SearchResultItemProps {
  id: string
  lnkPath: string
  targetPath: string
  tags?: string | null
  notes?: string | null
  frequency: number
  lastOpened: string | null
  expiresAt?: number | null
  query: string
  isSelected?: boolean
  onClick?: () => void
  onDoubleClick?: () => void
}

/**
 * Helper to compute expiration status from timestamp
 */
function computeExpirationStatus(expiresAt: number | null | undefined): ExpirationStatus {
  if (!expiresAt) {
    return { type: 'NotExpiring' }
  }

  const now = Math.floor(Date.now() / 1000)
  
  if (expiresAt < now) {
    return { type: 'Expired', expired_at: expiresAt }
  }

  const daysRemaining = Math.floor((expiresAt - now) / 86400)
  
  if (daysRemaining <= 7) {
    return { type: 'ExpiringSoon', expires_at: expiresAt, days_remaining: daysRemaining }
  }

  return { type: 'NotExpiring' }
}

/**
 * Individual search result item
 * Displays LNK file info with highlighted keywords, tags, frequency, and last opened time
 */
export function SearchResultItem({
  id,
  lnkPath,
  targetPath,
  tags,
  notes,
  frequency,
  lastOpened,
  expiresAt,
  query,
  isSelected = false,
  onClick,
  onDoubleClick,
}: SearchResultItemProps) {
  const tagsArray = parseTags(tags)
  const frequencyInfo = formatFrequency(frequency)
  const expirationStatus = computeExpirationStatus(expiresAt)
  
  // Determine border style based on expiration status
  const borderClass = expirationStatus.type === 'Expired'
    ? 'border-l-2 border-l-red-500'
    : expirationStatus.type === 'ExpiringSoon' && expirationStatus.days_remaining <= 3
      ? 'border-l-2 border-l-yellow-500'
      : ''

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      className={`
        group relative
        px-4 py-3
        bg-white dark:bg-dark-surface
        border-b border-gray-100 dark:border-dark-border
        cursor-pointer
        transition-all duration-200
        ${borderClass}
        ${isSelected
          ? 'bg-primary-50 dark:bg-primary-900/20 border-l-2 border-l-primary-500'
          : 'hover:bg-gray-50 dark:hover:bg-gray-800/50'}
      `}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
      role="option"
      aria-selected={isSelected}
      data-id={id}
    >
      <div className="flex items-start gap-3">
        {/* File icon */}
        <div className="flex-shrink-0 mt-1">
          <div className={`w-8 h-8 flex items-center justify-center rounded ${
            expirationStatus.type === 'Expired'
              ? 'bg-red-100 dark:bg-red-900/30'
              : expirationStatus.type === 'ExpiringSoon'
                ? 'bg-yellow-100 dark:bg-yellow-900/30'
                : 'bg-gray-100 dark:bg-gray-800'
          }`}>
            <svg
              className={`w-5 h-5 ${
                expirationStatus.type === 'Expired'
                  ? 'text-red-600 dark:text-red-400'
                  : expirationStatus.type === 'ExpiringSoon'
                    ? 'text-yellow-600 dark:text-yellow-400'
                    : 'text-gray-600 dark:text-gray-400'
              }`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
          </div>
        </div>

        {/* Content */}
        <div className="flex-1 min-w-0">
          {/* Target name/path - prominent */}
          <div className="mb-1 flex items-center gap-2">
            <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
              {highlightText(targetPath.split(/[\\/]/).pop() || targetPath, query)}
            </h3>
            {/* Expiration indicator badge */}
            {expirationStatus.type !== 'NotExpiring' && (
              <ExpirationIndicator status={expirationStatus} />
            )}
          </div>

          {/* Target path - subtle */}
          <div className="mb-1">
            <p className="text-xs text-gray-500 dark:text-gray-400 truncate">
              {highlightText(truncatePath(targetPath, 80), query)}
            </p>
          </div>

          {/* LNK path - very subtle */}
          <div className="text-xs text-gray-400 dark:text-gray-600 truncate">
            <span className="font-mono">LNK:</span>{' '}
            {highlightText(truncatePath(lnkPath, 80), query)}
          </div>

          {/* Tags - small pills */}
          {tagsArray.length > 0 && (
            <div className="flex gap-1.5 mt-2 flex-wrap">
              {tagsArray.slice(0, 5).map((tag, index) => (
                <span
                  key={index}
                  className="inline-flex items-center px-2 py-0.5 text-xs font-medium rounded-full
                    bg-gray-100 dark:bg-gray-800
                    text-gray-700 dark:text-gray-300
                    border border-gray-200 dark:border-gray-700"
                >
                  {highlightText(tag, query)}
                </span>
              ))}
              {tagsArray.length > 5 && (
                <span className="text-xs text-gray-400 dark:text-gray-600">
                  +{tagsArray.length - 5}
                </span>
              )}
            </div>
          )}

          {/* Notes snippet */}
          {notes && (
            <div className="mt-2 text-xs text-gray-600 dark:text-gray-400 italic truncate">
              {highlightText(notes, query)}
            </div>
          )}
        </div>

        {/* Right side metadata */}
        <div className="flex-shrink-0 text-right">
          {/* Expiration countdown */}
          {expirationStatus.type === 'ExpiringSoon' && (
            <div className="mb-2">
              <ExpirationTooltip expiresAt={expiresAt!}>
                <CountdownBadge expiresAt={expiresAt!} />
              </ExpirationTooltip>
            </div>
          )}
          
          {/* Usage frequency with progress bar */}
          <div className="mb-2">
            <div className="text-xs text-gray-500 dark:text-gray-400 mb-1">
              {frequencyInfo.text}
            </div>
            <div className="w-16 h-1 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              <motion.div
                initial={{ width: 0 }}
                animate={{ width: frequencyInfo.barWidth }}
                transition={{ duration: 0.5, delay: 0.1 }}
                className={`h-full rounded-full ${
                  frequencyInfo.intensity === 0
                    ? 'bg-gray-300 dark:bg-gray-600'
                    : frequencyInfo.intensity === 1
                    ? 'bg-blue-300 dark:bg-blue-700'
                    : frequencyInfo.intensity === 2
                    ? 'bg-blue-400 dark:bg-blue-600'
                    : frequencyInfo.intensity === 3
                    ? 'bg-primary-500'
                    : 'bg-primary-600'
                }`}
              />
            </div>
          </div>

          {/* Last opened time */}
          <div className="text-xs text-gray-500 dark:text-gray-400">
            {formatRelativeTime(lastOpened)}
          </div>
        </div>
      </div>

      {/* Hover action indicator */}
      <motion.div
        initial={false}
        animate={{ opacity: isSelected ? 1 : 0 }}
        className="absolute left-0 top-0 bottom-0 w-1 bg-primary-500"
      />
    </motion.div>
  )
}