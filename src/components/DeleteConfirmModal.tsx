import { useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'

interface DeleteConfirmModalProps {
  isOpen: boolean
  onConfirm: () => void
  onCancel: () => void
  title?: string
  message?: string
  itemName?: string
}

export function DeleteConfirmModal({
  isOpen,
  onConfirm,
  onCancel,
  title,
  message,
  itemName,
}: DeleteConfirmModalProps) {
  const { t } = useTranslation()

  useEffect(() => {
    if (!isOpen) return

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        onCancel()
      } else if (e.key === 'Enter') {
        onConfirm()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [isOpen, onConfirm, onCancel])

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 z-40"
            onClick={onCancel}
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95, y: 10 }}
            animate={{ opacity: 1, scale: 1, y: 0 }}
            exit={{ opacity: 0, scale: 0.95, y: 10 }}
            transition={{ duration: 0.15 }}
            className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none p-4"
          >
            <div className="pointer-events-auto w-full max-w-md bg-white dark:bg-dark-surface border border-gray-200 dark:border-dark-border rounded-lg shadow-xl overflow-hidden">
              <div className="p-6">
                <h3 className="text-lg font-semibold text-gray-900 dark:text-gray-100 mb-2">
                  {title || t('common.confirmDelete')}
                </h3>

                <p className="text-sm text-gray-600 dark:text-gray-400 mb-1">
                  {message || t('common.deleteConfirmMessage')}
                </p>

                {itemName && (
                  <p className="text-sm font-medium text-gray-900 dark:text-gray-100 mb-4">
                    {itemName}
                  </p>
                )}

                <div className="mt-6 flex justify-end gap-2">
                  <button
                    onClick={onCancel}
                    className="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 text-gray-900 dark:text-gray-100 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                  >
                    {t('common.cancel')}
                  </button>

                  <button
                    onClick={onConfirm}
                    className="px-4 py-2 text-sm bg-red-600 text-white rounded hover:bg-red-700 transition-colors"
                  >
                    {t('common.delete')}
                  </button>
                </div>
              </div>

              <div className="px-6 py-3 bg-gray-50 dark:bg-gray-800/50 border-t border-gray-200 dark:border-dark-border rounded-b-lg">
                <p className="text-xs text-gray-500 dark:text-gray-400">
                  {t('common.deleteShortcutHint')}
                </p>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}
