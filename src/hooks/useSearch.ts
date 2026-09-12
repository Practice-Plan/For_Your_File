import { useState, useEffect, useCallback, useRef } from 'react'
// @ts-ignore - Tauri API may not be available during development
import { invoke } from '@tauri-apps/api/core'
import type { SearchResult, PaginatedResults, Entry } from '../types'

interface UseSearchOptions {
  /** Debounce delay in milliseconds */
  debounceDelay?: number
  /** Maximum number of recent searches kept in history */
  maxRecentSearches?: number
  /** Maximum number of cached query results */
  maxCacheSize?: number
  /** Cache TTL in milliseconds */
  cacheTTL?: number
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
  /** Invalidate all cached results (call when entries are added/updated/deleted) */
  invalidateCache: () => void
}

/**
 * Custom hook for search functionality with caching and stability guards.
 *
 * Caching strategy:
 *   - Results are cached by query string (LRU eviction).
 *   - When the user types a known query, cached results show immediately
 *     while a fresh search runs in the background. This eliminates the
 *     "loading more" spinner on repeated queries while still keeping
 *     results up-to-date.
 *   - Callers invoke `invalidateCache()` after add/update/delete so stale
 *     entries never appear.
 *
 * Stability:
 *   - Request timeout (30 s) guards against hung IPC under resource pressure.
 *   - Exponential backoff on failure (up to 3 retries) with state reset.
 *   - Stale-response guard with monotonic request ID.
 */
export function useSearch(
  query: string,
  options: UseSearchOptions = {}
): UseSearchReturn {
  const {
    debounceDelay = 300,
    maxRecentSearches = 10,
    maxCacheSize = 50,
    cacheTTL = 5 * 60 * 1000, // 5 minutes
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
  const mountedRef = useRef(true)

  // ── In-memory query cache (LRU by Map insertion order) ────────────
  type CacheEntry = {
    results: SearchResult[]
    totalCount: number
    hasMore: boolean
    timestamp: number
  }
  const cacheRef = useRef<Map<string, CacheEntry>>(new Map())

  const invalidateCache = useCallback(() => {
    cacheRef.current.clear()
  }, [])

  // Touch a cache entry (move to end of Map = mark most recently used)
  const cacheTouch = (key: string) => {
    const m = cacheRef.current
    if (m.has(key)) {
      const val = m.get(key)!
      m.delete(key)
      m.set(key, val)
    }
  }

  // Insert into cache with LRU eviction
  const cachePut = (key: string, entry: CacheEntry) => {
    const m = cacheRef.current
    if (m.has(key)) m.delete(key)
    m.set(key, entry)
    while (m.size > maxCacheSize) {
      // Map iteration order is insertion order; first key = oldest
      const oldestKey = m.keys().next().value as string
      m.delete(oldestKey)
    }
  }

  const cacheGet = (key: string): CacheEntry | null => {
    const entry = cacheRef.current.get(key)
    if (!entry) return null
    if (Date.now() - entry.timestamp > cacheTTL) {
      cacheRef.current.delete(key)
      return null
    }
    cacheTouch(key)
    return entry
  }

  useEffect(() => {
    mountedRef.current = true
    return () => {
      mountedRef.current = false
    }
  }, [])

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

  // Perform search with timeout and retry
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

      const cached = offset === 0 ? cacheGet(searchQuery) : null
      if (cached && mountedRef.current && requestId === requestIdRef.current) {
        // Show cached results immediately
        setResults(cached.results)
        setTotalCount(cached.totalCount)
        setHasMore(cached.hasMore)
        offsetRef.current = 0
      }

      if (mountedRef.current) setIsLoading(true)

      // Retry loop with exponential backoff
      const maxRetries = 3

      for (let attempt = 0; attempt <= maxRetries; attempt++) {
        if (attempt > 0) {
          const backoff = Math.min(500 * Math.pow(2, attempt - 1), 4000)
          await new Promise(res => setTimeout(res, backoff))
          // Re-check request freshness after backoff
          if (requestId !== requestIdRef.current || !mountedRef.current) return
        }

        try {
          // Use Promise.race with timeout to guard against hung IPC
          const timeoutPromise = new Promise<never>((_, reject) =>
            setTimeout(() => reject(new Error('Search request timed out')), 30000)
          )

          const invokePromise = invoke<{
            results: Entry[]
            total_count: number
            offset: number
            limit: number
          }>('search_entries', {
            query: searchQuery,
            offset,
            limit: limitRef.current,
          })

          const response = await Promise.race([invokePromise, timeoutPromise])

          if (requestId !== requestIdRef.current || !mountedRef.current) {
            return // stale or unmounted
          }

          const paginatedResults: PaginatedResults = {
            results: response.results.map(entry => ({ entry, score: 1.0 })),
            total_count: response.total_count,
            offset: response.offset,
            limit: response.limit,
          }

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

          // Cache full-page results (offset === 0 only)
          if (offset === 0 && paginatedResults.results.length > 0) {
            cachePut(searchQuery, {
              results: paginatedResults.results,
              totalCount: paginatedResults.total_count,
              hasMore:
                offset + paginatedResults.results.length < paginatedResults.total_count,
              timestamp: Date.now(),
            })
          }

          setError(null)
          break // success: exit retry loop
        } catch (err) {
          if (requestId !== requestIdRef.current || !mountedRef.current) return
          if (attempt < maxRetries) continue // retry
          // Final failure
          console.error('Search failed after retries:', err)
          setError(err instanceof Error ? err.message : 'Search failed')
        }
      }

      if (mountedRef.current && requestId === requestIdRef.current) {
        setIsLoading(false)
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

  // Refresh current search (bypasses cache)
  const refresh = useCallback(async () => {
    invalidateCache()
    offsetRef.current = 0
    await performSearch(query, 0)
  }, [query, performSearch, invalidateCache])

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
    invalidateCache,
  }
}
