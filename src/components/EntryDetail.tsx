import { useState, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { useEntry } from '../hooks/useEntry'
import { useModalWithKeyboard } from '../hooks/useModal'
import { TagEditor } from './TagEditor'
import { FrequencyIndicator } from './FrequencyIndicator'
import { DeleteConfirmModal } from './DeleteConfirmModal'
import type { Entry } from '../types'

interface EntryDetailProps {
  entryId: number | null
  onClose: () => void
  onUpdate?: () => void
  onDelete?: () => void
}

export function EntryDetail({ entryId, onClose, onUpdate, onDelete }: EntryDetailProps) {
  const { t } = useTranslation()
  const { entry, loading, error, updateEntry, deleteEntry } = useEntry(entryId)
  const [isEditing, setIsEditing] = useState(false)
  const [editedFields, setEditedFields] = useState<Partial<Entry>>({})
  const deleteModal = useModalWithKeyboard(false)

  useEffect(() => {
    if (entry) {
      setEditedFields({})
      setIsEditing(false)
    }
  }, [entry])

  const handleSave = async () => {
    if (!entry || Object.keys(editedFields).length === 0) return

    try {
      await updateEntry(editedFields)
      setIsEditing(false)
      setEditedFields({})
      onUpdate?.()
    } catch (err) {
      console.error('Failed to save entry:', err)
    }
  }

  const handleCancel = () => {
    setEditedFields({})
    setIsEditing(false)
  }

  const handleDelete = async () => {
    try {
      await deleteEntry()
      deleteModal.close()
      onDelete?.()
      onClose()
    } catch (err) {
      console.error('Failed to delete entry:', err)
    }
  }

  const handleTagsChange = (tags: string[]) => {
    setEditedFields(prev => ({ ...prev, tags: tags.join(',') }))
  }

  const formatTimestamp = (timestamp?: number) => {
    if (!timestamp) return t('entry.never')
    const date = new Date(timestamp * 1000)
    const now = new Date()
    const diffMs = now.getTime() - date.getTime()
    const diffMins = Math.floor(diffMs / 60000)
    const diffHours = Math.floor(diffMins / 60)
    const diffDays = Math.floor(diffHours / 24)

    if (diffMins < 1) return t('entry.justNow')
    if (diffMins < 60) return t('entry.minutesAgo', { count: diffMins })
    if (diffHours < 24) return t('entry.hoursAgo', { count: diffHours })
    if (diffDays < 7) return t('entry.daysAgo', { count: diffDays })
    return date.toLocaleDateString()
  }

  const copyToClipboard = async (text: string) => {
    try {
      await navigator.clipboard.writeText(text)
    } catch (err) {
      console.error('Failed to copy:', err)
    }
  }

  return (
    <>
      <AnimatePresence>
        {entryId !== null && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="fixed inset-0 bg-black/30 z-40"
              onClick={onClose}
            />

            <motion.div
              initial={{ x: '100%' }}
              animate={{ x: 0 }}
              exit={{ x: '100%' }}
              transition={{ type: 'spring', damping: 25, stiffness: 300 }}
              className="fixed top-0 right-0 h-full w-full max-w-xl bg-white dark:bg-dark-bg border-l border-gray-200 dark:border-dark-border shadow-2xl z-50 overflow-y-auto"
            >
              {loading && (
                <div className="flex items-center justify-center h-64">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-500" />
                </div>
              )}

              {error && (
                <div className="p-6">
                  <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-4">
                    <p className="text-red-800 dark:text-red-200">{error}</p>
                  </div>
                </div>
              )}

              {entry && !loading && (
                <>
                  <div className="sticky top-0 bg-white dark:bg-dark-bg border-b border-gray-200 dark:border-dark-border px-6 py-4 z-10">
                    <div className="flex items-center justify-between">
                      <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                        {t('entry.entryDetails')}
                      </h2>
                      <button
                        onClick={onClose}
                        className="p-1 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
                        aria-label={t('entry.close')}
                      >
                        <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 20 20">
                          <path
                            fillRule="evenodd"
                            d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                            clipRule="evenodd"
                          />
                        </svg>
                      </button>
                    </div>
                  </div>

                  <div className="p-6 space-y-6">
                    <div className="space-y-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          {t('entry.targetPath')}
                        </label>
                        <div className="flex gap-2">
                          <input
                            type="text"
                            value={entry.target_path}
                            readOnly
                            className="flex-1 px-3 py-2 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded text-sm text-gray-600 dark:text-gray-400"
                          />
                          <button
                            onClick={() => copyToClipboard(entry.target_path)}
                            className="px-3 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                            title={t('entry.copyToClipboard')}
                          >
                            <svg className="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                              <path d="M8 3a1 1 0 011-1h2a1 1 0 110 2H9a1 1 0 01-1-1z" />
                              <path d="M6 3a2 2 0 00-2 2v11a2 2 0 002 2h8a2 2 0 002-2V5a2 2 0 00-2-2 3 3 0 01-3 3H9a3 3 0 01-3-3z" />
                            </svg>
                          </button>
                        </div>
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          {t('entry.parameters')}
                        </label>
                        {isEditing ? (
                          <input
                            type="text"
                            value={editedFields.parameters ?? entry.parameters ?? ''}
                            onChange={e => setEditedFields({ ...editedFields, parameters: e.target.value })}
                            className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                          />
                        ) : (
                          <div
                            onClick={() => setIsEditing(true)}
                            className="px-3 py-2 bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded text-sm cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                          >
                            {entry.parameters || t('entry.noParameters')}
                          </div>
                        )}
                      </div>

                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          {t('entry.workingDirectory')}
                        </label>
                        {isEditing ? (
                          <input
                            type="text"
                            value={editedFields.working_dir ?? entry.working_dir ?? ''}
                            onChange={e => setEditedFields({ ...editedFields, working_dir: e.target.value })}
                            className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                          />
                        ) : (
                          <div
                            onClick={() => setIsEditing(true)}
                            className="px-3 py-2 bg-gray-50 dark:bg-gray-800/50 border border-gray-200 dark:border-gray-700 rounded text-sm cursor-pointer hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors"
                          >
                            {entry.working_dir || t('entry.notSet')}
                          </div>
                        )}
                      </div>
                    </div>

                    <div className="border-t border-gray-200 dark:border-dark-border pt-6">
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        {t('entry.tags')}
                      </label>
                      <TagEditor
                        initialTags={entry.tags ? entry.tags.split(',').map(t => t.trim()) : []}
                        onChange={handleTagsChange}
                      />
                    </div>

                    <div className="border-t border-gray-200 dark:border-dark-border pt-6">
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                        {t('entry.notes')}
                      </label>
                      <div className="relative">
                        <textarea
                          value={editedFields.notes ?? entry.notes ?? ''}
                          onChange={e => {
                            setEditedFields({ ...editedFields, notes: e.target.value })
                            setIsEditing(true)
                          }}
                          placeholder={t('entry.addNotes')}
                          rows={4}
                          maxLength={500}
                          className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm resize-none"
                        />
                        <div className="absolute bottom-2 right-2 text-xs text-gray-400">
                          {(editedFields.notes ?? entry.notes ?? '').length}/500
                        </div>
                      </div>
                    </div>

                    <div className="border-t border-gray-200 dark:border-dark-border pt-6 space-y-4">
                      <div>
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                          {t('entry.usageFrequency')}
                        </label>
                        <FrequencyIndicator frequency={entry.frequency} />
                      </div>

                      <div className="grid grid-cols-2 gap-4">
                        <div>
                          <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                            {t('entry.lastOpened')}
                          </label>
                          <p className="text-sm text-gray-900 dark:text-gray-100">
                            {formatTimestamp(entry.last_opened)}
                          </p>
                        </div>
                        <div>
                          <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                            {t('entry.created')}
                          </label>
                          <p className="text-sm text-gray-900 dark:text-gray-100">
                            {formatTimestamp(entry.created_at)}
                          </p>
                        </div>
                        <div>
                          <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                            {t('entry.updated')}
                          </label>
                          <p className="text-sm text-gray-900 dark:text-gray-100">
                            {formatTimestamp(entry.updated_at)}
                          </p>
                        </div>
                      </div>
                    </div>

                    <div className="border-t border-gray-200 dark:border-dark-border pt-6 flex justify-between gap-2">
                      <button
                        onClick={() => deleteModal.open()}
                        className="px-4 py-2 text-sm text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 rounded transition-colors"
                      >
                        {t('entry.deleteEntry')}
                      </button>

                      <div className="flex gap-2">
                        {isEditing && (
                          <button
                            onClick={handleCancel}
                            className="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                          >
                            {t('entry.cancel')}
                          </button>
                        )}
                        <button
                          onClick={handleSave}
                          disabled={!isEditing || Object.keys(editedFields).length === 0}
                          className="px-4 py-2 text-sm bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                        >
                          {t('entry.saveChanges')}
                        </button>
                      </div>
                    </div>
                  </div>
                </>
              )}
            </motion.div>
          </>
        )}
      </AnimatePresence>

      <DeleteConfirmModal
        isOpen={deleteModal.isOpen}
        onConfirm={handleDelete}
        onCancel={deleteModal.close}
        title={t('entry.deleteEntry')}
        message={t('entry.deleteConfirm')}
        itemName={entry?.lnk_path}
      />
    </>
  )
}