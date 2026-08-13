/**
 * Application Selector Modal
 *
 * Displays installed applications from the Windows Start Menu.
 * Supports two view modes: list and thumbnail (default).
 * View preference is persisted in localStorage.
 * Icons are lazy-loaded via the get_app_icon Tauri command.
 */
import { useState, useEffect, useMemo, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'

export interface InstalledApp {
  name: string
  target_path: string
  lnk_path: string
  description?: string
}

interface AppSelectorModalProps {
  isOpen: boolean
  onClose: () => void
  onSelect: (app: InstalledApp) => void
}

type ViewMode = 'list' | 'thumbnail'

const VIEW_STORAGE_KEY = 'app-selector-view'

// Generic application icon (SVG) used as fallback
const DEFAULT_APP_ICON = (
  <svg className="w-8 h-8 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16v2a2 2 0 002 2h12a2 2 0 002-2v-2M7 10l5 5 5-5M12 15V3" />
  </svg>
)

// App icon component with lazy loading
function AppIcon({ exePath, size }: { exePath: string; size: number }) {
  const [iconBase64, setIconBase64] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    let cancelled = false
    setLoading(true)

    invoke<string>('get_app_icon', { exePath })
      .then(base64 => {
        if (!cancelled && base64) {
          setIconBase64(base64)
        }
      })
      .catch(() => {
        // Icon extraction failed — use default icon
      })
      .finally(() => {
        if (!cancelled) setLoading(false)
      })

    return () => { cancelled = true }
  }, [exePath])

  if (loading) {
    return (
      <div
        style={{ width: size, height: size }}
        className="flex items-center justify-center bg-gray-100 dark:bg-gray-700 rounded animate-pulse"
      />
    )
  }

  if (iconBase64) {
    return (
      <img
        src={`data:image/png;base64,${iconBase64}`}
        alt=""
        style={{ width: size, height: size }}
        className="rounded"
      />
    )
  }

  return <div style={{ width: size, height: size }} className="flex items-center justify-center">{DEFAULT_APP_ICON}</div>
}

export function AppSelectorModal({ isOpen, onClose, onSelect }: AppSelectorModalProps) {
  const { t } = useTranslation()
  const [apps, setApps] = useState<InstalledApp[]>([])
  const [loading, setLoading] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<ViewMode>(() => {
    const saved = localStorage.getItem(VIEW_STORAGE_KEY)
    return saved === 'list' ? 'list' : 'thumbnail'
  })

  // Load installed apps when modal opens
  useEffect(() => {
    if (!isOpen) return

    setLoading(true)
    setSearchQuery('')

    invoke<InstalledApp[]>('list_installed_apps')
      .then(setApps)
      .catch(err => {
        console.error('Failed to list installed apps:', err)
        setApps([])
      })
      .finally(() => setLoading(false))
  }, [isOpen])

  // Persist view mode preference
  const handleViewModeChange = (mode: ViewMode) => {
    setViewMode(mode)
    localStorage.setItem(VIEW_STORAGE_KEY, mode)
  }

  // Filter apps by search query
  const filteredApps = useMemo(() => {
    if (!searchQuery.trim()) return apps
    const query = searchQuery.toLowerCase()
    return apps.filter(app =>
      app.name.toLowerCase().includes(query) ||
      app.target_path.toLowerCase().includes(query)
    )
  }, [apps, searchQuery])

  const handleSelect = useCallback((app: InstalledApp) => {
    onSelect(app)
    onClose()
  }, [onSelect, onClose])

  if (!isOpen) return null

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none p-4">
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          className="pointer-events-auto w-full max-w-3xl max-h-[80vh] flex flex-col bg-white dark:bg-gray-800 rounded-lg shadow-2xl overflow-hidden"
        >
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
            <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
              {t('appSelector.title')}
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Search + View Toggle */}
          <div className="flex items-center gap-3 px-6 py-3 border-b border-gray-200 dark:border-gray-700">
            {/* Search */}
            <div className="flex-1 relative">
              <svg className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-4.35-4.35M11 19a8 8 0 100-16 8 8 0 000 16z" />
              </svg>
              <input
                type="text"
                value={searchQuery}
                onChange={e => setSearchQuery(e.target.value)}
                placeholder={t('appSelector.searchPlaceholder')}
                className="w-full pl-9 pr-3 py-2 text-sm bg-gray-100 dark:bg-gray-700 border border-transparent rounded-lg focus:ring-1 focus:ring-primary-500 focus:border-transparent text-gray-900 dark:text-gray-100"
              />
            </div>

            {/* View mode toggle */}
            <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-1">
              <button
                onClick={() => handleViewModeChange('list')}
                className={`p-1.5 rounded transition-colors ${viewMode === 'list' ? 'bg-white dark:bg-gray-600 shadow-sm' : 'hover:bg-gray-200 dark:hover:bg-gray-600'}`}
                title={t('appSelector.listView')}
              >
                <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
                </svg>
              </button>
              <button
                onClick={() => handleViewModeChange('thumbnail')}
                className={`p-1.5 rounded transition-colors ${viewMode === 'thumbnail' ? 'bg-white dark:bg-gray-600 shadow-sm' : 'hover:bg-gray-200 dark:hover:bg-gray-600'}`}
                title={t('appSelector.thumbnailView')}
              >
                <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2V6zM14 6a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2V6zM4 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2H6a2 2 0 01-2-2v-2zM14 16a2 2 0 012-2h2a2 2 0 012 2v2a2 2 0 01-2 2h-2a2 2 0 01-2-2v-2z" />
                </svg>
              </button>
            </div>
          </div>

          {/* Content */}
          <div className="flex-1 overflow-y-auto p-4">
            {loading ? (
              <div className="flex flex-col items-center justify-center py-12">
                <div className="w-8 h-8 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
                <p className="mt-3 text-sm text-gray-500 dark:text-gray-400">
                  {t('appSelector.loading')}
                </p>
              </div>
            ) : filteredApps.length === 0 ? (
              <div className="flex flex-col items-center justify-center py-12">
                <svg className="w-12 h-12 text-gray-300 dark:text-gray-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <p className="mt-3 text-sm text-gray-500 dark:text-gray-400">
                  {searchQuery ? t('appSelector.noResults') : t('appSelector.noApps')}
                </p>
              </div>
            ) : viewMode === 'list' ? (
              /* List View */
              <div className="space-y-1">
                {filteredApps.map(app => (
                  <button
                    key={app.lnk_path}
                    onClick={() => handleSelect(app)}
                    className="w-full flex items-center gap-3 px-3 py-2 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-left"
                  >
                    <div className="flex-shrink-0 w-8 h-8 flex items-center justify-center">
                      {DEFAULT_APP_ICON}
                    </div>
                    <div className="flex-1 min-w-0">
                      <div className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                        {app.name}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                        {app.target_path}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            ) : (
              /* Thumbnail View */
              <div className="grid grid-cols-4 sm:grid-cols-5 md:grid-cols-6 gap-3">
                {filteredApps.map(app => (
                  <button
                    key={app.lnk_path}
                    onClick={() => handleSelect(app)}
                    className="flex flex-col items-center gap-2 p-3 rounded-lg hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors text-center"
                  >
                    <div className="flex-shrink-0">
                      <AppIcon exePath={app.target_path} size={48} />
                    </div>
                    <div className="text-xs text-gray-700 dark:text-gray-300 line-clamp-2 break-all leading-tight">
                      {app.name}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="px-6 py-3 border-t border-gray-200 dark:border-gray-700 flex items-center justify-between">
            <span className="text-xs text-gray-500 dark:text-gray-400">
              {t('appSelector.count', { count: filteredApps.length })}
            </span>
            <button
              onClick={onClose}
              className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
            >
              {t('common.cancel')}
            </button>
          </div>
        </motion.div>
      </div>
    </>
  )
}
