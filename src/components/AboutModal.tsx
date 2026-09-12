/**
 * About Modal Component
 *
 * Displays application information including version, description, license,
 * project URL, and author URL.
 */
import { useState, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'

const PROJECT_URL = 'https://github.com/Practice-Plan/For_Your_File'
const AUTHOR_URL = 'https://github.com/Practice-Plan'

interface AboutModalProps {
  isOpen: boolean
  onClose: () => void
}

export function AboutModal({ isOpen, onClose }: AboutModalProps) {
  const { t } = useTranslation()
  const [version, setVersion] = useState<string>('0.0.4')

  useEffect(() => {
    if (isOpen) {
      invoke<string>('get_app_version')
        .then(setVersion)
        .catch(err => {
          console.error('Failed to get app version:', err)
          setVersion('0.0.4')
        })
    }
  }, [isOpen])

  const handleOpenUrl = useCallback((url: string) => {
    invoke('open_url', { url }).catch(err => {
      console.error('Failed to open URL:', err)
    })
  }, [])

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
            <div className="pointer-events-auto max-w-xl w-full mx-6 bg-white dark:bg-gray-800 rounded-xl shadow-2xl overflow-hidden">
              {/* Banner header - clean image, no overlay text/images */}
              <div className="relative overflow-hidden">
                <img
                  src="/banner.png"
                  alt=""
                  className="w-full h-auto object-contain max-h-52"
                  onError={(e) => {
                    // Fallback to SVG or gradient if PNG fails
                    const target = e.currentTarget as HTMLImageElement
                    if (!target.src.endsWith('.svg')) {
                      target.src = '/banner.svg'
                    } else {
                      // Final fallback: apply gradient via inline style
                      target.style.display = 'none'
                      target.parentElement!.style.background = 'linear-gradient(to top right, #3b82f6, #f97316)'
                      target.parentElement!.style.minHeight = '8rem'
                    }
                  }}
                />
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
                    <span className="text-gray-900 dark:text-gray-100">GPL-3.0</span>
                  </div>
                  <div className="flex justify-between text-sm items-center">
                    <span className="text-gray-500 dark:text-gray-400">{t('about.authors')}</span>
                    <button
                      onClick={() => handleOpenUrl(AUTHOR_URL)}
                      className="text-primary-500 hover:text-primary-600 dark:text-primary-400 dark:hover:text-primary-300 hover:underline transition-colors"
                    >
                      Practice Plan
                    </button>
                  </div>
                  <div className="flex justify-between text-sm items-center">
                    <span className="text-gray-500 dark:text-gray-400">{t('about.projectUrl')}</span>
                    <button
                      onClick={() => handleOpenUrl(PROJECT_URL)}
                      className="text-primary-500 hover:text-primary-600 dark:text-primary-400 dark:hover:text-primary-300 hover:underline transition-colors truncate max-w-[200px]"
                    >
                      {PROJECT_URL}
                    </button>
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
