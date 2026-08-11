import { useState, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useWindowPosition } from '../hooks/useWindowPosition'

export function MainWindow() {
  const [searchQuery, setSearchQuery] = useState('')
  const [results, setResults] = useState<string[]>([])
  const [theme, setTheme] = useState<'light' | 'dark'>('light')
  const { windowState, setWindowState } = useWindowPosition()
  const appWindow = getCurrentWindow()

  useEffect(() => {
    // Apply theme to document
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [theme])

  useEffect(() => {
    // Listen for window state changes
    const unlisten = appWindow.listen('window-state-changed', async (event) => {
      if (event.payload === 'maximized' || event.payload === 'unmaximized') {
        const maximized = await appWindow.isMaximized()
        setWindowState(prev => ({ ...prev, maximized }))
      }
    })

    return () => {
      unlisten.then((fn: () => void) => fn())
    }
  }, [setWindowState, appWindow])

  const handleMinimize = async () => {
    // Minimize to tray instead of taskbar
    await appWindow.hide()
  }

  const handleMaximize = async () => {
    if (windowState.maximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
    const maximized = await appWindow.isMaximized()
    setWindowState(prev => ({ ...prev, maximized }))
  }

  const handleClose = async () => {
    await appWindow.close()
  }

  const handleSearch = (query: string) => {
    setSearchQuery(query)
    // TODO: Implement actual search logic
    if (query.trim()) {
      setResults(['Result 1', 'Result 2', 'Result 3'])
    } else {
      setResults([])
    }
  }

  return (
    <div className="h-screen flex flex-col bg-white dark:bg-dark-bg overflow-hidden">
      {/* Custom Title Bar - Draggable region */}
      <header 
        data-tauri-drag-region
        className="h-12 bg-gray-50 dark:bg-dark-surface border-b border-gray-200 dark:border-dark-border flex items-center justify-between px-4 select-none"
      >
        <div className="flex items-center gap-3">
          <div className="w-6 h-6 flex items-center justify-center">
            <svg className="w-4 h-4 text-primary-600 dark:text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
            </svg>
          </div>
          <span className="text-sm font-medium text-gray-700 dark:text-gray-200">
            LNK File Management Center
          </span>
        </div>

        {/* Window Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={() => setTheme(theme === 'light' ? 'dark' : 'light')}
            className="w-6 h-6 flex items-center justify-center rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title="Toggle theme"
          >
            {theme === 'light' ? '🌙' : '☀️'}
          </button>
          <div className="flex items-center gap-0.5 ml-2">
            <button
              onClick={handleMinimize}
              className="w-11 h-8 flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              title="Minimize"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20 12H4" />
              </svg>
            </button>
            <button
              onClick={handleMaximize}
              className="w-11 h-8 flex items-center justify-center hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
              title={windowState.maximized ? "Restore" : "Maximize"}
            >
              {windowState.maximized ? (
                <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 4H6a2 2 0 00-2 2v12a2 2 0 002 2h12a2 2 0 002-2v-2m-4-8h4m0 0v4m0-4L8 16" />
                </svg>
              ) : (
                <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8V6a2 2 0 012-2h2m8 0h2a2 2 0 012 2v2m0 8v2a2 2 0 01-2 2h-2m-8 0H6a2 2 0 01-2-2v-2" />
                </svg>
              )}
            </button>
            <button
              onClick={handleClose}
              className="w-11 h-8 flex items-center justify-center hover:bg-red-500 hover:text-white transition-colors rounded-tr"
              title="Close"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>
        </div>
      </header>

      {/* Main Content Area */}
      <main className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-dark-bg">
        {/* Search Area - Full-bleed hero */}
        <motion.div 
          initial={{ opacity: 0, y: -20 }}
          animate={{ opacity: 1, y: 0 }}
          className="px-12 pt-16 pb-8"
        >
          <div className="max-w-3xl mx-auto">
            {/* Brand/Title */}
            <motion.div
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
              className="text-center mb-8"
            >
              <h1 className="text-4xl font-light text-gray-900 dark:text-white tracking-tight mb-2">
                LNK Files
              </h1>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                Search and manage your Windows shortcuts
              </p>
            </motion.div>

            {/* Search Input - Large and prominent */}
            <motion.div
              initial={{ scale: 0.95, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              transition={{ delay: 0.2, type: "spring", stiffness: 200 }}
            >
              <div className="relative">
                <input
                  type="text"
                  value={searchQuery}
                  onChange={(e) => handleSearch(e.target.value)}
                  placeholder="Search LNK files..."
                  className="w-full h-14 px-5 py-3 pl-12 text-lg bg-gray-50 dark:bg-dark-surface border-2 border-gray-200 dark:border-dark-border rounded-xl focus:outline-none focus:border-primary-500 dark:focus:border-primary-400 transition-colors"
                  autoFocus
                />
                <svg 
                  className="absolute left-4 top-1/2 transform -translate-y-1/2 w-5 h-5 text-gray-400" 
                  fill="none" 
                  viewBox="0 0 24 24" 
                  stroke="currentColor"
                >
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </div>
            </motion.div>
          </div>
        </motion.div>

        {/* Results Area */}
        <AnimatePresence>
          {searchQuery.trim() && (
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              className="flex-1 px-12 pb-8 overflow-auto"
            >
              <div className="max-w-3xl mx-auto">
                <div className="space-y-2">
                  {results.map((result, index) => (
                    <motion.div
                      key={index}
                      initial={{ opacity: 0, x: -10 }}
                      animate={{ opacity: 1, x: 0 }}
                      transition={{ delay: index * 0.05 }}
                      className="p-4 bg-gray-50 dark:bg-dark-surface rounded-lg hover:bg-gray-100 dark:hover:bg-gray-800 transition-colors cursor-pointer"
                    >
                      <div className="flex items-center gap-3">
                        <div className="w-8 h-8 flex items-center justify-center bg-primary-100 dark:bg-primary-900 rounded">
                          <svg className="w-4 h-4 text-primary-600 dark:text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                          </svg>
                        </div>
                        <div>
                          <div className="text-sm font-medium text-gray-900 dark:text-white">{result}</div>
                          <div className="text-xs text-gray-500 dark:text-gray-400">Shortcut path</div>
                        </div>
                      </div>
                    </motion.div>
                  ))}
                </div>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        {/* Empty State */}
        {!searchQuery.trim() && (
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="flex-1 flex items-center justify-center px-12"
          >
            <div className="text-center">
              <div className="text-gray-300 dark:text-gray-600 text-6xl mb-4">📁</div>
              <p className="text-gray-400 dark:text-gray-500 text-sm">
                Start typing to search your LNK files
              </p>
            </div>
          </motion.div>
        )}
      </main>

      {/* Status Bar */}
      <footer className="h-7 bg-gray-50 dark:bg-dark-surface border-t border-gray-200 dark:border-dark-border flex items-center px-4 text-xs text-gray-500 dark:text-gray-400">
        <span>Ready</span>
        <span className="ml-auto">
          {results.length > 0 ? `${results.length} results` : 'No search query'}
        </span>
      </footer>
    </div>
  )
}