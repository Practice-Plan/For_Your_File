/**
 * Detail view showing entries in a group
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import type { GroupWithCount, Entry } from '../types'

interface GroupDetailProps {
  group: GroupWithCount
  entries: Entry[]
  isLoading?: boolean
  onOpenEntry: (entry: Entry) => void
  onRemoveEntry: (entryId: number, groupId: number) => void
  onOpenAll: () => void
  selectedEntryIds: number[]
  onToggleEntrySelection: (entryId: number) => void
}

export function GroupDetail({
  group,
  entries,
  isLoading = false,
  onOpenEntry,
  onRemoveEntry,
  onOpenAll,
  selectedEntryIds,
  onToggleEntrySelection,
}: GroupDetailProps) {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy, setSortBy] = useState<'name' | 'frequency' | 'recent'>('frequency')

  // Filter and sort entries
  const filteredEntries = entries
    .filter((entry) => {
      if (!searchQuery) return true
      const query = searchQuery.toLowerCase()
      return (
        entry.lnk_path.toLowerCase().includes(query) ||
        entry.target_path.toLowerCase().includes(query) ||
        (entry.tags?.toLowerCase().includes(query) ?? false)
      )
    })
    .sort((a, b) => {
      switch (sortBy) {
        case 'name':
          return a.lnk_path.localeCompare(b.lnk_path)
        case 'frequency':
          return b.frequency - a.frequency
        case 'recent':
          return (b.last_opened ?? 0) - (a.last_opened ?? 0)
        default:
          return 0
      }
    })

  const isAllSelected = filteredEntries.length > 0 &&
    filteredEntries.every(e => e.id && selectedEntryIds.includes(e.id))

  const handleSelectAll = () => {
    filteredEntries.forEach(e => {
      if (e.id) {
        onToggleEntrySelection(e.id)
      }
    })
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center gap-3 px-4 py-3 border-b border-gray-200 dark:border-gray-700">
        <div
          className="w-4 h-4 rounded-full"
          style={{ backgroundColor: group.color }}
        />
        <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100 flex-1">
          {group.name}
        </h2>
        <span className="text-sm text-gray-500 dark:text-gray-400">
          {t('group.entries', { count: group.entry_count })}
        </span>
      </div>

      {/* Toolbar */}
      <div className="flex items-center gap-2 px-4 py-2 border-b border-gray-200 dark:border-gray-700">
        {/* Search */}
        <div className="flex-1 relative">
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            placeholder={t('group.searchInGroup')}
            className="w-full px-3 py-1.5 pl-8 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
          />
          <svg
            className="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
          </svg>
        </div>

        {/* Sort */}
        <select
          value={sortBy}
          onChange={(e) => setSortBy(e.target.value as 'name' | 'frequency' | 'recent')}
          className="px-2 py-1.5 text-sm border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
        >
          <option value="frequency">{t('sort.mostUsed')}</option>
          <option value="recent">{t('sort.recentlyOpened')}</option>
          <option value="name">{t('sort.alphabetical')}</option>
        </select>

        {/* Actions */}
        <button
          onClick={onOpenAll}
          disabled={filteredEntries.length === 0}
          className="px-3 py-1.5 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 disabled:bg-gray-400 disabled:cursor-not-allowed rounded-lg transition-colors"
        >
          {t('group.openAll')}
        </button>
      </div>

      {/* Entry list */}
      <div className="flex-1 overflow-auto">
        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <div className="w-6 h-6 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
          </div>
        ) : filteredEntries.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-gray-400 dark:text-gray-600">
            <svg className="w-12 h-12 mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M20 13V6a2 2 0 00-2-2H6a2 2 0 00-2 2v7m16 0v5a2 2 0 01-2 2H6a2 2 0 01-2-2v-5m16 0h-2.586a1 1 0 00-.707.293l-2.414 2.414a1 1 0 01-.707.293h-3.172a1 1 0 01-.707-.293l-2.414-2.414A1 1 0 006.586 13H4" />
            </svg>
            <p className="text-sm">
              {searchQuery ? t('group.noMatchingEntries') : t('group.noEntries')}
            </p>
          </div>
        ) : (
          <div className="divide-y divide-gray-100 dark:divide-gray-800">
            {/* Select all header */}
            <div className="flex items-center gap-3 px-4 py-2 bg-gray-50 dark:bg-gray-800/50">
              <input
                type="checkbox"
                checked={isAllSelected}
                onChange={handleSelectAll}
                className="w-4 h-4 rounded border-gray-300 dark:border-gray-600"
              />
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {t('common.selected')} {selectedEntryIds.length}
              </span>
            </div>

            {/* Entry items */}
            <AnimatePresence>
              {filteredEntries.map((entry) => {
                const isSelected = entry.id && selectedEntryIds.includes(entry.id)

                return (
                  <motion.div
                    key={entry.id}
                    initial={{ opacity: 0, y: 5 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, x: -10 }}
                    className={`group flex items-center gap-3 px-4 py-2 hover:bg-gray-50 dark:hover:bg-gray-800/50 transition-colors ${
                      isSelected ? 'bg-primary-50 dark:bg-primary-900/20' : ''
                    }`}
                  >
                    {/* Checkbox */}
                    <input
                      type="checkbox"
                      checked={Boolean(isSelected)}
                      onChange={() => entry.id && onToggleEntrySelection(entry.id)}
                      className="w-4 h-4 rounded border-gray-300 dark:border-gray-600"
                      onClick={(e) => e.stopPropagation()}
                    />

                    {/* Entry info */}
                    <button
                      onClick={() => onOpenEntry(entry)}
                      className="flex-1 text-left"
                    >
                      <div className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                        {entry.lnk_path.split(/[/\\]/).pop()}
                      </div>
                      <div className="text-xs text-gray-500 dark:text-gray-400 truncate">
                        {entry.target_path}
                      </div>
                    </button>

                    {/* Tags */}
                    {entry.tags && (
                      <div className="flex gap-1">
                        {entry.tags.split(',').slice(0, 2).map((tag, i) => (
                          <span
                            key={i}
                            className="px-1.5 py-0.5 text-xs bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400 rounded"
                          >
                            {tag.trim()}
                          </span>
                        ))}
                      </div>
                    )}

                    {/* Frequency indicator */}
                    <div className="text-xs text-gray-400 dark:text-gray-600">
                      {entry.frequency} {t('entry.opens')}
                    </div>

                    {/* Remove button */}
                    <button
                      onClick={() => entry.id && group.id && onRemoveEntry(entry.id, group.id)}
                      className="opacity-0 group-hover:opacity-100 p-1 text-gray-400 hover:text-red-500 transition-all"
                      title="Remove from group"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                      </svg>
                    </button>
                  </motion.div>
                )
              })}
            </AnimatePresence>
          </div>
        )}
      </div>
    </div>
  )
}