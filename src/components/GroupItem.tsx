/**
 * Single group item with color indicator and actions
 */
import { useState, useRef, useEffect } from 'react'
import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import { motion } from 'framer-motion'
import type { GroupWithCount } from '../types'

interface GroupItemProps {
  group: GroupWithCount
  isSelected: boolean
  onSelect: (groupId: number) => void
  onEdit: (group: GroupWithCount) => void
  onDelete: (groupId: number) => void
  onDropToGroup: (entryId: number, groupId: number) => void
  isCollapsed?: boolean
}

export function GroupItem({
  group,
  isSelected,
  onSelect,
  onEdit,
  onDelete,
  onDropToGroup,
  isCollapsed = false,
}: GroupItemProps) {
  const { t } = useTranslation()
  const [showMenu, setShowMenu] = useState(false)
  const [menuPos, setMenuPos] = useState<{ top: number; left: number } | null>(null)
  const [isDragOver, setIsDragOver] = useState(false)
  const menuBtnRef = useRef<HTMLButtonElement>(null)

  if (!group.id) return null

  const groupId = group.id

  const openMenu = (e: React.MouseEvent) => {
    e.stopPropagation()
    if (menuBtnRef.current) {
      const rect = menuBtnRef.current.getBoundingClientRect()
      // Position dropdown below the button, aligned to its right edge.
      // Clamp left so the menu doesn't go off-screen.
      const menuWidth = 120
      const left = Math.max(8, rect.right - menuWidth)
      const top = rect.bottom + 4
      setMenuPos({ top, left })
    }
    setShowMenu(true)
  }

  // Close menu on Escape or scroll
  useEffect(() => {
    if (!showMenu) return
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') setShowMenu(false)
    }
    const handleScroll = () => setShowMenu(false)
    window.addEventListener('keydown', handleEsc)
    window.addEventListener('scroll', handleScroll, true)
    return () => {
      window.removeEventListener('keydown', handleEsc)
      window.removeEventListener('scroll', handleScroll, true)
    }
  }, [showMenu])

  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      className={`
        group relative flex items-center gap-2 px-3 py-2 rounded-lg cursor-pointer
        transition-colors duration-150
        ${isDragOver ? 'ring-2 ring-primary-500 bg-primary-50 dark:bg-primary-900/20' : ''}
        ${isSelected
          ? 'bg-primary-100 dark:bg-primary-900/30 border border-primary-300 dark:border-primary-700'
          : 'hover:bg-gray-100 dark:hover:bg-gray-800'
        }
      `}
      onClick={() => onSelect(group.id!)}
      onDragOver={(e) => {
        e.preventDefault()
        e.dataTransfer.dropEffect = 'link'
        setIsDragOver(true)
      }}
      onDragLeave={() => setIsDragOver(false)}
      onDrop={(e) => {
        e.preventDefault()
        setIsDragOver(false)
        const entryId = parseInt(e.dataTransfer.getData('text/plain'), 10)
        if (!isNaN(entryId) && groupId) {
          onDropToGroup(entryId, groupId)
        }
      }}
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
            ref={menuBtnRef}
            onClick={openMenu}
            className="opacity-0 group-hover:opacity-100 p-1 rounded hover:bg-gray-200 dark:hover:bg-gray-700 transition-opacity"
          >
            <svg className="w-4 h-4 text-gray-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
            </svg>
          </button>

          {/* Dropdown menu - rendered via portal to escape sidebar overflow clipping */}
          {showMenu && menuPos && createPortal(
            <>
              <div
                className="fixed inset-0 z-40"
                onClick={() => setShowMenu(false)}
              />
              <motion.div
                initial={{ opacity: 0, scale: 0.95 }}
                animate={{ opacity: 1, scale: 1 }}
                style={{ position: 'fixed', top: menuPos.top, left: menuPos.left, zIndex: 50 }}
                className="bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1 min-w-[120px]"
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
            </>,
            document.body
          )}
        </>
      )}
    </motion.div>
  )
}