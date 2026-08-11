import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { listen } from '@tauri-apps/api/event'
import { ExpirationNotificationPayload } from '../types'

interface ToastNotification {
  id: string
  type: 'expired' | 'expiring_soon'
  title: string
  message: string
  entryId: number
  entryName: string
}

/**
 * Toast notification component for expiration alerts
 */
export function ExpirationNotification() {
  const [notifications, setNotifications] = useState<ToastNotification[]>([])

  useEffect(() => {
    // Listen for expiration notifications from backend
    const unlisten = listen<ExpirationNotificationPayload>('expiration-notification', (event) => {
      const payload = event.payload
      const notification: ToastNotification = {
        id: `${payload.entry_id}-${Date.now()}`,
        type: payload.status,
        title: payload.status === 'expired' ? 'File Expired' : 'File Expiring Soon',
        message: payload.message,
        entryId: payload.entry_id,
        entryName: payload.entry_name
      }
      
      setNotifications(prev => [...prev, notification])

      // Auto-dismiss after 5 seconds
      setTimeout(() => {
        setNotifications(prev => prev.filter(n => n.id !== notification.id))
      }, 5000)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const dismissNotification = (id: string) => {
    setNotifications(prev => prev.filter(n => n.id !== id))
  }

  return (
    <div className="fixed bottom-4 right-4 z-50 flex flex-col gap-2 max-w-sm">
      <AnimatePresence>
        {notifications.map((notification) => (
          <motion.div
            key={notification.id}
            initial={{ opacity: 0, y: 50, scale: 0.95 }}
            animate={{ opacity: 1, y: 0, scale: 1 }}
            exit={{ opacity: 0, y: 20, scale: 0.95 }}
            className={`relative p-4 rounded-lg shadow-lg border ${
              notification.type === 'expired'
                ? 'bg-red-50 dark:bg-red-900/90 border-red-200 dark:border-red-800'
                : 'bg-yellow-50 dark:bg-yellow-900/90 border-yellow-200 dark:border-yellow-800'
            }`}
          >
            {/* Icon */}
            <div className="flex items-start gap-3">
              <div className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center ${
                notification.type === 'expired'
                  ? 'bg-red-100 dark:bg-red-800'
                  : 'bg-yellow-100 dark:bg-yellow-800'
              }`}>
                {notification.type === 'expired' ? (
                  <svg className="w-5 h-5 text-red-600 dark:text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-1.964-1.333-2.732 0L3.732 16c-.77 1.333.192 3 1.732 3z" />
                  </svg>
                ) : (
                  <svg className="w-5 h-5 text-yellow-600 dark:text-yellow-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                  </svg>
                )}
              </div>

              {/* Content */}
              <div className="flex-1 min-w-0">
                <h4 className={`text-sm font-semibold ${
                  notification.type === 'expired'
                    ? 'text-red-800 dark:text-red-200'
                    : 'text-yellow-800 dark:text-yellow-200'
                }`}>
                  {notification.title}
                </h4>
                <p className={`text-sm mt-1 ${
                  notification.type === 'expired'
                    ? 'text-red-700 dark:text-red-300'
                    : 'text-yellow-700 dark:text-yellow-300'
                }`}>
                  {notification.message}
                </p>
              </div>

              {/* Close button */}
              <button
                onClick={() => dismissNotification(notification.id)}
                className={`flex-shrink-0 p-1 rounded transition-colors ${
                  notification.type === 'expired'
                    ? 'hover:bg-red-100 dark:hover:bg-red-800 text-red-600 dark:text-red-400'
                    : 'hover:bg-yellow-100 dark:hover:bg-yellow-800 text-yellow-600 dark:text-yellow-400'
                }`}
              >
                <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Progress bar */}
            <motion.div
              initial={{ width: '100%' }}
              animate={{ width: '0%' }}
              transition={{ duration: 5, ease: 'linear' }}
              className={`absolute bottom-0 left-0 h-1 rounded-b-lg ${
                notification.type === 'expired'
                  ? 'bg-red-400 dark:bg-red-600'
                  : 'bg-yellow-400 dark:bg-yellow-600'
              }`}
            />
          </motion.div>
        ))}
      </AnimatePresence>
    </div>
  )
}

/**
 * Hook for managing expiration notifications
 */
export function useExpirationNotifications() {
  const [hasUnread, setHasUnread] = useState(false)
  const [notificationCount, setNotificationCount] = useState(0)

  useEffect(() => {
    const unlisten = listen<ExpirationNotificationPayload>('expiration-notification', () => {
      setHasUnread(true)
      setNotificationCount(prev => prev + 1)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [])

  const markAsRead = () => {
    setHasUnread(false)
    setNotificationCount(0)
  }

  return { hasUnread, notificationCount, markAsRead }
}