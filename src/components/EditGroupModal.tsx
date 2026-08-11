/**
 * Modal for editing an existing group
 */
import { useState, useEffect } from 'react'
import { motion } from 'framer-motion'
import type { GroupWithCount } from '../types'

interface EditGroupModalProps {
  isOpen: boolean
  group: GroupWithCount | null
  onClose: () => void
  onSave: (groupId: number, name: string, color: string) => void
  onDelete: (groupId: number) => void
  existingNames: string[]
}

// Preset colors for quick selection
const PRESET_COLORS = [
  '#3498db', // Blue
  '#e74c3c', // Red
  '#2ecc71', // Green
  '#f1c40f', // Yellow
  '#9b59b6', // Purple
  '#1abc9c', // Teal
  '#e67e22', // Orange
  '#34495e', // Dark Gray
  '#f39c12', // Gold
  '#00bcd4', // Cyan
]

export function EditGroupModal({
  isOpen,
  group,
  onClose,
  onSave,
  onDelete,
  existingNames,
}: EditGroupModalProps) {
  const [name, setName] = useState('')
  const [color, setColor] = useState(PRESET_COLORS[0])
  const [error, setError] = useState('')
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)

  // Initialize form when group changes
  useEffect(() => {
    if (group) {
      setName(group.name)
      setColor(group.color)
      setError('')
      setShowDeleteConfirm(false)
    }
  }, [group])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()

    if (!group?.id) return

    const trimmedName = name.trim()

    if (!trimmedName) {
      setError('Group name is required')
      return
    }

    // Check for duplicate name (excluding current group)
    if (existingNames.some(n => n.toLowerCase() === trimmedName.toLowerCase() && n.toLowerCase() !== group.name.toLowerCase())) {
      setError('A group with this name already exists')
      return
    }

    onSave(group.id, trimmedName, color)
    onClose()
  }

  const handleDelete = () => {
    if (!group?.id) return
    onDelete(group.id)
    onClose()
  }

  if (!isOpen || !group) return null

  return (
    <>
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black/50 z-40"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="fixed inset-0 flex items-center justify-center z-50 pointer-events-none">
        <motion.div
          initial={{ opacity: 0, scale: 0.95, y: -20 }}
          animate={{ opacity: 1, scale: 1, y: 0 }}
          exit={{ opacity: 0, scale: 0.95, y: -20 }}
          className="pointer-events-auto bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-sm mx-4"
        >
          <form onSubmit={handleSubmit}>
            {/* Header */}
            <div className="flex items-center justify-between px-4 py-3 border-b border-gray-200 dark:border-gray-700">
              <h2 className="text-lg font-semibold text-gray-900 dark:text-gray-100">
                Edit Group
              </h2>
              <button
                type="button"
                onClick={onClose}
                className="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors"
              >
                <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>

            {/* Body */}
            <div className="px-4 py-4 space-y-4">
              {/* Name input */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                  Name
                </label>
                <input
                  type="text"
                  value={name}
                  onChange={(e) => {
                    setName(e.target.value)
                    setError('')
                  }}
                  placeholder="Enter group name"
                  className="w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100 focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                  autoFocus
                />
                {error && (
                  <p className="mt-1 text-sm text-red-500">{error}</p>
                )}
              </div>

              {/* Color picker */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                  Color
                </label>

                {/* Preset colors */}
                <div className="flex flex-wrap gap-2">
                  {PRESET_COLORS.map((presetColor) => (
                    <button
                      key={presetColor}
                      type="button"
                      onClick={() => setColor(presetColor)}
                      className={`w-7 h-7 rounded-full ring-2 ring-offset-2 ring-offset-white dark:ring-offset-gray-800 transition-transform ${
                        color === presetColor
                          ? 'ring-primary-500 scale-110'
                          : 'ring-transparent hover:scale-105'
                      }`}
                      style={{ backgroundColor: presetColor }}
                    />
                  ))}
                </div>

                {/* Custom color input */}
                <div className="mt-3 flex items-center gap-2">
                  <label className="text-xs text-gray-500 dark:text-gray-400">Custom:</label>
                  <input
                    type="color"
                    value={color}
                    onChange={(e) => setColor(e.target.value)}
                    className="w-8 h-8 rounded cursor-pointer"
                  />
                  <input
                    type="text"
                    value={color}
                    onChange={(e) => setColor(e.target.value)}
                    className="flex-1 px-2 py-1 text-xs border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-700 text-gray-900 dark:text-gray-100"
                    placeholder="#000000"
                  />
                </div>
              </div>

              {/* Entry count */}
              <div className="text-xs text-gray-500 dark:text-gray-400">
                {group.entry_count} entries in this group
              </div>

              {/* Delete confirmation */}
              {showDeleteConfirm ? (
                <div className="p-3 bg-red-50 dark:bg-red-900/20 rounded-lg">
                  <p className="text-sm text-red-800 dark:text-red-200 mb-2">
                    Are you sure you want to delete this group?
                    Entries will not be deleted.
                  </p>
                  <div className="flex gap-2">
                    <button
                      type="button"
                      onClick={() => setShowDeleteConfirm(false)}
                      className="flex-1 px-3 py-1.5 text-sm font-medium text-gray-700 dark:text-gray-300 bg-white dark:bg-gray-800 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition-colors"
                    >
                      Cancel
                    </button>
                    <button
                      type="button"
                      onClick={handleDelete}
                      className="flex-1 px-3 py-1.5 text-sm font-medium text-white bg-red-600 hover:bg-red-700 rounded transition-colors"
                    >
                      Delete
                    </button>
                  </div>
                </div>
              ) : (
                <button
                  type="button"
                  onClick={() => setShowDeleteConfirm(true)}
                  className="text-sm text-red-500 hover:text-red-600 dark:text-red-400 dark:hover:text-red-300 transition-colors"
                >
                  Delete group
                </button>
              )}
            </div>

            {/* Footer */}
            <div className="flex justify-end gap-2 px-4 py-3 border-t border-gray-200 dark:border-gray-700">
              <button
                type="button"
                onClick={onClose}
                className="px-4 py-2 text-sm font-medium text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition-colors"
              >
                Cancel
              </button>
              <button
                type="submit"
                className="px-4 py-2 text-sm font-medium text-white bg-primary-600 hover:bg-primary-700 rounded-lg transition-colors"
              >
                Save
              </button>
            </div>
          </form>
        </motion.div>
      </div>
    </>
  )
}