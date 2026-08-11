import { motion } from 'framer-motion'
import { ExpirationStatus } from '../types'

interface ExpirationIndicatorProps {
  status: ExpirationStatus
  showCountdown?: boolean
  className?: string
}

/**
 * Visual indicator for entry expiration status
 * Shows red badge for expired, yellow for expiring soon
 */
export function ExpirationIndicator({ 
  status, 
  showCountdown = true,
  className = '' 
}: ExpirationIndicatorProps) {
  if (status.type === 'NotExpiring') {
    return null
  }

  if (status.type === 'Expired') {
    return (
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium
          bg-red-100 dark:bg-red-900/30 text-red-700 dark:text-red-400
          border border-red-200 dark:border-red-800 ${className}`}
      >
        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z" />
        </svg>
        <span>Expired</span>
      </motion.div>
    )
  }

  if (status.type === 'ExpiringSoon') {
    const isUrgent = status.days_remaining <= 3
    const colorClass = isUrgent
      ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800'
      : 'bg-orange-100 dark:bg-orange-900/30 text-orange-700 dark:text-orange-400 border-orange-200 dark:border-orange-800'

    return (
      <motion.div
        initial={{ scale: 0 }}
        animate={{ scale: 1 }}
        className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium
          ${colorClass} ${className}`}
      >
        <svg className="w-3 h-3" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        {showCountdown && (
          <span>
            {status.days_remaining === 0 
              ? 'Expires today' 
              : status.days_remaining === 1 
                ? 'Expires tomorrow'
                : `${status.days_remaining} days left`}
          </span>
        )}
      </motion.div>
    )
  }

  return null
}

/**
 * Badge showing countdown to expiration
 */
interface CountdownBadgeProps {
  expiresAt: number
  className?: string
}

export function CountdownBadge({ expiresAt, className = '' }: CountdownBadgeProps) {
  const now = Date.now() / 1000
  const diff = expiresAt - now

  if (diff <= 0) {
    return (
      <span className={`text-xs text-red-600 dark:text-red-400 font-medium ${className}`}>
        Expired
      </span>
    )
  }

  const days = Math.floor(diff / 86400)
  const hours = Math.floor((diff % 86400) / 3600)
  const minutes = Math.floor((diff % 3600) / 60)

  let display: string
  if (days > 0) {
    display = `${days}d ${hours}h`
  } else if (hours > 0) {
    display = `${hours}h ${minutes}m`
  } else {
    display = `${minutes}m`
  }

  const isUrgent = days <= 1
  const colorClass = isUrgent
    ? 'text-red-600 dark:text-red-400'
    : 'text-yellow-600 dark:text-yellow-400'

  return (
    <span className={`text-xs font-medium ${colorClass} ${className}`}>
      {display}
    </span>
  )
}

/**
 * Hover tooltip showing exact expiration date
 */
interface ExpirationTooltipProps {
  expiresAt: number
  children: React.ReactNode
}

export function ExpirationTooltip({ expiresAt, children }: ExpirationTooltipProps) {
  const date = new Date(expiresAt * 1000)
  const formatted = date.toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit'
  })

  return (
    <div className="relative group">
      {children}
      <div className="absolute bottom-full left-1/2 -translate-x-1/2 mb-1 
        px-2 py-1 bg-gray-900 dark:bg-gray-700 text-white text-xs rounded
        opacity-0 group-hover:opacity-100 transition-opacity whitespace-nowrap z-10">
        Expires: {formatted}
        <div className="absolute top-full left-1/2 -translate-x-1/2 border-4 
          border-transparent border-t-gray-900 dark:border-t-gray-700" />
      </div>
    </div>
  )
}