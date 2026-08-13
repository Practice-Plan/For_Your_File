import { useState, useRef, useEffect, useMemo } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'

interface TagEditorProps {
  /** Controlled tags array — parent is the single source of truth */
  tags: string[]
  /** Called immediately when tags change (add/remove) */
  onTagsChange: (tags: string[]) => void
  /** Existing tags for autocomplete suggestions */
  existingTags?: string[]
  className?: string
}

const MAX_TAGS = 10
const MAX_TAG_LENGTH = 30

/**
 * Controlled tag editor component.
 *
 * The parent owns the `tags` state. All mutations (add/remove) call
 * `onTagsChange` synchronously with the new array, so the parent's state
 * updates in the same React batch — no useEffect sync delay.
 */
export function TagEditor({
  tags,
  onTagsChange,
  existingTags = [],
  className = '',
}: TagEditorProps) {
  const { t } = useTranslation()
  const [isEditing, setIsEditing] = useState(false)
  const [input, setInput] = useState('')
  const [error, setError] = useState<string | undefined>(undefined)
  const inputRef = useRef<HTMLInputElement>(null)

  // Autocomplete suggestions based on current input
  const suggestions = useMemo(() => {
    if (!input.trim()) return []
    const inputLower = input.toLowerCase()
    return existingTags
      .filter(
        tag =>
          tag.toLowerCase().includes(inputLower) &&
          !tags.includes(tag)
      )
      .slice(0, 5)
  }, [input, existingTags, tags])

  // Auto-focus the input when entering edit mode
  useEffect(() => {
    if (isEditing) {
      inputRef.current?.focus()
    }
  }, [isEditing])

  const addTag = (tag: string): boolean => {
    const trimmedTag = tag.trim()

    if (!trimmedTag) {
      return false
    }

    if (trimmedTag.length > MAX_TAG_LENGTH) {
      setError(t('tagEditor.tooLong', { max: MAX_TAG_LENGTH }))
      return false
    }

    if (tags.includes(trimmedTag)) {
      setError(t('tagEditor.alreadyExists'))
      return false
    }

    if (tags.length >= MAX_TAGS) {
      setError(t('tagEditor.tooMany', { max: MAX_TAGS }))
      return false
    }

    // Synchronous update — parent state updates immediately
    onTagsChange([...tags, trimmedTag])
    setInput('')
    setError(undefined)
    return true
  }

  const removeTag = (tag: string) => {
    onTagsChange(tags.filter(t => t !== tag))
    setError(undefined)
  }

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter' && input.trim()) {
      e.preventDefault()
      addTag(input)
    } else if (e.key === 'Backspace' && !input && tags.length > 0) {
      removeTag(tags[tags.length - 1])
    } else if (e.key === 'Escape') {
      setIsEditing(false)
      setInput('')
      setError(undefined)
    }
  }

  const handleBlur = () => {
    // Add the tag on blur if input is non-empty
    if (input.trim()) {
      addTag(input)
    }
    setIsEditing(false)
    setInput('')
    setError(undefined)
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
              onChange={e => {
                setInput(e.target.value)
                setError(undefined)
              }}
              onKeyDown={handleKeyDown}
              onBlur={handleBlur}
              placeholder={t('tagEditor.placeholder')}
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
            {t('tagEditor.addTag')}
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
