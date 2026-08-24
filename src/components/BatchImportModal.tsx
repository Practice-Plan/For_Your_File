/**
 * Batch Import Modal
 *
 * Large rectangular modal for adding multiple entries at once.
 *
 * Layout (per user spec):
 *   +---------+----------------+------------------+
 *   |  Modes  |  Drop / Browse |   File List      |
 *   |  (App)  |  (multiple)    |   (scrollable)   |
 *   |  (File) |  ---------     |                  |
 *   |  (Fldr) |  Unified Cfg   |                  |
 *   +---------+----------------+------------------+
 *   |  [Search.............................. x]    |
 *   +------------------------------------------------+
 *
 * Features:
 * - Three modes: Application, File, Folder (leftmost column)
 * - Drag-and-drop or browse multiple files (middle top)
 * - Unified configuration applied to all files (middle bottom)
 * - File list with per-file status (right column)
 * - Search at the bottom with clear (x) button
 * - Double-click a file in the list to configure it individually
 */
import { useState, useCallback, useMemo, useRef, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import type { Entry } from '../types'
import { TagEditor } from './TagEditor'

type AddMode = 'application' | 'file' | 'folder'
type OpenMethod = 'explorer' | 'app' | 'custom'

interface BatchFileItem {
  /** Unique id for React keys */
  uid: string
  /** Full path to the file/folder */
  path: string
  /** Auto-extracted filename (without extension for .exe) */
  fileName: string
  /** Per-file overrides; undefined means "use unified config" */
  override?: {
    description?: string
    tags?: string[]
    parameters?: string
    working_dir?: string
    open_method?: OpenMethod
    app?: string
    custom_command?: string
  }
  /** Result of batch creation (set after import) */
  status?: 'pending' | 'success' | 'error'
  error?: string
}

interface BatchImportModalProps {
  isOpen: boolean
  onClose: () => void
  onCreated?: (entry: Entry) => void
}

function extractFileName(filePath: string): string {
  const fileName = filePath.split(/[\\/]/).pop() || ''
  const lastDot = fileName.lastIndexOf('.')
  return lastDot > 0 ? fileName.substring(0, lastDot) : fileName
}

function makeUid(): string {
  return `${Date.now()}-${Math.random().toString(36).slice(2, 9)}`
}

export function BatchImportModal({ isOpen, onClose, onCreated }: BatchImportModalProps) {
  const { t } = useTranslation()
  const [mode, setMode] = useState<AddMode>('application')
  const [files, setFiles] = useState<BatchFileItem[]>([])
  const [isDragging, setIsDragging] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [isImporting, setIsImporting] = useState(false)

  // Unified configuration
  const [unifiedTags, setUnifiedTags] = useState<string[]>([])
  const [unifiedParameters, setUnifiedParameters] = useState('')
  const [unifiedWorkingDir, setUnifiedWorkingDir] = useState('')
  const [unifiedOpenMethod, setUnifiedOpenMethod] = useState<OpenMethod>('explorer')
  const [unifiedApp, setUnifiedApp] = useState('')
  const [unifiedCustomCommand, setUnifiedCustomCommand] = useState('')

  // Per-file edit popover
  const [editingUid, setEditingUid] = useState<string | null>(null)

  // Progress tracking
  const [progress, setProgress] = useState<{ current: number; total: number } | null>(null)
  const [importErrors, setImportErrors] = useState<string[]>([])

  const dropZoneRef = useRef<HTMLDivElement>(null)

  // Listen for progress events from the backend
  useEffect(() => {
    if (!isOpen || !isImporting) return

    let cancelled = false
    let unlisten: (() => void) | undefined

    listen<{ current: number; total: number; target_path: string; status: string }>(
      'batch-import-progress',
      (e) => {
        setProgress({ current: e.payload.current, total: e.payload.total })
      }
    ).then(fn => {
      if (cancelled) {
        fn()
      } else {
        unlisten = fn
      }
    })

    return () => {
      cancelled = true
      if (unlisten) unlisten()
    }
  }, [isOpen, isImporting])

  // Reset everything when the modal closes
  const resetAll = useCallback(() => {
    setMode('application')
    setFiles([])
    setSearchQuery('')
    setIsDragging(false)
    setUnifiedTags([])
    setUnifiedParameters('')
    setUnifiedWorkingDir('')
    setUnifiedOpenMethod('explorer')
    setUnifiedApp('')
    setUnifiedCustomCommand('')
    setEditingUid(null)
    setProgress(null)
    setImportErrors([])
  }, [])

  // Validate that a path is acceptable for the current mode
  const isPathValid = useCallback((filePath: string, currentMode: AddMode): boolean => {
    const lower = filePath.toLowerCase()
    if (currentMode === 'application') {
      return lower.endsWith('.lnk') || lower.endsWith('.exe')
    }
    // file and folder modes accept anything
    return true
  }, [])

  // Add multiple file paths to the list, deduplicating by path
  const addPaths = useCallback((paths: string[]) => {
    const validPaths = paths.filter(p => p && isPathValid(p, mode))
    if (validPaths.length === 0) return

    setFiles(prev => {
      const existing = new Set(prev.map(f => f.path.toLowerCase()))
      const newItems: BatchFileItem[] = []
      for (const p of validPaths) {
        if (!existing.has(p.toLowerCase())) {
          newItems.push({
            uid: makeUid(),
            path: p,
            fileName: extractFileName(p),
            status: 'pending',
          })
          existing.add(p.toLowerCase())
        }
      }
      return [...prev, ...newItems]
    })
  }, [mode, isPathValid])

  // Browse for multiple files/folders
  const handleBrowse = async () => {
    try {
      if (mode === 'application') {
        const result = await open({
          multiple: true,
          filters: [
            { name: t('addEntry.lnkFilter'), extensions: ['lnk'] },
            { name: t('addEntry.exeFilter'), extensions: ['exe'] },
          ],
        })
        if (result) {
          const paths = Array.isArray(result) ? result : [result]
          addPaths(paths)
        }
      } else if (mode === 'file') {
        const result = await open({ multiple: true })
        if (result) {
          const paths = Array.isArray(result) ? result : [result]
          addPaths(paths)
        }
      } else {
        // folder mode - can only select one folder at a time in Tauri dialog
        const result = await open({ multiple: true, directory: true })
        if (result) {
          const paths = Array.isArray(result) ? result : [result]
          addPaths(paths)
        }
      }
    } catch (err) {
      console.error('Failed to browse:', err)
    }
  }

  // Select application for "open with app" unified config
  const handleSelectApp = async () => {
    try {
      const result = await open({
        multiple: false,
        filters: [{ name: t('addEntry.exeFilter'), extensions: ['exe'] }],
      })
      if (result) {
        setUnifiedApp(result)
      }
    } catch (err) {
      console.error('Failed to select application:', err)
    }
  }

  /**
   * Extract file paths from an HTML5 DragEvent.
   * In Tauri's webview (with dragDropEnabled: false), each File object
   * exposes a non-standard `path` property containing the full file path.
   */
  const extractFilePaths = (e: React.DragEvent): string[] => {
    const files = e.dataTransfer?.files
    if (!files || files.length === 0) return []
    const paths: string[] = []
    for (let i = 0; i < files.length; i++) {
      const file = files[i] as File & { path?: string }
      if (file.path) {
        paths.push(file.path)
      }
    }
    return paths
  }

  // HTML5 drag-drop handlers for the drop zone
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
    if (e.currentTarget === e.target) {
      setIsDragging(false)
    }
  }, [])

  const handleDrop = useCallback((e: React.DragEvent) => {
    e.preventDefault()
    e.stopPropagation()
    setIsDragging(false)
    const paths = extractFilePaths(e)
    const validPaths = paths.filter(p => isPathValid(p, mode))
    if (validPaths.length > 0) {
      addPaths(validPaths)
    }
  }, [mode, isPathValid, addPaths])

  // Switch mode: clear file list (different modes accept different file types)
  const handleModeChange = (newMode: AddMode) => {
    if (newMode === mode) return
    setMode(newMode)
    setFiles([])
    setEditingUid(null)
  }

  // Remove a file from the list
  const handleRemoveFile = (uid: string) => {
    setFiles(prev => prev.filter(f => f.uid !== uid))
    if (editingUid === uid) setEditingUid(null)
  }

  // Update a file's override config
  const handleUpdateOverride = (uid: string, override: Partial<NonNullable<BatchFileItem['override']>>) => {
    setFiles(prev => prev.map(f =>
      f.uid === uid
        ? { ...f, override: { ...f.override, ...override } }
        : f
    ))
  }

  // Clear all files
  const handleClearAll = () => {
    setFiles([])
    setEditingUid(null)
  }

  // Filtered file list based on search
  const filteredFiles = useMemo(() => {
    if (!searchQuery.trim()) return files
    const q = searchQuery.toLowerCase()
    return files.filter(f =>
      f.path.toLowerCase().includes(q) ||
      f.fileName.toLowerCase().includes(q)
    )
  }, [files, searchQuery])

  // Build the parameters JSON for file/folder modes
  const buildFileFolderParams = (item: BatchFileItem): string => {
    const openMethod = item.override?.open_method ?? unifiedOpenMethod
    const app = item.override?.app ?? unifiedApp
    const customCommand = item.override?.custom_command ?? unifiedCustomCommand
    return JSON.stringify({ openMethod, app, customCommand })
  }

  // Build the payload for a single file item
  const buildEntryInput = (item: BatchFileItem) => {
    const tags = (item.override?.tags ?? unifiedTags).join(',')
    const description = item.override?.description ?? item.fileName
    if (mode === 'application') {
      const lower = item.path.toLowerCase()
      const isLnk = lower.endsWith('.lnk')
      return {
        // For .lnk files, both lnk_path and target_path are set to the .lnk
        // path. The backend's open_entry command checks lnk_path first and
        // launches it directly (Windows Shell resolves the actual target).
        // Setting target_path to the .lnk path (instead of empty) ensures:
        // 1. FTS5 search can find the entry via target_path
        // 2. BatchCreateResult.target_path matches item.path for status matching
        lnk_path: isLnk ? item.path : '',
        target_path: item.path,
        target_type: 'File',
        parameters: (item.override?.parameters ?? unifiedParameters) || null,
        working_dir: (item.override?.working_dir ?? unifiedWorkingDir) || null,
        description: description || null,
        tags: tags || null,
        notes: null,
      }
    }
    // file / folder mode
    return {
      lnk_path: '',
      target_path: item.path,
      target_type: mode === 'folder' ? 'Folder' : 'File',
      parameters: buildFileFolderParams(item) || null,
      working_dir: null,
      description: description || null,
      tags: tags || null,
      notes: null,
    }
  }

  // Import all files with chunked processing for better progress feedback
  const handleImport = async () => {
    if (files.length === 0) return
    setIsImporting(true)
    setProgress({ current: 0, total: files.length })
    setImportErrors([])

    try {
      const inputs = files.map(buildEntryInput)
      const CHUNK_SIZE = 20
      const allResults: Array<{ success: boolean; error: string | null; entry_id: number | null; target_path: string }> = []
      const allErrors: string[] = []

      // Process in chunks for progress feedback
      for (let i = 0; i < inputs.length; i += CHUNK_SIZE) {
        const chunk = inputs.slice(i, i + CHUNK_SIZE)
        const results = await invoke<Array<{ success: boolean; error: string | null; entry_id: number | null; target_path: string }>>(
          'batch_create_entries',
          { entries: chunk }
        )
        allResults.push(...results)

        // Collect errors
        for (const r of results) {
          if (!r.success && r.error) {
            allErrors.push(r.error)
          }
        }

        // Update progress
        setProgress({ current: Math.min(i + CHUNK_SIZE, inputs.length), total: inputs.length })
      }

      // Update file statuses
      setFiles(prev => prev.map(f => {
        const result = allResults.find(r => r.target_path === f.path)
        if (result) {
          return {
            ...f,
            status: result.success ? 'success' : 'error',
            error: result.error || undefined,
          }
        }
        return f
      }))

      // Store error messages for summary display
      if (allErrors.length > 0) {
        setImportErrors(allErrors)
      }

      // Count successes
      const successCount = allResults.filter(r => r.success).length

      // Always notify parent to refresh entry list and invalidate search cache
      onCreated?.({} as Entry)

      // If all succeeded, close after a brief delay
      if (successCount === files.length) {
        setTimeout(() => {
          resetAll()
          onClose()
        }, 800)
      }
    } catch (err) {
      console.error('Batch import failed:', err)
      setImportErrors([String(err)])
    } finally {
      setIsImporting(false)
      setProgress(null)
    }
  }

  // Remove successfully imported files from the list
  const handleRemoveImported = () => {
    setFiles(prev => prev.filter(f => f.status !== 'success'))
  }

  if (!isOpen) return null

  const pendingCount = files.filter(f => f.status === 'pending' || !f.status).length
  const successCount = files.filter(f => f.status === 'success').length
  const errorCount = files.filter(f => f.status === 'error').length

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
            className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none p-4"
          >
            <div className="pointer-events-auto w-full max-w-6xl h-[85vh] flex flex-col bg-white dark:bg-dark-surface border border-gray-200 dark:border-dark-border rounded-lg shadow-2xl overflow-hidden">
              {/* Header */}
              <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-dark-border flex-shrink-0">
                <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                  {t('batchImport.title')}
                </h2>
                <button
                  onClick={() => { resetAll(); onClose() }}
                  className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                >
                  <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </div>

              {/* Main 3-column area */}
              <div className="flex-1 flex overflow-hidden min-h-0">
                {/* Left: Mode selection */}
                <div className="w-32 flex-shrink-0 border-r border-gray-200 dark:border-dark-border p-3 flex flex-col gap-2 overflow-y-auto">
                  {(['application', 'file', 'folder'] as AddMode[]).map((m) => (
                    <button
                      key={m}
                      onClick={() => handleModeChange(m)}
                      className={`px-2 py-3 rounded text-xs transition-colors text-center ${
                        mode === m
                          ? 'bg-primary-500 text-white'
                          : 'bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 hover:bg-gray-200 dark:hover:bg-gray-700'
                      }`}
                    >
                      <div className="flex justify-center mb-1">
                        {m === 'application' && <AppIcon />}
                        {m === 'file' && <FileIcon />}
                        {m === 'folder' && <FolderIcon />}
                      </div>
                      <div className="font-medium">
                        {m === 'application' && t('addEntry.modeApplication')}
                        {m === 'file' && t('addEntry.modeFile')}
                        {m === 'folder' && t('addEntry.modeFolder')}
                      </div>
                    </button>
                  ))}
                  <div className="mt-auto text-xs text-gray-400 dark:text-gray-600 text-center">
                    {files.length} {t('batchImport.files')}
                  </div>
                </div>

                {/* Middle: Drop area + unified config */}
                <div className="w-80 flex-shrink-0 border-r border-gray-200 dark:border-dark-border flex flex-col overflow-y-auto">
                  {/* Drop/Browse area */}
                  <div className="p-3">
                    <div
                      ref={dropZoneRef}
                      onDragEnter={handleDragEnter}
                      onDragOver={handleDragOver}
                      onDragLeave={handleDragLeave}
                      onDrop={handleDrop}
                      className={`relative border-2 border-dashed rounded-lg p-4 transition-colors ${
                        isDragging
                          ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                          : 'border-gray-300 dark:border-gray-600'
                      }`}
                    >
                      <div className="text-center">
                        <svg className="mx-auto h-10 w-10 text-gray-400" stroke="currentColor" fill="none" viewBox="0 0 48 48">
                          <path d="M28 8H12a4 4 0 00-4 4v20m32-12v8m0 0v8a4 4 0 01-4 4H12a4 4 0 01-4-4v-4m32-4h-4a4 4 0 00-4 4v4h8v-4a4 4 0 00-4-4zm-20 0h12" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" />
                        </svg>
                        <p className="mt-2 text-xs text-gray-600 dark:text-gray-400">
                          {mode === 'application' && t('batchImport.dropApp')}
                          {mode === 'file' && t('batchImport.dropFile')}
                          {mode === 'folder' && t('batchImport.dropFolder')}
                        </p>
                        <button
                          onClick={handleBrowse}
                          className="mt-2 px-3 py-1.5 bg-primary-500 text-white rounded hover:bg-primary-600 transition-colors text-xs"
                        >
                          {t('batchImport.browse')}
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Unified config */}
                  <div className="px-3 pb-3 space-y-3">
                    <h3 className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide">
                      {t('batchImport.unifiedConfig')}
                    </h3>

                    {/* Tags */}
                    <div>
                      <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                        {t('addEntry.tagsRequired')}
                      </label>
                      <TagEditor tags={unifiedTags} onTagsChange={setUnifiedTags} />
                    </div>

                    {/* Mode-specific config */}
                    {mode === 'application' && (
                      <div className="space-y-2">
                        <div>
                          <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                            {t('addEntry.parameters')}
                          </label>
                          <input
                            type="text"
                            value={unifiedParameters}
                            onChange={e => setUnifiedParameters(e.target.value)}
                            placeholder={t('addEntry.parametersPlaceholder')}
                            className="w-full px-2 py-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs focus:ring-1 focus:ring-primary-500"
                          />
                        </div>
                        <div>
                          <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                            {t('addEntry.workingDirectory')}
                          </label>
                          <input
                            type="text"
                            value={unifiedWorkingDir}
                            onChange={e => setUnifiedWorkingDir(e.target.value)}
                            placeholder={t('addEntry.workingDirectoryPlaceholder')}
                            className="w-full px-2 py-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs focus:ring-1 focus:ring-primary-500"
                          />
                        </div>
                      </div>
                    )}

                    {(mode === 'file' || mode === 'folder') && (
                      <div className="space-y-2">
                        <div>
                          <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                            {t('addEntry.openMethod')}
                          </label>
                          <select
                            value={unifiedOpenMethod}
                            onChange={e => setUnifiedOpenMethod(e.target.value as OpenMethod)}
                            className="w-full px-2 py-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs focus:ring-1 focus:ring-primary-500"
                          >
                            <option value="explorer">{t('addEntry.openMethodDefault')}</option>
                            <option value="app">{t('addEntry.openMethodApp')}</option>
                            <option value="custom">{t('addEntry.openMethodCustom')}</option>
                          </select>
                        </div>

                        {unifiedOpenMethod === 'app' && (
                          <div>
                            <button
                              onClick={handleSelectApp}
                              className="px-2 py-1.5 bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors text-xs w-full truncate"
                            >
                              {unifiedApp || t('addEntry.selectApp')}
                            </button>
                            {unifiedApp && (
                              <button
                                onClick={() => setUnifiedApp('')}
                                className="mt-1 text-xs text-red-500 hover:text-red-600"
                              >
                                {t('common.cancel')}
                              </button>
                            )}
                          </div>
                        )}

                        {unifiedOpenMethod === 'custom' && (
                          <div>
                            <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                              {t('addEntry.customCommandFormat')}
                            </label>
                            <input
                              type="text"
                              value={unifiedCustomCommand}
                              onChange={e => setUnifiedCustomCommand(e.target.value)}
                              placeholder={t('addEntry.customCommandFormatPlaceholder')}
                              className="w-full px-2 py-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs focus:ring-1 focus:ring-primary-500"
                            />
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                </div>

                {/* Right: File list */}
                <div className="flex-1 flex flex-col overflow-hidden min-w-0">
                  {/* File list header */}
                  <div className="flex items-center justify-between px-4 py-2 border-b border-gray-200 dark:border-dark-border flex-shrink-0">
                    <span className="text-xs text-gray-500 dark:text-gray-400">
                      {filteredFiles.length} / {files.length} {t('batchImport.files')}
                      {successCount > 0 && <span className="text-green-500 ml-2">({successCount} {t('batchImport.imported')})</span>}
                      {errorCount > 0 && <span className="text-red-500 ml-2">({errorCount} {t('batchImport.errors')})</span>}
                    </span>
                    {files.length > 0 && (
                      <button
                        onClick={handleClearAll}
                        className="text-xs text-gray-400 hover:text-red-500 transition-colors"
                      >
                        {t('batchImport.clearAll')}
                      </button>
                    )}
                  </div>

                  {/* File list (scrollable) */}
                  <div className="flex-1 overflow-y-auto">
                    {filteredFiles.length === 0 ? (
                      <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-600">
                        <svg className="w-12 h-12 mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 13h6m-3-3v6m-9 1V7a2 2 0 012-2h6l2 2h6a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z" />
                        </svg>
                        <p className="text-sm">{t('batchImport.emptyList')}</p>
                      </div>
                    ) : (
                      filteredFiles.map((item) => (
                        <FileListItem
                          key={item.uid}
                          item={item}
                          isEditing={editingUid === item.uid}
                          onEdit={() => setEditingUid(editingUid === item.uid ? null : item.uid)}
                          onRemove={() => handleRemoveFile(item.uid)}
                          onUpdateOverride={(override) => handleUpdateOverride(item.uid, override)}
                        />
                      ))
                    )}
                  </div>

                  {/* Search bar at the bottom */}
                  <div className="border-t border-gray-200 dark:border-dark-border p-2 flex-shrink-0">
                    <div className="relative">
                      <svg className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                      </svg>
                      <input
                        type="text"
                        value={searchQuery}
                        onChange={e => setSearchQuery(e.target.value)}
                        placeholder={t('batchImport.searchPlaceholder')}
                        className="w-full pl-8 pr-8 py-1.5 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded text-sm focus:ring-1 focus:ring-primary-500"
                      />
                      {searchQuery && (
                        <button
                          onClick={() => setSearchQuery('')}
                          className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                          title={t('search.clear')}
                        >
                          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                          </svg>
                        </button>
                      )}
                    </div>
                  </div>
                </div>
              </div>

              {/* Progress bar */}
              {isImporting && progress && (
                <div className="px-6 py-2 border-t border-gray-200 dark:border-dark-border flex-shrink-0">
                  <div className="flex items-center gap-3">
                    <div className="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                      <motion.div
                        className="h-full bg-primary-500 rounded-full"
                        initial={{ width: 0 }}
                        animate={{ width: `${progress.total > 0 ? (progress.current / progress.total) * 100 : 0}%` }}
                        transition={{ duration: 0.3 }}
                      />
                    </div>
                    <span className="text-xs text-gray-500 dark:text-gray-400 whitespace-nowrap">
                      {progress.current} / {progress.total}
                    </span>
                  </div>
                </div>
              )}

              {/* Error summary */}
              {importErrors.length > 0 && !isImporting && (
                <div className="px-6 py-2 border-t border-gray-200 dark:border-dark-border flex-shrink-0">
                  <div className="bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded p-3">
                    <div className="flex items-start justify-between">
                      <div className="flex-1 min-w-0">
                        <p className="text-xs font-medium text-red-600 dark:text-red-400">
                          {t('batchImport.partialFailure', { count: importErrors.length })}
                        </p>
                        <ul className="mt-1 text-xs text-red-500 dark:text-red-400 max-h-24 overflow-y-auto">
                          {importErrors.slice(0, 10).map((err, i) => (
                            <li key={i} className="truncate">{err}</li>
                          ))}
                          {importErrors.length > 10 && (
                            <li className="text-gray-400">... {t('batchImport.moreErrors', { count: importErrors.length - 10 })}</li>
                          )}
                        </ul>
                      </div>
                      <button
                        onClick={() => setImportErrors([])}
                        className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 ml-2 flex-shrink-0"
                      >
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    </div>
                  </div>
                </div>
              )}

              {/* Footer */}
              <div className="flex items-center justify-between px-6 py-3 border-t border-gray-200 dark:border-dark-border flex-shrink-0">
                <div className="text-xs text-gray-500 dark:text-gray-400">
                  {pendingCount > 0 && <span>{pendingCount} {t('batchImport.pending')}</span>}
                  {successCount > 0 && (
                    <button onClick={handleRemoveImported} className="text-primary-500 hover:text-primary-600 ml-3">
                      {t('batchImport.removeImported')}
                    </button>
                  )}
                </div>
                <div className="flex gap-2">
                  <button
                    onClick={() => { resetAll(); onClose() }}
                    className="px-4 py-2 text-sm bg-gray-200 dark:bg-gray-700 rounded hover:bg-gray-300 dark:hover:bg-gray-600 transition-colors"
                  >
                    {t('common.close')}
                  </button>
                  <button
                    onClick={handleImport}
                    disabled={files.length === 0 || isImporting || pendingCount === 0}
                    className="px-4 py-2 text-sm bg-primary-500 text-white rounded hover:bg-primary-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                  >
                    {isImporting ? t('batchImport.importing') : t('batchImport.import', { count: pendingCount })}
                  </button>
                </div>
              </div>
            </div>
          </motion.div>
        </>
      )}
    </AnimatePresence>
  )
}

// ---------------------------------------------------------------------------
// FileListItem: single row in the batch file list
// ---------------------------------------------------------------------------

interface FileListItemProps {
  item: BatchFileItem
  isEditing: boolean
  onEdit: () => void
  onRemove: () => void
  onUpdateOverride: (override: Partial<NonNullable<BatchFileItem['override']>>) => void
}

function FileListItem({ item, isEditing, onEdit, onRemove, onUpdateOverride }: FileListItemProps) {
  const { t } = useTranslation()

  return (
    <div
      className="border-b border-gray-100 dark:border-dark-border"
      onDoubleClick={onEdit}
    >
      <div className="flex items-center gap-2 px-3 py-2 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors">
        {/* Status icon */}
        <div className="flex-shrink-0 w-5 h-5 flex items-center justify-center">
          {item.status === 'success' ? (
            <svg className="w-4 h-4 text-green-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
            </svg>
          ) : item.status === 'error' ? (
            <svg className="w-4 h-4 text-red-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          ) : (
            <div className="w-2 h-2 rounded-full bg-gray-300 dark:bg-gray-600" />
          )}
        </div>

        {/* File info */}
        <div className="flex-1 min-w-0">
          <div className="text-sm text-gray-900 dark:text-gray-100 truncate">
            {item.override?.description || item.fileName}
          </div>
          <div className="text-xs text-gray-400 dark:text-gray-600 truncate">
            {item.path}
          </div>
          {item.error && (
            <div className="text-xs text-red-500 truncate mt-0.5">{item.error}</div>
          )}
        </div>

        {/* Override indicator */}
        {item.override && (
          <span className="text-xs text-primary-500 flex-shrink-0" title={t('batchImport.customConfig')}>
            *
          </span>
        )}

        {/* Actions */}
        <button
          onClick={(e) => { e.stopPropagation(); onEdit() }}
          className="p-1 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 flex-shrink-0"
          title={t('batchImport.editFile')}
        >
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        </button>
        <button
          onClick={(e) => { e.stopPropagation(); onRemove() }}
          className="p-1 text-gray-400 hover:text-red-500 flex-shrink-0"
          title={t('common.delete')}
        >
          <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {/* Inline per-file editor */}
      <AnimatePresence>
        {isEditing && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden"
          >
            <div className="px-4 py-3 bg-gray-50 dark:bg-gray-800/50 space-y-2">
              <div>
                <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                  {t('addEntry.name')}
                </label>
                <input
                  type="text"
                  value={item.override?.description ?? item.fileName}
                  onChange={e => onUpdateOverride({ description: e.target.value })}
                  className="w-full px-2 py-1.5 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded text-xs focus:ring-1 focus:ring-primary-500"
                />
              </div>
              <div>
                <label className="block text-xs font-medium text-gray-600 dark:text-gray-400 mb-1">
                  {t('addEntry.tags')}
                </label>
                <TagEditor
                  tags={item.override?.tags ?? []}
                  onTagsChange={(tags) => onUpdateOverride({ tags })}
                />
                <p className="text-xs text-gray-400 mt-1">
                  {t('batchImport.tagsOverrideHint')}
                </p>
              </div>
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

// ---------------------------------------------------------------------------
// Icons
// ---------------------------------------------------------------------------

function AppIcon() {
  return (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 10V3L4 14h7v7l9-11h-7z" />
    </svg>
  )
}

function FileIcon() {
  return (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
    </svg>
  )
}

function FolderIcon() {
  return (
    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 19a2 2 0 01-2-2V5a2 2 0 012-2h4l2 3h4a2 2 0 012 2v9a2 2 0 01-2 2H5z" />
    </svg>
  )
}
