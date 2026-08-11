/**
 * About Modal Component
 *
 * Displays application information including version, description, and license.
 */
import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'

interface AboutModalProps {
  isOpen: boolean
  onClose: () => void
}

export function AboutModal({ isOpen, onClose }: AboutModalProps) {
  const { t } = useTranslation()
  const [version, setVersion] = useState<string>('0.0.1')

  useEffect(() => {
    if (isOpen) {
      invoke<string>('get_app_version')
        .then(setVersion)
        .catch(err => {
          console.error('Failed to get app version:', err)
          setVersion('0.0.1')
        })
    }
  }, [isOpen])

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            onClick={onClose}
            className="fixed inset-0 bg-black/50 z-40"
          />
          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: -20 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: -20 }}
            className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none"
          >
            <div className="pointer-events-auto max-w-sm w-full mx-4 bg-white dark:bg-gray-800 rounded-lg shadow-xl overflow-hidden">
              {/* Header with icon */}
              <div className="bg-gradient-to-br from-primary-500 to-primary-700 px-6 py-8 text-center">
                <div className="w-16 h-16 mx-auto mb-3 bg-white/20 rounded-2xl flex items-center justify-center backdrop-blur-sm">
                  <svg className="w-9 h-9 text-white" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                  </svg>
                </div>
                <h2 className="text-xl font-semibold text-white">{t('app.name')}</h2>
                <p className="text-sm text-primary-100 mt-1">v{version}</p>
              </div>

              {/* Body */}
              <div className="px-6 py-5 space-y-3">
                <p className="text-sm text-gray-600 dark:text-gray-300 text-center">
                  {t('about.description')}
                </p>

                <div className="space-y-2 pt-2 border-t border-gray-100 dark:border-gray-700">
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">{t('about.version')}</span>
                    <span className="text-gray-900 dark:text-gray-100 font-mono">{version}</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">{t('about.license')}</span>
                    <span className="text-gray-900 dark:text-gray-100">MIT</span>
                  </div>
                  <div className="flex justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">{t('about.authors')}</span>
                    <span className="text-gray-900 dark:text-gray-100">Practice Plan</span>
                  </div>
                </div>
              </div>

              {/* Footer */}
              <div className="px-6 py-4 bg-gray-50 dark:bg-gray-900/50 flex justify-end">
                <button
                  onClick={onClose}
                  className="px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors text-sm"
                >
                  {t('about.close')}
                </button>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}

export default AboutModal
