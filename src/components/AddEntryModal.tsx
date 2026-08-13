import { useState, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useCreateEntry } from '../hooks/useEntry'
import { TagEditor } from './TagEditor'
import { AppSelectorModal, type InstalledApp } from './AppSelectorModal'
import type { Entry, ParsedLnkProperties } from '../types'

type AddMode = 'application' | 'file' | 'folder'
type OpenMethod = 'explorer' | 'app' | 'custom'

interface AddEntryModalProps {
  isOpen: boolean
  onClose: () => void
  onCreated?: (entry: Entry) => void
}

/**
 * Extract the filename (without extension) from a file path.
 * Used to auto-fill the description when an .exe is selected in application mode.
 */
function extractExeDescription(filePath: string): string {
  const fileName = filePath.split(/[\\/]/).pop() || ''
  const lastDot = fileName.lastIndexOf('.')
  return lastDot > 0 ? fileName.substring(0, lastDot) : fileName
}

export function AddEntryModal({ isOpen, onClose, onCreated }: AddEntryModalProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<AddMode>('application')
  const [lnkPath, setLnkPath] = useState('')
  const [targetPath, setTargetPath] = useState('')
  const [parameters, setParameters] = useState('')
  const [workingDir, setWorkingDir] = useState('')
  const [description, setDescription] = useState('')
  const [tags, setTags] = useState<string[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const [isParsing, setIsParsing] = useState(false)
  const [parseStatus, setParseStatus] = useState<'idle' | 'success' | 'error'>('idle')
  const [openMethod, setOpenMethod] = useState<OpenMethod>('explorer')
  const [selectedApp, setSelectedApp] = useState('')
  const [customCommand, setCustomCommand] = useState('')
  const [showAppSelector, setShowAppSelector] = useState(false)

  const { createEntry, loading, error } = useCreateEntry()

  const resetForm = () => {
    setMode('application')
    setLnkPath('')
    setTargetPath('')
    setParameters('')
    setWorkingDir('')
    setDescription('')
    setTags([])
    setParseStatus('idle')
    setOpenMethod('explorer')
    setSelectedApp('')
    setCustomCommand('')
    setShowAppSelector(false)
  }

  /**
   * Parse a .lnk file and auto-fill form fields with the extracted properties.
   * This implements the auto-completion feature (Task 6).
   *
   * All mode-specific fields are cleared before parsing so that stale values
   * from a previously-selected file are not retained when the new .lnk lacks
   * the corresponding property. This enforces the single-file overwrite
   * semantics: selecting a new file replaces the old one entirely.
   */
  const parseAndAutoFill = useCallback(async (filePath: string) => {
    if (!filePath) return

    setIsParsing(true)
    setParseStatus('idle')

    // Clear all auto-filled fields first so the new LNK starts from a clean
    // state (prevents stale values persisting from a previous selection).
    setTargetPath('')
    setParameters('')
    setWorkingDir('')
    setDescription('')

    try {
      const props = await invoke<ParsedLnkProperties>('parse_lnk_file', { path: filePath })

      // Auto-fill from parsed properties
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

  /**
   * Handle a file/folder path selected via browse or drag-drop.
   * Behavior depends on the current mode:
   * - application: .lnk triggers parseAndAutoFill, .exe sets target_path and description
   * - file/folder: sets target_path and auto-fills description from filename
   *
   * Single-file overwrite: selecting a new file always replaces the previous
   * selection. Only one file may be selected at a time per mode.
   */
  const handleFileSelected = useCallback((filePath: string) => {
    if (!filePath) return

    if (mode === 'application') {
      const lower = filePath.toLowerCase()
      if (lower.endsWith('.lnk')) {
        setLnkPath(filePath)
        parseAndAutoFill(filePath)
      } else if (lower.endsWith('.exe')) {
        setLnkPath('')
        setTargetPath(filePath)
        setDescription(extractExeDescription(filePath))
        setParameters('')
        setWorkingDir('')
        setParseStatus('idle')
      }
    } else {
      // file or folder mode - overwrite target path and auto-fill description
      setTargetPath(filePath)
      setDescription(extractExeDescription(filePath))
      setParseStatus('idle')
    }
  }, [mode, parseAndAutoFill])

  /**
   * Browse for a file/folder using the Tauri dialog plugin.
   * Filter and directory options depend on the current mode.
   */
  const handleBrowse = async () => {
    try {
      let result: string | null = null
      if (mode === 'application') {
        result = await open({
          multiple: false,
          filters: [
            { name: t('addEntry.lnkFilter'), extensions: ['lnk'] },
            { name: t('addEntry.exeFilter'), extensions: ['exe'] },
          ],
        })
      } else if (mode === 'file') {
        result = await open({ multiple: false })
      } else {
        // folder mode
        result = await open({ multiple: false, directory: true })
      }
      if (result) {
        handleFileSelected(result)
      }
    } catch (err) {
      console.error('Failed to browse:', err)
    }
  }

  /**
   * Select an .exe application for the "Open with application" open method.
   */
  const handleSelectApp = async () => {
    try {
      const result = await open({
        multiple: false,
        filters: [{ name: t('addEntry.exeFilter'), extensions: ['exe'] }],
      })
      if (result) {
        setSelectedApp(result)
      }
    } catch (err) {
      console.error('Failed to select application:', err)
    }
  }

  /**
   * Extract file path from an HTML5 DragEvent.
   * In Tauri's webview (with dragDropEnabled: false), the File object
   * exposes a non-standard `path` property containing the full file path.
   */
  const extractFilePath = (e: React.DragEvent): string | null => {
    const files = e.dataTransfer?.files
    if (files && files.length > 0) {
      // Tauri webview exposes the full path via the non-standard `path` property
      const file = files[0] as File & { path?: string }
      if (file.path) return file.path
    }
    return null
  }

  /**
   * HTML5 drag-drop handlers for the drop zone.
   * Replaces the previous Tauri onDragDropEvent listener, which was
   * incompatible with HTML5 drag-drop (used for internal item dragging).
   */
  const handleDragEnter = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.dataTransfer?.types?.includes('Files')) {
      setIsDragging(true)
    }
  }, [])

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    if (e.dataTransfer?.types?.includes('Files')) {
      e.dataTransfer.dropEffect = 'copy'
      setIsDragging(true)
    }
  }, [])

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    // Only hide drag state when leaving the drop zone (not entering a child)
    if (e.currentTarget === e.target) {
      setIsDragging(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)

    const filePath = extractFilePath(e)
    if (filePath) {
      // In application mode, only accept .lnk or .exe
      if (mode === 'application') {
        const lower = filePath.toLowerCase()
        if (!lower.endsWith('.lnk') && !lower.endsWith('.exe')) {
          return
        }
      }
      handleFileSelected(filePath)
    }
  }, [mode, handleFileSelected])

  /**
   * Switch to a new mode. Resets mode-specific fields but keeps tags
   * (the common field across all modes).
   */
  const handleModeChange = (newMode: AddMode) => {
    if (newMode === mode) return
    setMode(newMode)
    // Reset mode-specific fields; keep tags as the common field
    setLnkPath('')
    setTargetPath('')
    setParameters('')
    setWorkingDir('')
    setDescription('')
    setParseStatus('idle')
    setOpenMethod('explorer')
    setSelectedApp('')
    setCustomCommand('')
    setShowAppSelector(false)
  }

  /**
   * Build the JSON string stored in the `parameters` field for file/folder modes.
   * Format: {"openMethod": "explorer|app|custom", "app": "path", "customCommand": "format"}
   */
  const buildParametersJson = (): string => {
    return JSON.stringify({
      openMethod,
      app: selectedApp,
      customCommand,
    })
  }

  // In application mode, lnkPath alone is sufficient (the backend opens via lnk_path || target_path).
  // In file/folder modes, targetPath is always required.
  const hasValidTarget = mode === 'application'
    ? (targetPath.trim() !== '' || lnkPath.trim() !== '')
    : targetPath.trim() !== ''

  const isFormValid =
    description.trim() !== '' &&
    hasValidTarget &&
    tags.length > 0

  const handleCreate = async () => {
    if (!isFormValid) return

    try {
      const entry = await createEntry({
        lnk_path: lnkPath || '',
        target_path: targetPath,
        target_type: mode === 'folder'
          ? { type: 'Folder' as const, path: targetPath }
          : { type: 'File' as const, path: targetPath },
        parameters: mode === 'application'
          ? (parameters || undefined)
          : (buildParametersJson() || undefined),
        working_dir: mode === 'application' ? (workingDir || undefined) : undefined,
        description: description || undefined,
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

  const getDragDropText = (): string => {
    if (mode === 'application') return t('addEntry.dragAndDropApp')
    if (mode === 'file') return t('addEntry.dragAndDropFile')
    return t('addEntry.dragAndDropFolder')
  }

  // Selected file/folder path to display in the upload area
  const selectedPath = lnkPath || targetPath

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

          {/* Flex container for centering - avoids transform conflict with framer-motion scale animation */}
          <motion.div
            initial={{ opacity: 0, scale: 0.95 }}
            animate={{ opacity: 1, scale: 1 }}
            exit={{ opacity: 0, scale: 0.95 }}
            className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none p-4"
          >
          <div className="pointer-events-auto w-full max-w-2xl max-h-[85vh] flex flex-col bg-white dark:bg-dark-surface border border-gray-200 dark:border-dark-border rounded-lg shadow-2xl overflow-hidden">
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

              {/* Mode Selector */}
              <div className="mb-6">
                <div className="grid grid-cols-3 gap-2">
                  {(['application', 'file', 'folder'] as AddMode[]).map((m) => (
                    <button
                      key={m}
                      onClick={() => handleModeChange(m)}
                      className={`px-3 py-2 rounded text-sm transition-colors text-left ${
                        mode === m
                          ? 'bg-primary-500 text-white'
                          : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                      }`}
                    >
                      <div className="font-medium">
                        {m === 'application' && t('addEntry.modeApplication')}
                        {m === 'file' && t('addEntry.modeFile')}
                        {m === 'folder' && t('addEntry.modeFolder')}
                      </div>
                      <div className={`text-xs mt-0.5 ${mode === m ? 'text-primary-100' : 'text-gray-500 dark:text-gray-400'}`}>
                        {m === 'application' && t('addEntry.modeApplicationDesc')}
                        {m === 'file' && t('addEntry.modeFileDesc')}
                        {m === 'folder' && t('addEntry.modeFolderDesc')}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              <div className="space-y-6">
                {/* Upload Area (drag-drop or browse) */}
                <div>
                  <div
                    onDragEnter={handleDragEnter}
                    onDragOver={handleDragOver}
                    onDragLeave={handleDragLeave}
                    onDrop={handleDrop}
                    className={`relative border-2 border-dashed rounded-lg p-6 transition-colors ${
                      isDragging
                        ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                        : 'border-gray-300 dark:border-gray-600'
                    }`}
                  >
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
                            {getDragDropText()}
                          </p>
                          <button
                            onClick={handleBrowse}
                            className="mt-2 px-4 py-2 bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors text-sm"
                          >
                            {t('addEntry.browseFiles')}
                          </button>
                        </>
                      )}
                    </div>

                    {selectedPath && !isParsing && (
                      <div className="mt-3 px-3 py-2 bg-gray-50 dark:bg-gray-800 rounded text-sm flex items-center gap-2">
                        <svg className="w-4 h-4 text-green-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                        </svg>
                        <span className="truncate flex-1">{selectedPath}</span>
                        <button
                          onClick={() => {
                            setLnkPath('')
                            setTargetPath('')
                            setParseStatus('idle')
                          }}
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
                </div>

                {/* Name/Description (required, used as the overall entry name) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.nameRequired')}
                  </label>
                  <input
                    type="text"
                    value={description}
                    onChange={e => setDescription(e.target.value)}
                    placeholder={t('addEntry.namePlaceholder')}
                    className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                  />
                </div>

                {/* Target Path (auto-filled from file analysis, but editable) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.targetPath')} *
                  </label>
                  <input
                    type="text"
                    value={targetPath}
                    onChange={e => setTargetPath(e.target.value)}
                    placeholder={t('addEntry.targetPathPlaceholder')}
                    className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                  />
                </div>

                {/* Application mode: Parameters & Working Directory */}
                {mode === 'application' && (
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
                )}

                {/* File/Folder mode: Open Method selector */}
                {(mode === 'file' || mode === 'folder') && (
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                      {t('addEntry.openMethod')}
                    </label>
                    <select
                      value={openMethod}
                      onChange={e => setOpenMethod(e.target.value as OpenMethod)}
                      className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                    >
                      <option value="explorer">{t('addEntry.openMethodDefault')}</option>
                      <option value="app">{t('addEntry.openMethodApp')}</option>
                      <option value="custom">{t('addEntry.openMethodCustom')}</option>
                    </select>

                    {openMethod === 'app' && (
                      <div className="mt-2">
                        <div className="flex gap-2">
                          <button
                            onClick={handleSelectApp}
                            className="px-3 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors text-sm"
                          >
                            {t('addEntry.selectApp')}
                          </button>
                          <button
                            onClick={() => setShowAppSelector(true)}
                            className="px-3 py-2 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors text-sm"
                          >
                            {t('addEntry.browseApps')}
                          </button>
                        </div>
                        {selectedApp && (
                          <div className="mt-2 px-3 py-2 bg-gray-50 dark:bg-gray-800 rounded text-sm flex items-center gap-2">
                            <svg className="w-4 h-4 text-green-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                            </svg>
                            <span className="truncate flex-1">{selectedApp}</span>
                            <button
                              onClick={() => setSelectedApp('')}
                              className="text-gray-400 hover:text-red-500 transition-colors flex-shrink-0"
                              title="Remove"
                            >
                              ✕
                            </button>
                          </div>
                        )}
                      </div>
                    )}

                    {openMethod === 'custom' && (
                      <div className="mt-2">
                        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                          {t('addEntry.customCommandFormat')}
                        </label>
                        <input
                          type="text"
                          value={customCommand}
                          onChange={e => setCustomCommand(e.target.value)}
                          placeholder={t('addEntry.customCommandFormatPlaceholder')}
                          className="w-full px-3 py-2 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded focus:ring-1 focus:ring-primary-500 text-sm"
                        />
                        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                          {mode === 'folder'
                            ? t('addEntry.customCommandFolderHint')
                            : t('addEntry.customCommandHint')}
                        </p>
                      </div>
                    )}
                  </div>
                )}

                {/* Tags (required) */}
                <div>
                  <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                    {t('addEntry.tagsRequired')}
                  </label>
                  <TagEditor
                    tags={tags}
                    onTagsChange={setTags}
                  />
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
          </div>
          </motion.div>
        </>
      )}

      {/* App Selector Modal — shown when user clicks "Browse Apps" */}
      <AppSelectorModal
        isOpen={showAppSelector}
        onClose={() => setShowAppSelector(false)}
        onSelect={(app: InstalledApp) => {
          setSelectedApp(app.target_path)
        }}
      />
    </AnimatePresence>
  )
}
