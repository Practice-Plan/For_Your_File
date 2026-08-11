import { useState, useEffect, useCallback, useRef } from 'react'
// @ts-ignore - Tauri API may not be available during development
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, PaginatedResults, Entry } from '../types'

interface UseSearchOptions {
  /** Debounce delay in milliseconds */
  debounceDelay?: number
  /** Enable caching of recent searches */
  enableCache?: boolean
  /** Maximum cache size */
  maxCacheSize?: number
}

interface UseSearchReturn {
  /** Search results */
  results: SearchResult[]
  /** Loading state */
  isLoading: boolean
  /** Error state */
  error: string | null
  /** Total count of results */
  totalCount: number
  /** Has more results to load */
  hasMore: boolean
  /** Load more results */
  loadMore: () => Promise<void>
  /** Clear search */
  clearSearch: () => void
  /** Recent searches */
  recentSearches: string[]
  /** Refresh current search */
  refresh: () => Promise<void>
}

/**
 * Custom hook for search functionality
 * Connects to Rust backend via Tauri invoke
 * Implements debouncing, caching, and pagination
 */
export function useSearch(
  query: string,
  options: UseSearchOptions = {}
): UseSearchReturn {
  const {
    debounceDelay = 300,
    enableCache = true,
    maxCacheSize = 10,
  } = options

  const [results, setResults] = useState<SearchResult[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [totalCount, setTotalCount] = useState(0)
  const [hasMore, setHasMore] = useState(false)
  const [recentSearches, setRecentSearches] = useState<string[]>([])

  const offsetRef = useRef(0)
  const limitRef = useRef(50)
  const cacheRef = useRef<Map<string, PaginatedResults>>(new Map())

  // Load recent searches from localStorage
  useEffect(() => {
    if (enableCache) {
      const cached = localStorage.getItem('recentSearches')
      if (cached) {
        try {
          setRecentSearches(JSON.parse(cached))
        } catch (e) {
          console.error('Failed to parse recent searches:', e)
        }
      }
    }
  }, [enableCache])

  // Perform search
  const performSearch = useCallback(
    async (searchQuery: string, offset: number = 0) => {
      if (!searchQuery.trim()) {
        setResults([])
        setTotalCount(0)
        setHasMore(false)
        return
      }

      setIsLoading(true)
      setError(null)

      try {
        // Check cache first
        const cacheKey = `${searchQuery}:${offset}`
        if (enableCache && cacheRef.current.has(cacheKey)) {
          const cached = cacheRef.current.get(cacheKey)!
          setResults(offset === 0 ? cached.results : [...results, ...cached.results])
          setTotalCount(cached.total_count)
          setHasMore(cached.results.length === limitRef.current)
          setIsLoading(false)
          return
        }

        // Try to invoke Tauri backend
        let paginatedResults: PaginatedResults

        try {
          const response = await invoke<{
            results: Entry[]
            total_count: number
            offset: number
            limit: number
          }>('search_entries', {
            query: searchQuery,
            offset,
            limit: limitRef.current,
          })

          paginatedResults = {
            results: response.results.map(entry => ({ entry, score: 1.0 })),
            total_count: response.total_count,
            offset: response.offset,
            limit: response.limit,
          }
        } catch (tauriError) {
          // If Tauri backend is not available, use mock data
          console.warn('Tauri backend not available, using mock data:', tauriError)
          paginatedResults = await mockSearch(searchQuery, offset, limitRef.current)
        }

        // Cache results
        if (enableCache) {
          cacheRef.current.set(cacheKey, paginatedResults)
          // Limit cache size
          if (cacheRef.current.size > maxCacheSize) {
            const firstKey = cacheRef.current.keys().next().value
            if (firstKey) {
              cacheRef.current.delete(firstKey)
            }
          }
        }

        setResults(
          offset === 0
            ? paginatedResults.results
            : [...results, ...paginatedResults.results]
        )
        setTotalCount(paginatedResults.total_count)
        setHasMore(
          offset + paginatedResults.results.length < paginatedResults.total_count
        )
        offsetRef.current = offset
      } catch (err) {
        console.error('Search failed:', err)
        setError(err instanceof Error ? err.message : 'Search failed')
      } finally {
        setIsLoading(false)
      }
    },
    [results, enableCache, maxCacheSize]
  )

  // Debounced search effect
  useEffect(() => {
    const timer = setTimeout(() => {
      offsetRef.current = 0
      performSearch(query, 0)
    }, debounceDelay)

    return () => clearTimeout(timer)
  }, [query, debounceDelay, performSearch])

  // Add to recent searches
  useEffect(() => {
    if (query.trim() && results.length > 0 && enableCache) {
      const recent = [
        query,
        ...recentSearches.filter(s => s !== query),
      ].slice(0, maxCacheSize)

      setRecentSearches(recent)
      localStorage.setItem('recentSearches', JSON.stringify(recent))
    }
  }, [query, results, enableCache, recentSearches, maxCacheSize])

  // Refresh current search
  const refresh = useCallback(async () => {
    offsetRef.current = 0
    await performSearch(query, 0)
  }, [query, performSearch])

  // Load more results
  const loadMore = useCallback(async () => {
    if (!hasMore || isLoading) return

    const newOffset = offsetRef.current + limitRef.current
    await performSearch(query, newOffset)
  }, [hasMore, isLoading, query, performSearch])

  // Clear search
  const clearSearch = useCallback(() => {
    setResults([])
    setTotalCount(0)
    setHasMore(false)
    setError(null)
    offsetRef.current = 0
  }, [])

  return {
    results,
    isLoading,
    error,
    totalCount,
    hasMore,
    loadMore,
    clearSearch,
    recentSearches,
    refresh,
  }
}

/**
 * Mock search function for development/testing
 */
async function mockSearch(
  query: string,
  offset: number,
  limit: number
): Promise<PaginatedResults> {
  // Simulate network delay
  await new Promise(resolve => setTimeout(resolve, 200))

  // Mock data
  const mockEntries: Entry[] = [
    {
      id: 1,
      lnk_path: 'C:\\Users\\User\\Desktop\\Chrome.lnk',
      target_path: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe',
      target_type: { type: 'File', path: 'C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe' },
      parameters: '--profile-directory="Default"',
      working_dir: 'C:\\Program Files\\Google\\Chrome\\Application',
      tags: 'browser,web,internet',
      notes: 'Default browser shortcut',
      frequency: 150,
      last_opened: Date.now() - 3600000,
      created_at: Date.now() - 86400000 * 30,
      updated_at: Date.now() - 86400000,
    },
    {
      id: 2,
      lnk_path: 'C:\\Users\\User\\Desktop\\VS Code.lnk',
      target_path: 'C:\\Users\\User\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe',
      target_type: { type: 'File', path: 'C:\\Users\\User\\AppData\\Local\\Programs\\Microsoft VS Code\\Code.exe' },
      parameters: '',
      working_dir: '',
      tags: 'development,editor,code',
      notes: 'Primary code editor',
      frequency: 89,
      last_opened: Date.now() - 7200000,
      created_at: Date.now() - 86400000 * 60,
      updated_at: Date.now() - 86400000 * 2,
    },
    {
      id: 3,
      lnk_path: 'C:\\Users\\User\\Desktop\\Documents.lnk',
      target_path: 'C:\\Users\\User\\Documents',
      target_type: { type: 'Folder', path: 'C:\\Users\\User\\Documents' },
      tags: 'folder,documents',
      frequency: 45,
      last_opened: Date.now() - 14400000,
      created_at: Date.now() - 86400000 * 90,
      updated_at: Date.now() - 86400000 * 5,
    },
    {
      id: 4,
      lnk_path: 'C:\\Users\\User\\Desktop\\Project Folder.lnk',
      target_path: 'K:\\Practice Plan\\For_Your_File',
      target_type: { type: 'Folder', path: 'K:\\Practice Plan\\For_Your_File' },
      tags: 'project,development,practice',
      notes: 'Current project directory',
      frequency: 78,
      last_opened: Date.now() - 300000,
      created_at: Date.now() - 86400000 * 10,
      updated_at: Date.now() - 86400000,
    },
  ]

  // Filter by query
  const filtered = mockEntries.filter(entry => {
    const searchLower = query.toLowerCase()
    return (
      entry.lnk_path.toLowerCase().includes(searchLower) ||
      entry.target_path.toLowerCase().includes(searchLower) ||
      (entry.tags && entry.tags.toLowerCase().includes(searchLower)) ||
      (entry.notes && entry.notes.toLowerCase().includes(searchLower))
    )
  })

  // Paginate
  const paginated = filtered.slice(offset, offset + limit)

  return {
    results: paginated.map(entry => ({
      entry,
      score: 1.0,
      snippet: undefined,
    })),
    total_count: filtered.length,
    offset,
    limit,
  }
}