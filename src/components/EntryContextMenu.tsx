/**
 * Right-click context menu for search result entries.
 *
 * Renders a portal-positioned menu at the cursor with the following actions:
 * - Open
 * - Edit
 * - Delete
 * - Add to group (submenu of available groups)
 * - Remove from this group (disabled when "All Items" is selected or the
 *   entry is not in the currently-viewed group)
 * - Open working directory in Windows File Explorer
 */
import { useState, useEffect, useRef, useCallback } from 'react'
import { createPortal } from 'react-dom'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import type { Entry, GroupWithCount } from '../types'

export interface EntryContextMenuState {
  entry: Entry
  x: number
  y: number
}

interface EntryContextMenuProps {
  state: EntryContextMenuState | null
  onClose: () => void
  onOpen: (entry: Entry) => void
  onEdit: (entry: Entry) => void
  onDelete: (entry: Entry) => void
  onAddToGroup: (entry: Entry, groupId: number) => void
  onRemoveFromGroup: (entry: Entry, groupId: number) => void
  onOpenWorkingDir: (entry: Entry) => void
  /** Currently selected group id, or null/0 when viewing "All Items". */
  selectedGroupId: number | null
  /** All available groups for the "Add to group" submenu. */
  groups: GroupWithCount[]
  /** IDs of groups the focused entry belongs to (for checkmarks). */
  entryGroupIds?: number[]
}

export function EntryContextMenu({
  state,
  onClose,
  onOpen,
  onEdit,
  onDelete,
  onAddToGroup,
  onRemoveFromGroup,
  onOpenWorkingDir,
  selectedGroupId,
  groups,
  entryGroupIds = [],
}: EntryContextMenuProps) {
  const { t } = useTranslation()
  const [submenuPos, setSubmenuPos] = useState<{ top: number; left: number } | null>(null)
  const [submenuOpen, setSubmenuOpen] = useState(false)
  const menuRef = useRef<HTMLDivElement>(null)
  const submenuTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // "All Items" (null or 0) is not a group, so removal is not applicable.
  const viewingAllItems = !selectedGroupId || selectedGroupId === 0
  const canRemoveFromGroup = !viewingAllItems

  const closeSubmenu = useCallback(() => {
    setSubmenuOpen(false)
    setSubmenuPos(null)
  }, [])

  // Close on Escape / scroll / outside click
  useEffect(() => {
    if (!state) return
    const handleEsc = (e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        if (submenuOpen) {
          closeSubmenu()
        } else {
          onClose()
        }
      }
    }
    const handleScroll = () => onClose()
    window.addEventListener('keydown', handleEsc)
    window.addEventListener('scroll', handleScroll, true)
    return () => {
      window.removeEventListener('keydown', handleEsc)
      window.removeEventListener('scroll', handleScroll, true)
    }
  }, [state, onClose, submenuOpen, closeSubmenu])

  // Clear any pending submenu timer on unmount
  useEffect(() => {
    return () => {
      if (submenuTimerRef.current) clearTimeout(submenuTimerRef.current)
    }
  }, [])

  if (!state) return null

  const { entry, x, y } = state

  // Clamp menu position so it stays on-screen.
  const menuWidth = 220
  const menuHeight = 260
  const left = Math.min(x, window.innerWidth - menuWidth - 8)
  const top = Math.min(y, window.innerHeight - menuHeight - 8)

  const handleSubmenuEnter = (e: React.MouseEvent) => {
    if (submenuTimerRef.current) clearTimeout(submenuTimerRef.current)
    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const subWidth = 180
    let subLeft = rect.right
    if (subLeft + subWidth > window.innerWidth - 8) {
      subLeft = rect.left - subWidth
    }
    setSubmenuPos({ top: rect.top, left: subLeft })
    setSubmenuOpen(true)
  }

  const handleSubmenuLeave = () => {
    submenuTimerRef.current = setTimeout(() => {
      setSubmenuOpen(false)
      setSubmenuPos(null)
    }, 200)
  }

  const run = (fn: () => void) => {
    fn()
    onClose()
  }

  return createPortal(
    <>
      {/* Click-catcher to close menu on outside click */}
      <div className="fixed inset-0 z-[60]" onClick={onClose} onContextMenu={(e) => { e.preventDefault(); onClose() }} />

      <motion.div
        ref={menuRef}
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        exit={{ opacity: 0, scale: 0.95 }}
        transition={{ duration: 0.12 }}
        style={{ position: 'fixed', left, top, zIndex: 61 }}
        className="bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 py-1 min-w-[220px]"
      >
        {/* Open */}
        <MenuItem
          icon={<OpenIcon />}
          label={t('contextMenu.open')}
          onClick={() => run(() => onOpen(entry))}
        />

        {/* Edit */}
        <MenuItem
          icon={<EditIcon />}
          label={t('contextMenu.edit')}
          onClick={() => run(() => onEdit(entry))}
        />

        <Divider />

        {/* Add to group (submenu) */}
        {groups.length > 0 && (
          <div
            onMouseEnter={handleSubmenuEnter}
            onMouseLeave={handleSubmenuLeave}
          >
            <MenuItem
              icon={<AddToGroupIcon />}
              label={t('contextMenu.addToGroup')}
              hasChevron
              onClick={() => {
                // Toggle submenu on click as well (for touch devices)
                if (submenuOpen) {
                  closeSubmenu()
                } else {
                  const rect = menuRef.current?.getBoundingClientRect()
                  if (rect) {
                    const subWidth = 180
                    let subLeft = rect.right
                    if (subLeft + subWidth > window.innerWidth - 8) {
                      subLeft = rect.left - subWidth
                    }
                    setSubmenuPos({ top: rect.top, left: subLeft })
                    setSubmenuOpen(true)
                  }
                }
              }}
            />
          </div>
        )}

        {/* Remove from this group */}
        <MenuItem
          icon={<RemoveFromGroupIcon />}
          label={t('contextMenu.removeFromGroup')}
          disabled={!canRemoveFromGroup}
          title={
            viewingAllItems
              ? t('contextMenu.removeFromGroupDisabledAllItems')
              : t('contextMenu.removeFromGroupDisabled')
          }
          onClick={() => {
            if (!canRemoveFromGroup || !selectedGroupId) return
            run(() => onRemoveFromGroup(entry, selectedGroupId))
          }}
        />

        <Divider />

        {/* Open working directory */}
        <MenuItem
          icon={<FolderIcon />}
          label={t('contextMenu.openWorkingDir')}
          onClick={() => run(() => onOpenWorkingDir(entry))}
        />

        <Divider />

        {/* Delete */}
        <MenuItem
          icon={<DeleteIcon />}
          label={t('contextMenu.delete')}
          danger
          onClick={() => run(() => onDelete(entry))}
        />
      </motion.div>

      {/* Submenu for "Add to group" */}
      <AnimatePresence>
        {submenuOpen && submenuPos && (
          <motion.div
            initial={{ opacity: 0, x: -8 }}
            animate={{ opacity: 1, x: 0 }}
            exit={{ opacity: 0, x: -8 }}
            transition={{ duration: 0.12 }}
            style={{ position: 'fixed', left: submenuPos.left, top: submenuPos.top, zIndex: 62 }}
            className="bg-white dark:bg-gray-800 rounded-lg shadow-2xl border border-gray-200 dark:border-gray-700 py-1 min-w-[180px] max-h-[280px] overflow-y-auto"
            onMouseEnter={() => {
              if (submenuTimerRef.current) clearTimeout(submenuTimerRef.current)
            }}
            onMouseLeave={handleSubmenuLeave}
          >
            {groups.map((g) => {
              const inGroup = entryGroupIds.includes(g.id!)
              return (
                <button
                  key={g.id}
                  onClick={(e) => {
                    e.stopPropagation()
                    if (g.id) {
                      run(() => onAddToGroup(entry, g.id!))
                    }
                  }}
                  className="w-full flex items-center gap-2 px-3 py-1.5 text-sm text-left hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-200"
                >
                  <span
                    className="w-2.5 h-2.5 rounded-full flex-shrink-0"
                    style={{ backgroundColor: g.color }}
                  />
                  <span className="flex-1 truncate">{g.name}</span>
                  {inGroup && (
                    <svg className="w-3.5 h-3.5 text-green-500 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                    </svg>
                  )}
                </button>
              )
            })}
          </motion.div>
        )}
      </AnimatePresence>
    </>,
    document.body
  )
}

// ---------------------------------------------------------------------------
// Internal sub-components
// ---------------------------------------------------------------------------

interface MenuItemProps {
  icon: React.ReactNode
  label: string
  onClick: () => void
  disabled?: boolean
  danger?: boolean
  hasChevron?: boolean
  title?: string
}

function MenuItem({ icon, label, onClick, disabled, danger, hasChevron, title }: MenuItemProps) {
  return (
    <button
      onClick={(e) => {
        e.stopPropagation()
        if (!disabled) onClick()
      }}
      disabled={disabled}
      title={title}
      className={`
        w-full flex items-center gap-2.5 px-3 py-1.5 text-sm text-left transition-colors
        ${disabled
          ? 'text-gray-300 dark:text-gray-600 cursor-not-allowed'
          : danger
            ? 'text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/30'
            : 'text-gray-700 dark:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700'
        }
      `}
    >
      <span className="w-4 h-4 flex items-center justify-center flex-shrink-0">{icon}</span>
      <span className="flex-1">{label}</span>
      {hasChevron && (
        <svg className="w-3 h-3 text-gray-400 flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
        </svg>
      )}
    </button>
  )
}

function Divider() {
  return <div className="my-1 border-t border-gray-200 dark:border-gray-700" />
}

function OpenIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
    </svg>
  )
}

function EditIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
    </svg>
  )
}

function DeleteIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
    </svg>
  )
}

function AddToGroupIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
    </svg>
  )
}

function RemoveFromGroupIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M18 12H6" />
    </svg>
  )
}

function FolderIcon() {
  return (
    <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 19a2 2 0 01-2-2V5a2 2 0 012-2h4l2 3h4a2 2 0 012 2v9a2 2 0 01-2 2H5z" />
    </svg>
  )
}
