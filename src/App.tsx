import { useState, useRef, useCallback, useEffect } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { invoke } from '@tauri-apps/api/core'
import { SearchBox } from './components/SearchBox'
import { SearchResults } from './components/SearchResults'
import { GroupList } from './components/GroupList'
import { EntryDetail } from './components/EntryDetail'
import { AddEntryModal } from './components/AddEntryModal'
import { CreateGroupModal } from './components/CreateGroupModal'
import { HotkeySettings } from './components/HotkeySettings'
import { InterfaceShortcutSettings } from './components/InterfaceShortcutSettings'
import { AboutModal } from './components/AboutModal'
import { LanguageSwitcher } from './components/LanguageSwitcher'
import { useSearch } from './hooks/useSearch'
import { useGroups } from './hooks/useGroups'
import { useKeyboardShortcuts } from './hooks/useKeyboardShortcuts'
import { useProtocol } from './hooks/useProtocol'
import type { SearchResult as SearchResultType, GroupWithCount, Entry } from './types'

interface WindowState {
  isMaximized: boolean
  theme: 'light' | 'dark'
}

function App() {
  const { t } = useTranslation()

  const [windowState, setWindowState] = useState<WindowState>({
    isMaximized: false,
    theme: 'light',
  })

  const [searchQuery, setSearchQuery] = useState('')
  const [selectedIndex, setSelectedIndex] = useState(0)
  const [showSettings, setShowSettings] = useState(false)
  const [showAddModal, setShowAddModal] = useState(false)
  const [showCreateGroupModal, setShowCreateGroupModal] = useState(false)
  const [showAboutModal, setShowAboutModal] = useState(false)
  const [settingsTab, setSettingsTab] = useState<'hotkey' | 'shortcuts'>('hotkey')
  const [selectedEntryId, setSelectedEntryId] = useState<number | null>(null)
  const [selectedGroupId, setSelectedGroupId] = useState<number | null>(null)
  const [groupSearchResults, setGroupSearchResults] = useState<SearchResultType[]>([])
  const [isGroupSearching, setIsGroupSearching] = useState(false)

  const searchInputRef = useRef<HTMLInputElement>(null)

  const { groups, isLoading: groupsLoading, createGroup, refreshGroups } = useGroups()
  const {
    results,
    isLoading,
    error,
    totalCount,
    hasMore,
    loadMore,
    clearSearch,
    recentSearches,
    refresh: refreshSearch,
  } = useSearch(searchQuery)

  // Display results: either group-filtered or search results
  const displayResults = selectedGroupId && selectedGroupId !== 0
    ? groupSearchResults
    : results

  // Load entries when a group is selected
  useEffect(() => {
    if (!selectedGroupId || selectedGroupId === 0) {
      setGroupSearchResults([])
      return
    }

    const loadGroupEntries = async () => {
      setIsGroupSearching(true)
      try {
        const entries = await invoke<Entry[]>('get_group_entries', { groupId: selectedGroupId })
        setGroupSearchResults(entries.map(entry => ({ entry, score: 1.0 })))
      } catch (err) {
        console.error('Failed to load group entries:', err)
        setGroupSearchResults([])
      } finally {
        setIsGroupSearching(false)
      }
    }

    loadGroupEntries()
  }, [selectedGroupId])

  // Focus search box
  const focusSearch = useCallback(() => {
    searchInputRef.current?.focus()
  }, [])

  // Clear search
  const handleClearSearch = useCallback(() => {
    setSearchQuery('')
    setSelectedIndex(0)
    clearSearch()
    searchInputRef.current?.focus()
  }, [clearSearch])

  // Handle item selection
  const handleItemSelect = useCallback((result: SearchResultType) => {
    if (result.entry.id !== null) {
      setSelectedEntryId(result.entry.id)
    }
  }, [])

  // Handle item open (double-click or Enter)
  const handleItemOpen = useCallback(async (result: SearchResultType) => {
    // Use lnk_path if available, otherwise fall back to target_path (entries without .lnk file)
    const pathToOpen = result.entry.lnk_path || result.entry.target_path
    if (!pathToOpen) return

    try {
      await invoke('open_lnk_file', { path: pathToOpen })
    } catch (err) {
      console.error('Failed to open file:', err)
    }
  }, [])

  // Handle entry update from detail panel
  const handleEntryUpdate = useCallback(() => {
    refreshSearch()
  }, [refreshSearch])

  // Handle entry delete from detail panel
  const handleEntryDelete = useCallback(() => {
    setSelectedEntryId(null)
    refreshSearch()
  }, [refreshSearch])

  // Handle create group - opens the CreateGroupModal (replaces window.prompt)
  const handleCreateGroup = useCallback(() => {
    setShowCreateGroupModal(true)
  }, [])

  // Handle create group submission from modal
  const handleCreateGroupSubmit = useCallback(async (name: string, color: string) => {
    try {
      await createGroup(name, color)
    } catch (err) {
      console.error('Failed to create group:', err)
    }
  }, [createGroup])

  // Handle edit group
  const handleEditGroup = useCallback((group: GroupWithCount) => {
    const name = window.prompt(t('group.name'), group.name)
    if (name !== null && name.trim() && name.trim() !== group.name) {
      invoke('update_group', { id: group.id, name: name.trim() })
        .then(() => refreshGroups())
        .catch(err => console.error('Failed to update group:', err))
    }
  }, [refreshGroups, t])

  // Handle delete group
  const handleDeleteGroup = useCallback((groupId: number) => {
    if (window.confirm(t('group.deleteConfirm') || 'Are you sure?')) {
      invoke('delete_group', { id: groupId })
        .then(() => {
          refreshGroups()
          if (selectedGroupId === groupId) {
            setSelectedGroupId(null)
          }
        })
        .catch(err => console.error('Failed to delete group:', err))
    }
  }, [refreshGroups, selectedGroupId, t])

  // Handle new entry created
  const handleEntryCreated = useCallback(() => {
    refreshSearch()
    refreshGroups()
  }, [refreshSearch, refreshGroups])

  // Keyboard shortcuts are active but their display hints are removed from non-settings pages (Task 3).
  // Shortcut key customization is exclusively managed within the Settings interface.
  // Global hotkeys for window popup are handled by the backend hotkey system.
  useKeyboardShortcuts({
    onFocusSearch: focusSearch,
    onClearSearch: handleClearSearch,
    enabled: !showSettings,
  })

  // Handle protocol events (deep links and CLI args)
  useProtocol({
    onAdd: useCallback((path: string) => {
      console.log('[App] Protocol: Add entry from', path)
      setShowAddModal(true)
    }, []),
    onOpen: useCallback((id: string) => {
      console.log('[App] Protocol: Open entry', id)
      setSelectedEntryId(Number(id))
    }, []),
    onSearch: useCallback((query: string) => {
      console.log('[App] Protocol: Search for', query)
      setSearchQuery(query)
      searchInputRef.current?.focus()
    }, []),
    onSettings: useCallback(() => {
      console.log('[App] Protocol: Open settings')
      setShowSettings(true)
    }, []),
  })

  // Global context menu prevention
  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      e.preventDefault()
      return false
    }

    document.addEventListener('contextmenu', handleContextMenu)

    return () => {
      document.removeEventListener('contextmenu', handleContextMenu)
    }
  }, [])

  // Apply theme to document
  const applyTheme = useCallback((theme: 'light' | 'dark') => {
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [])

  // Apply initial theme
  useState(() => {
    applyTheme(windowState.theme)
  })

  const toggleTheme = () => {
    const newTheme = windowState.theme === 'light' ? 'dark' : 'light'
    setWindowState(prev => ({ ...prev, theme: newTheme }))
    applyTheme(newTheme)
  }

  return (
    <div className="h-screen flex flex-col bg-white dark:bg-dark-bg text-gray-900 dark:text-gray-100 overflow-hidden">
      {/* Title Bar */}
      <header className="window-drag h-9 bg-gray-100 dark:bg-dark-surface border-b border-gray-200 dark:border-dark-border flex items-center justify-between px-3 select-none">
        <div className="flex items-center gap-2">
          <svg className="w-4 h-4 text-primary-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
          </svg>
          <span className="text-xs font-medium">{t('app.name')}</span>
        </div>
        <div className="flex items-center gap-1 -webkit-app-region-no-drag">
          <LanguageSwitcher />
          <button
            onClick={() => setShowAddModal(true)}
            className="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title={t('addEntry.title')}
            aria-label={t('addEntry.title')}
          >
            <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </button>
          <button
            onClick={() => setShowSettings(true)}
            className="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title={t('settings.title')}
            aria-label={t('settings.title')}
          >
            <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
          </button>
          <button
            onClick={() => setShowAboutModal(true)}
            className="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title={t('about.title')}
            aria-label={t('about.title')}
          >
            <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
            </svg>
          </button>
          <button
            onClick={toggleTheme}
            className="p-1.5 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title={t('settings.theme')}
            aria-label={t('settings.theme')}
          >
            {windowState.theme === 'light' ? (
              <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
              </svg>
            ) : (
              <svg className="w-4 h-4 text-gray-600 dark:text-gray-300" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 3v1m0 16v1m9-9h-1M4 12H3m15.364 6.364l-.707-.707M6.343 6.343l-.707-.707m12.728 0l-.707.707M6.343 17.657l-.707.707M16 12a4 4 0 11-8 0 4 4 0 018 0z" />
              </svg>
            )}
          </button>
        </div>
      </header>

      {/* Main Content */}
      <div className="flex-1 flex overflow-hidden">
        {/* Sidebar - Group List */}
        <aside className="w-64 bg-gray-50 dark:bg-dark-surface border-r border-gray-200 dark:border-dark-border flex flex-col overflow-hidden">
          <div className="flex-1 overflow-y-auto py-2">
            <GroupList
              groups={groups}
              selectedGroupId={selectedGroupId}
              onSelectGroup={(groupId) => {
                setSelectedGroupId(groupId === 0 ? null : groupId)
                setSelectedIndex(0)
              }}
              onCreateGroup={handleCreateGroup}
              onEditGroup={handleEditGroup}
              onDeleteGroup={handleDeleteGroup}
              isLoading={groupsLoading}
            />
          </div>
        </aside>

        {/* Main Area - Search & Results */}
        <main className="flex-1 flex flex-col overflow-hidden min-w-0">
          {/* Hero Search Section */}
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="pt-8 pb-4 px-8"
          >
            <div className="max-w-3xl mx-auto">
              <SearchBox
                ref={searchInputRef}
                value={searchQuery}
                onChange={setSearchQuery}
                isLoading={isLoading}
                placeholder={t('search.placeholder')}
              />

              {/* Recent searches (when query is empty) */}
              {!searchQuery && recentSearches.length > 0 && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="mt-4 text-center"
                >
                  <p className="text-xs text-gray-400 dark:text-gray-600 mb-2">
                    {t('search.recentSearches')}
                  </p>
                  <div className="flex flex-wrap justify-center gap-2">
                    {recentSearches.slice(0, 5).map((search, index) => (
                      <button
                        key={index}
                        onClick={() => setSearchQuery(search)}
                        className="px-3 py-1 text-xs bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 rounded-full transition-colors"
                      >
                        {search}
                      </button>
                    ))}
                  </div>
                </motion.div>
              )}
            </div>
          </motion.div>

          {/* Results Area */}
          <div className="flex-1 overflow-hidden px-4 pb-2">
            <div className="max-w-5xl mx-auto h-full bg-white dark:bg-dark-bg border border-gray-200 dark:border-dark-border rounded-lg overflow-hidden">
              {/* Error state */}
              {error && (
                <motion.div
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  className="m-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg"
                >
                  <p className="text-sm text-red-800 dark:text-red-200">{error}</p>
                </motion.div>
              )}

              {/* Search results */}
              <SearchResults
                results={displayResults}
                query={searchQuery}
                isLoading={isLoading || isGroupSearching}
                hasMore={hasMore}
                onLoadMore={loadMore}
                selectedIndex={selectedIndex}
                onSelectedIndexChange={setSelectedIndex}
                onItemSelect={handleItemSelect}
                onItemOpen={handleItemOpen}
              />
            </div>
          </div>
        </main>
      </div>

      {/* Status Bar */}
      <footer className="h-6 bg-gray-100 dark:bg-dark-surface border-t border-gray-200 dark:border-dark-border flex items-center px-3 text-xs text-gray-600 dark:text-gray-400 justify-between">
        <span>
          {selectedGroupId && selectedGroupId !== 0
            ? `${displayResults.length} ${t('group.entries')}`
            : searchQuery && displayResults.length > 0
              ? t('search.results', { count: totalCount })
              : t('status.ready')}
        </span>
        <span className="text-gray-400 dark:text-gray-600">
          {groups.length > 0 ? `${groups.length} ${t('group.groups').toLowerCase()}` : ''}
        </span>
      </footer>

      {/* Settings Modal */}
      <AnimatePresence>
        {showSettings && (
          <>
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              onClick={() => setShowSettings(false)}
              className="fixed inset-0 bg-black/50 z-40"
            />
            <motion.div
              initial={{ opacity: 0, scale: 0.95, y: -20 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: -20 }}
              className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none"
            >
              <div className="pointer-events-auto max-w-lg w-full mx-4 max-h-[85vh] flex flex-col bg-white dark:bg-gray-800 rounded-lg shadow-xl overflow-hidden">
                {/* Settings Header with Tabs */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-gray-200 dark:border-gray-700">
                  <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                    {t('settings.title')}
                  </h2>
                  <button
                    onClick={() => setShowSettings(false)}
                    className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
                  >
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
                {/* Tab Navigation */}
                <div className="flex border-b border-gray-200 dark:border-gray-700">
                  <button
                    onClick={() => setSettingsTab('hotkey')}
                    className={`flex-1 px-4 py-2 text-sm font-medium transition-colors ${
                      settingsTab === 'hotkey'
                        ? 'text-primary-600 dark:text-primary-400 border-b-2 border-primary-500'
                        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
                    }`}
                  >
                    {t('settings.hotkey')}
                  </button>
                  <button
                    onClick={() => setSettingsTab('shortcuts')}
                    className={`flex-1 px-4 py-2 text-sm font-medium transition-colors ${
                      settingsTab === 'shortcuts'
                        ? 'text-primary-600 dark:text-primary-400 border-b-2 border-primary-500'
                        : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-300'
                    }`}
                  >
                    {t('settings.interfaceShortcuts')}
                  </button>
                </div>
                {/* Tab Content */}
                <div className="flex-1 overflow-y-auto">
                  {settingsTab === 'hotkey' ? (
                    <HotkeySettings />
                  ) : (
                    <InterfaceShortcutSettings />
                  )}
                </div>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>

      {/* Add Entry Modal */}
      <AddEntryModal
        isOpen={showAddModal}
        onClose={() => setShowAddModal(false)}
        onCreated={handleEntryCreated}
      />

      {/* Create Group Modal */}
      <CreateGroupModal
        isOpen={showCreateGroupModal}
        onClose={() => setShowCreateGroupModal(false)}
        onCreate={handleCreateGroupSubmit}
        existingNames={groups.map(g => g.name)}
      />

      {/* About Modal */}
      <AboutModal
        isOpen={showAboutModal}
        onClose={() => setShowAboutModal(false)}
      />

      {/* Entry Detail Panel */}
      <EntryDetail
        entryId={selectedEntryId}
        onClose={() => setSelectedEntryId(null)}
        onUpdate={handleEntryUpdate}
        onDelete={handleEntryDelete}
      />
    </div>
  )
}

export default App
