import { useState, useRef, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { useCreateEntry } from '../hooks/useEntry'
import { TagEditor } from './TagEditor'
import type { Entry, ParsedLnkProperties } from '../types'

interface AddEntryModalProps {
  isOpen: boolean
  onClose: () => void
  onCreated?: (entry: Entry) => void
}

export function AddEntryModal({ isOpen, onClose, onCreated }: AddEntryModalProps) {
  const { t } = useTranslation()
  const [lnkPath, setLnkPath] = useState('')
  const [targetPath, setTargetPath] = useState('')
  const [parameters, setParameters] = useState('')
  const [workingDir, setWorkingDir] = useState('')
  const [description, setDescription] = useState('')
  const [notes, setNotes] = useState('')
  const [tags, setTags] = useState<string[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const [isParsing, setIsParsing] = useState(false)
  const [parseStatus, setParseStatus] = useState<'idle' | 'success' | 'error'>('idle')

  const { createEntry, loading, error } = useCreateEntry()
  const fileInputRef = useRef<HTMLInputElement>(null)

  const resetForm = () => {
    setLnkPath('')
    setTargetPath('')
    setParameters('')
    setWorkingDir('')
    setDescription('')
    setNotes('')
    setTags([])
    setParseStatus('idle')
  }

  /**
   * Parse a .lnk file and auto-fill form fields with the extracted properties.
   * This implements the auto-completion feature (Task 6).
   */
  const parseAndAutoFill = useCallback(async (filePath: string) => {
    if (!filePath) return

    setIsParsing(true)
    setParseStatus('idle')

    try {
      const props = await invoke<ParsedLnkProperties>('parse_lnk_file', { path: filePath })

      // Auto-fill fields that are empty (don't overwrite user edits)
      if (props.target_path) setTargetPath(props.target_path)
      if (props.arguments) setParameters(props.arguments)
      if (props.working_directory) setWorkingDir(props.working_directory)
      if (props.description) setDescription(props.description)

      setParseStatus('success')
    } catch (err) {
      console.error('Failed to parse LNK file:', err)
      setParseStatus('error')
    } finally {
      setIsParsing(false)
    }
  }, [])

  const handleBrowseLnk = () => {
    fileInputRef.current?.click()
  }

  const handleBrowseTarget = () => {
    const input = window.prompt(t('addEntry.enterTargetPath'))
    if (input) setTargetPath(input)
  }

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)

    const files = Array.from(e.dataTransfer.files)
    const lnkFile = files.find(f => f.name.endsWith('.lnk'))

    if (lnkFile) {
      // In Tauri, file objects have a path property
      const filePath = (lnkFile as any).path || lnkFile.name
      setLnkPath(filePath)
      // Auto-parse the dropped .lnk file to fill in fields
      parseAndAutoFill(filePath)
    }
  }

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(true)
  }

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault()
    setIsDragging(false)
  }

  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0]
    if (file) {
      const filePath = (file as any).path || file.name
      setLnkPath(filePath)
      // Auto-parse the selected .lnk file to fill in fields
      parseAndAutoFill(filePath)
    }
    // Reset the input so the same file can be selected again
    e.target.value = ''
  }

  const handleCreate = async () => {
    // Only targetPath is required; lnkPath is optional (Task 7)
    if (!targetPath) return

    try {
      const entry = await createEntry({
        lnk_path: lnkPath || '',
        target_path: targetPath,
        target_type: { type: 'File', path: targetPath },
        parameters: parameters || undefined,
        working_dir: workingDir || undefined,
        description: description || undefined,
        notes: notes || undefined,
        tags: tags.length > 0 ? tags.join(',') : undefined,
        frequency: 0,
      })

      onCreated?.(entry)
      resetForm()
      onClose()
    } catch (err) {
      console.error('Failed to create entry:', err)
    }
  }

  // Only targetPath is required; lnk file is optional (Task 7)
  const isFormValid = targetPath.trim() !== ''

  return (
    <AnimatePresence>
      {isOpen && (
        <>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 bg-black/50 z-40"
            onClick={onClose}
          />

          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-full max-w-2xl max-h-[85vh] bg-white dark:bg-dark-surface border border-gray-200 dark:border-dark-border rounded-lg shadow-2xl z-50 overflow-hidden flex flex-col"
          >
            <div className="px-6 py-4 border-b border-gray-200 dark:border-dark-border">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                {t('addEntry.title')}
              </h2>
            </div>

            <div className="flex-1 overflow-y-auto p-6">
              {error && (
                <div className="mb-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3">
                  <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
                </div>
              )}

              <div className="space-y-6">
                {/* LNK File Upload (optional) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                    {t('addEntry.lnkFileOptional')}
                  </label>
                  <div
                    onDrop={handleDrop}
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    className={`relative border-2 border-dashed rounded-lg p-6 transition-colors ${
                      isDragging
                        ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                        : 'border-gray-300 dark:border-gray-600'
                    }`}
                  >
                    <input
                      ref={fileInputRef}
                      type="file"
                      accept=".lnk"
                      onChange={handleFileSelect}
                      className="hidden"
                    />

                    <div className="text-center">
                      {isParsing ? (
                        <>
                          <div className="mx-auto h-12 w-12 flex items-center justify-center">
                            <div className="w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
                          </div>
                          <p className="mt-2 text-sm text-primary-600 dark:text-primary-400">
                            {t('addEntry.parsingLnk')}
                          </p>
                        </>
                      ) : (
                        <>
                          <svg
                            className="mx-auto h-12 w-12 text-gray-400"
                            stroke="currentColor"
                            fill="none"
                            viewBox="0 0 48 48"
                          >
                            <path
                              d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4h-4a4 4 0 00-4 4v4h8v-4a4 4 0 00-4-4zm-20 0h12"
                              strokeWidth={2}
                              strokeLinecap="round"
                              strokeLinejoin="round"
                            />
                          </svg>
                          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
                            {t('addEntry.dragAndDropOptional')}
                          </p>
                          <button
                            onClick={handleBrowseLnk}
                            className="mt-2 px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors text-sm"
                          >
                            {t('addEntry.browseFiles')}
                          </button>
                        </>
                      )}
                    </div>

                    {lnkPath && !isParsing && (
                      <div className="mt-3 px-3 py-2 bg-gray-50 dark:bg-gray-800 rounded text-sm flex items-center gap-2">
                        <svg className="w-4 h-4 text-green-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        <span className="truncate flex-1">{lnkPath}</span>
                        <button
                          onClick={() => { setLnkPath(''); setParseStatus('idle') }}
                          className="text-gray-400 hover:text-red-500 transition-colors flex-shrink-0"
                          title="Remove"
                        >
                          ✕
                        </button>
                      </div>
                    )}

                    {/* Parse status feedback */}
                    <AnimatePresence>
                      {parseStatus === 'success' && (
                        <motion.p
                          initial={{ opacity: 0, y: -5 }}
                          animate={{ opacity: 1, y: 0 }}
                          exit={{ opacity: 0 }}
                          className="mt-2 text-xs text-green-600 dark:text-green-400 text-center"
                        >
                          ✓ {t('addEntry.parseSuccess')}
                        </motion.p>
                      )}
                      {parseStatus === 'error' && (
                        <motion.p
                          initial={{ opacity: 0, y: -5 }}
                          animate={{ opacity: 1, y: 0 }}
                          exit={{ opacity: 0 }}
                          className="mt-2 text-xs text-red-600 dark:text-red-400 text-center"
                        >
                          ⚠️ {t('addEntry.parseError')}
                        </motion.p>
                      )}
                    </AnimatePresence>
                  </div>

                  {!lnkPath && !isParsing && (
                    <p className="mt-2 text-xs text-gray-400 dark:text-gray-500">
                      {t('addEntry.noLnkFile')}
                    </p>
                  )}
                </div>

                {/* Target Path (required) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.targetPath')} *
                  </label>
                  <div className="flex gap-2">
                    <input
                      type="text"
                      value={targetPath}
                      onChange={e => setTargetPath(e.target.value)}
                      placeholder={t('addEntry.targetPathPlaceholder')}
                      className="flex-1 px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                    />
                    <button
                      onClick={handleBrowseTarget}
                      className="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors text-sm"
                    >
                      {t('addEntry.browse')}
                    </button>
                  </div>
                </div>

                {/* Parameters & Working Directory */}
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      {t('addEntry.parameters')}
                    </label>
                    <input
                      type="text"
                      value={parameters}
                      onChange={e => setParameters(e.target.value)}
                      placeholder={t('addEntry.parametersPlaceholder')}
                      className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                    />
                  </div>

                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      {t('addEntry.workingDirectory')}
                    </label>
                    <input
                      type="text"
                      value={workingDir}
                      onChange={e => setWorkingDir(e.target.value)}
                      placeholder={t('addEntry.workingDirectoryPlaceholder')}
                      className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                    />
                  </div>
                </div>

                {/* Description (auto-filled from LNK) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.description')}
                  </label>
                  <input
                    type="text"
                    value={description}
                    onChange={e => setDescription(e.target.value)}
                    placeholder={t('addEntry.descriptionPlaceholder')}
                    className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                  />
                </div>

                {/* Tags */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.tags')}
                  </label>
                  <TagEditor
                    initialTags={tags}
                    onChange={setTags}
                  />
                </div>

                {/* Notes */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.notes')}
                  </label>
                  <div className="relative">
                    <textarea
                      value={notes}
                      onChange={e => setNotes(e.target.value)}
                      placeholder={t('addEntry.notesPlaceholder')}
                      rows={3}
                      maxLength={500}
                      className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm resize-none"
                    />
                    <div className="absolute bottom-2 right-2 text-xs text-gray-400">
                      {notes.length}/500
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <div className="px-6 py-4 border-t border-gray-200 dark:border-dark-border flex justify-end gap-2">
              <button
                onClick={() => {
                  resetForm()
                  onClose()
                }}
                className="px-4 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors text-sm"
              >
                {t('addEntry.cancel')}
              </button>
              <button
                onClick={handleCreate}
                disabled={!isFormValid || loading}
                className="px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors text-sm"
              >
                {loading ? t('addEntry.creating') : t('addEntry.createEntry')}
              </button>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}
