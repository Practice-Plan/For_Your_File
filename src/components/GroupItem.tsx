/**
 * Single group item with color indicator and actions
 */
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { motion } from 'framer-motion'
import type { GroupWithCount } from '../types'

interface GroupItemProps {
  group: GroupWithCount
  isSelected: boolean
  onSelect: (groupId: number) => void
  onEdit: (group: GroupWithCount) => void
  onDelete: (groupId: number) => void
  isCollapsed?: boolean
}

export function GroupItem({
  group,
  isSelected,
  onSelect,
  onEdit,
  onDelete,
  isCollapsed = false,
}: GroupItemProps) {
  const { t } = useTranslation()
  const [showMenu, setShowMenu] = useState(false)

  if (!group.id) return null

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      className={`
        group relative flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer
        transition-colors duration-150
        ${isSelected
          ? 'bg-primary-100 dark:bg-primary-900/30 border border-primary-300 dark:border-primary-700'
          : 'hover:bg-gray-100 dark:hover:bg-gray-800'
        }
      `}
      onClick={() => onSelect(group.id!)}
    >
      {/* Color indicator */}
      <div
        className="w-3 h-3 rounded-full flex-shrink-0 ring-2 ring-white dark:ring-gray-900"
        style={{ backgroundColor: group.color }}
        title={group.color}
      />

      {/* Group name and count */}
      {!isCollapsed && (
        <>
          <span className="flex-1 text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
            {group.name}
          </span>

          {/* Entry count badge */}
          <span className="text-xs text-gray-500 dark:text-gray-400 bg-gray-200 dark:bg-gray-700 px-2 py-0.5 rounded-full">
            {group.entry_count}
          </span>

          {/* Actions menu button */}
          <button
            onClick={(e) => {
              e.stopPropagation()
              setShowMenu(!showMenu)
            }}
            className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-opacity"
          >
            <svg className="w-4 h-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
            </svg>
          </button>

          {/* Dropdown menu */}
          {showMenu && (
            <>
              <div
                className="fixed inset-0 z-40"
                onClick={() => setShowMenu(false)}
              />
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                className="absolute right-0 top-full mt-1 z-50 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-[120px]"
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    setShowMenu(false)
                    onEdit(group)
                  }}
                  className="w-full text-left px-3 py-1.5 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200"
                >
                  {t('group.edit')}
                </button>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    setShowMenu(false)
                    onDelete(group.id!)
                  }}
                  className="w-full text-left px-3 py-1.5 text-sm hover:bg-red-100 dark:hover:bg-red-900/30 text-red-600 dark:text-red-400"
                >
                  {t('group.delete')}
                </button>
              </motion.div>
            </>
          )}
        </>
      )}
    </motion.div>
  )
}