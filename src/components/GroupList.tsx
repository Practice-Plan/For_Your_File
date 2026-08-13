/**
 * Collapsible list of groups in sidebar
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { GroupItem } from './GroupItem'
import type { GroupWithCount } from '../types'

interface GroupListProps {
  groups: GroupWithCount[]
  selectedGroupId: number | null
  onSelectGroup: (groupId: number) => void
  onCreateGroup: () => void
  onEditGroup: (group: GroupWithCount) => void
  onDeleteGroup: (groupId: number) => void
  onDropToGroup: (entryId: number, groupId: number) => void
  onDropToAllEntries: (entryId: number) => void
  isLoading?: boolean
}

export function GroupList({
  groups,
  selectedGroupId,
  onSelectGroup,
  onCreateGroup,
  onEditGroup,
  onDeleteGroup,
  onDropToGroup,
  onDropToAllEntries,
  isLoading = false,
}: GroupListProps) {
  const { t } = useTranslation()
  const [isCollapsed, setIsCollapsed] = useState(false)
  const [dragOverAllEntries, setDragOverAllEntries] = useState(false)

  return (
    <div className="flex flex-col">
      {/* Header */}
      <div className="flex items-center justify-between px-3 py-2">
        <button
          onClick={() => setIsCollapsed(!isCollapsed)}
          className="flex items-center gap-2 text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wide hover:text-gray-700 dark:hover:text-gray-300 transition-colors"
        >
          <svg
            className={`w-3 h-3 transition-transform ${isCollapsed ? '-rotate-90' : ''}`}
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
          </svg>
          <span>{t('group.groups')}</span>
          {!isCollapsed && (
            <span className="text-gray-400 dark:text-gray-600 font-normal normal-case">
              ({groups.length})
            </span>
          )}
        </button>

        {/* Create button */}
        {!isCollapsed && (
          <button
            onClick={onCreateGroup}
            className="p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-colors"
            title={t('group.createNewGroup')}
          >
            <svg className="w-4 h-4 text-gray-500 dark:text-gray-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </button>
        )}
      </div>

      {/* Group list */}
      <AnimatePresence>
        {!isCollapsed && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            animate={{ height: 'auto', opacity: 1 }}
            exit={{ height: 0, opacity: 0 }}
            className="overflow-hidden"
          >
            <div className="space-y-1 px-2 pb-2">
              {/* All entries option — also a drop target to remove from group */}
              <button
                onClick={() => onSelectGroup(0)}
                onDragOver={(e) => {
                  e.preventDefault()
                  e.dataTransfer.dropEffect = 'link'
                  setDragOverAllEntries(true)
                }}
                onDragLeave={() => setDragOverAllEntries(false)}
                onDrop={(e) => {
                  e.preventDefault()
                  setDragOverAllEntries(false)
                  const entryId = parseInt(e.dataTransfer.getData('text/plain'), 10)
                  if (!isNaN(entryId)) {
                    onDropToAllEntries(entryId)
                  }
                }}
                className={`
                  w-full flex items-center gap-2 px-3 py-2 rounded-lg text-sm
                  transition-colors
                  ${dragOverAllEntries ? 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-900/20' : ''}
                  ${selectedGroupId === null || selectedGroupId === 0
                    ? 'bg-gray-100 dark:bg-gray-800 text-gray-900 dark:text-gray-100'
                    : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-800/50'
                  }
                `}
              >
                <div className="w-3 h-3 rounded-full bg-gradient-to-r from-gray-400 to-gray-500" />
                <span className="flex-1 text-left">{t('group.allEntries')}</span>
              </button>

              {/* Divider */}
              {groups.length > 0 && (
                <div className="h-px bg-gray-200 dark:bg-gray-700 my-1" />
              )}

              {/* Loading state */}
              {isLoading && (
                <div className="flex items-center justify-center py-4">
                  <div className="w-5 h-5 border-2 border-primary-500 border-t-transparent rounded-full animate-spin" />
                </div>
              )}

              {/* Group items */}
              {!isLoading && groups.length === 0 && (
                <p className="text-xs text-gray-400 dark:text-gray-600 text-center py-2">
                  {t('group.noGroupsYet')}
                </p>
              )}

              {!isLoading && groups.map((group) => (
                <GroupItem
                  key={group.id}
                  group={group}
                  isSelected={selectedGroupId === group.id}
                  onSelect={onSelectGroup}
                  onEdit={onEditGroup}
                  onDelete={onDeleteGroup}
                  onDropToGroup={onDropToGroup}
                />
              ))}
            </div>
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}