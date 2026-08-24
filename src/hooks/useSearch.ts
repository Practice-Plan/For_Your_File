import { useState, useEffect, useCallback, useRef } from 'react'
// @ts-ignore - Tauri API may not be available during development
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, PaginatedResults, Entry } from '../types'

interface UseSearchOptions {
  /** Debounce delay in milliseconds */
  debounceDelay?: number
  /** Maximum number of recent searches kept in history */
  maxRecentSearches?: number
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
 * Implements debouncing and pagination.
 *
 * NOTE: This hook previously cached search results in memory by query
 * string. That cache was removed: entries created/updated/deleted after a
 * search would not appear in subsequent searches of the same query (the
 * cache served stale results), while the database preview — which always
 * reads fresh from SQLite — showed them. Local FTS queries are fast enough
 * that caching is unnecessary; the debounce already limits query rate.
 */
export function useSearch(
  query: string,
  options: UseSearchOptions = {}
): UseSearchReturn {
  const {
    debounceDelay = 300,
    maxRecentSearches = 10,
  } = options

  const [results, setResults] = useState<SearchResult[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [totalCount, setTotalCount] = useState(0)
  const [hasMore, setHasMore] = useState(false)
  const [recentSearches, setRecentSearches] = useState<string[]>([])

  const offsetRef = useRef(0)
  const limitRef = useRef(50)
  const requestIdRef = useRef(0)

  // Load recent searches from localStorage
  useEffect(() => {
    const cached = localStorage.getItem('recentSearches')
    if (cached) {
      try {
        setRecentSearches(JSON.parse(cached))
      } catch (e) {
        console.error('Failed to parse recent searches:', e)
      }
    }
  }, [])

  // Perform search
  const performSearch = useCallback(
    async (searchQuery: string, offset: number = 0) => {
      if (!searchQuery.trim()) {
        setResults([])
        setTotalCount(0)
        setHasMore(false)
        return
      }

      // Guard against out-of-order responses from rapid re-searches
      const requestId = ++requestIdRef.current

      setIsLoading(true)
      setError(null)

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

        if (requestId !== requestIdRef.current) {
          // A newer search has started; discard this stale response
          return
        }

        const paginatedResults: PaginatedResults = {
          results: response.results.map(entry => ({ entry, score: 1.0 })),
          total_count: response.total_count,
          offset: response.offset,
          limit: response.limit,
        }

        // Functional update: avoids depending on `results` state inside
        // this callback, which previously caused an implicit re-search
        // loop (every result change recreated the debounced effect).
        setResults(prevResults =>
          offset === 0
            ? paginatedResults.results
            : [...prevResults, ...paginatedResults.results]
        )
        setTotalCount(paginatedResults.total_count)
        setHasMore(
          offset + paginatedResults.results.length < paginatedResults.total_count
        )
        offsetRef.current = offset
      } catch (err) {
        console.error('Search failed:', err)
        if (requestId === requestIdRef.current) {
          setError(err instanceof Error ? err.message : 'Search failed')
        }
      } finally {
        if (requestId === requestIdRef.current) {
          setIsLoading(false)
        }
      }
    },
    []
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
    if (query.trim() && results.length > 0) {
      const recent = [
        query,
        ...recentSearches.filter(s => s !== query),
      ].slice(0, maxRecentSearches)

      setRecentSearches(recent)
      localStorage.setItem('recentSearches', JSON.stringify(recent))
    }
  }, [query, results, recentSearches, maxRecentSearches])

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
    requestIdRef.current++
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
