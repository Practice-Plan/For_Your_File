import { useState, useMemo, useCallback } from 'react'

interface TagValidation {
  isValid: boolean
  error?: string
}

interface UseTagsOptions {
  maxTags?: number
  maxTagLength?: number
  allowedCharacters?: RegExp
  existingTags?: string[]
}

interface UseTagsReturn {
  tags: string[]
  input: string
  suggestions: string[]
  error?: string
  addTag: (tag: string) => boolean
  removeTag: (tag: string) => void
  setInput: (input: string) => void
  validateTag: (tag: string) => TagValidation
  clearTags: () => void
  setTags: (tags: string[]) => void
}

const DEFAULT_ALLOWED_CHARACTERS = /^[\w\u4e00-\u9fa5\-_]+$/

export function useTags(
  initialTags: string[] = [],
  options: UseTagsOptions = {}
): UseTagsReturn {
  const {
    maxTags = 10,
    maxTagLength = 20,
    allowedCharacters = DEFAULT_ALLOWED_CHARACTERS,
    existingTags = [],
  } = options

  const [tags, setTags] = useState<string[]>(initialTags)
  const [input, setInput] = useState('')
  const [error, setError] = useState<string>()

  const validateTag = useCallback(
    (tag: string): TagValidation => {
      if (!tag || tag.trim().length === 0) {
        return { isValid: false, error: 'Tag cannot be empty' }
      }

      if (tag.length > maxTagLength) {
        return {
          isValid: false,
          error: `Tag must be ${maxTagLength} characters or less`,
        }
      }

      if (!allowedCharacters.test(tag)) {
        return {
          isValid: false,
          error: 'Tag contains invalid characters',
        }
      }

      if (tags.includes(tag)) {
        return { isValid: false, error: 'Tag already exists' }
      }

      return { isValid: true }
    },
    [tags, maxTagLength, allowedCharacters]
  )

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

  const addTag = useCallback(
    (tag: string): boolean => {
      const trimmedTag = tag.trim()
      const validation = validateTag(trimmedTag)

      if (!validation.isValid) {
        setError(validation.error)
        return false
      }

      if (tags.length >= maxTags) {
        setError(`Maximum ${maxTags} tags allowed`)
        return false
      }

      setTags(prev => [...prev, trimmedTag])
      setInput('')
      setError(undefined)
      return true
    },
    [tags, maxTags, validateTag]
  )

  const removeTag = useCallback((tag: string) => {
    setTags(prev => prev.filter(t => t !== tag))
    setError(undefined)
  }, [])

  const clearTags = useCallback(() => {
    setTags([])
    setInput('')
    setError(undefined)
  }, [])

  return {
    tags,
    input,
    suggestions,
    error,
    addTag,
    removeTag,
    setInput,
    validateTag,
    clearTags,
    setTags,
  }
}