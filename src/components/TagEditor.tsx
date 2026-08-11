import { useState, useRef, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import { useTags } from '../hooks/useTags'

interface TagEditorProps {
  initialTags?: string[]
  existingTags?: string[]
  onChange?: (tags: string[]) => void
  className?: string
}

export function TagEditor({
  initialTags = [],
  existingTags = [],
  onChange,
  className = '',
}: TagEditorProps) {
  const [isEditing, setIsEditing] = useState(false)
  const inputRef = useRef<HTMLInputElement>(null)

  const {
    tags,
    input,
    suggestions,
    error,
    addTag,
    removeTag,
    setInput,
  } = useTags(initialTags, { existingTags })

  useEffect(() => {
    if (onChange) {
      onChange(tags)
    }
  }, [tags, onChange])

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim()) {
      e.preventDefault()
      addTag(input)
    } else if (e.key === 'Backspace' && !input && tags.length > 0) {
      removeTag(tags[tags.length - 1])
    } else if (e.key === 'Escape') {
      setIsEditing(false)
      setInput('')
    }
  }

  const handleSuggestionClick = (tag: string) => {
    addTag(tag)
    inputRef.current?.focus()
  }

  return (
    <div className={className}>
      <div className="flex flex-wrap gap-2 items-center">
        <AnimatePresence mode="popLayout">
          {tags.map(tag => (
            <motion.span
              key={tag}
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              exit={{ opacity: 0, scale: 0.8 }}
              className="inline-flex items-center gap-1 px-2 py-1 bg-primary-100 dark:bg-primary-900 text-primary-800 dark:text-primary-200 text-sm rounded"
            >
              {tag}
              <button
                onClick={() => removeTag(tag)}
                className="hover:bg-primary-200 dark:hover:bg-primary-800 rounded p-0.5 transition-colors"
                aria-label={`Remove tag ${tag}`}
              >
                <svg className="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
                  <path
                    fillRule="evenodd"
                    d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                    clipRule="evenodd"
                  />
                </svg>
              </button>
            </motion.span>
          ))}
        </AnimatePresence>

        {isEditing ? (
          <div className="relative flex-1 min-w-[120px]">
            <input
              ref={inputRef}
              type="text"
              value={input}
              onChange={e => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              onBlur={() => {
                if (!input.trim()) {
                  setIsEditing(false)
                }
              }}
              placeholder="Add tag..."
              className="w-full px-2 py-1 text-sm bg-transparent border border-gray-300 dark:border-gray-600 rounded focus:outline-none focus:ring-1 focus:ring-primary-500"
              autoFocus
            />

            {suggestions.length > 0 && (
              <motion.div
                initial={{ opacity: 0, y: -5 }}
                animate={{ opacity: 1, y: 0 }}
                className="absolute top-full left-0 right-0 mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded shadow-lg z-10"
              >
                {suggestions.map(suggestion => (
                  <button
                    key={suggestion}
                    onMouseDown={e => {
                      e.preventDefault()
                      handleSuggestionClick(suggestion)
                    }}
                    className="w-full text-left px-3 py-1.5 text-sm hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                  >
                    {suggestion}
                  </button>
                ))}
              </motion.div>
            )}

            {error && (
              <motion.p
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                className="text-xs text-red-600 dark:text-red-400 mt-1"
              >
                {error}
              </motion.p>
            )}
          </div>
        ) : (
          <button
            onClick={() => setIsEditing(true)}
            className="px-2 py-1 text-sm text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800 rounded transition-colors"
          >
            + Add tag
          </button>
        )}
      </div>
    </div>
  )
}

export function TagList({ tags }: { tags?: string }) {
  if (!tags) return null

  const tagArray = tags.split(',').map(t => t.trim()).filter(Boolean)

  return (
    <div className="flex flex-wrap gap-1">
      {tagArray.map(tag => (
        <span
          key={tag}
          className="px-2 py-0.5 bg-gray-100 dark:bg-gray-800 text-gray-700 dark:text-gray-300 text-xs rounded"
        >
          {tag}
        </span>
      ))}
    </div>
  )
}