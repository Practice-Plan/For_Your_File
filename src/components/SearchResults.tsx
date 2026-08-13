import { useRef, useEffect, useCallback } from 'react'
import { useTranslation } from 'react-i18next'
import { motion, AnimatePresence } from 'framer-motion'
import { SearchResultItem } from './SearchResultItem'
import type { SearchResult } from '../types'

interface SearchResultsProps {
  results: SearchResult[]
  query: string
  isLoading: boolean
  hasMore: boolean
  onLoadMore: () => void
  selectedIndex: number
  onSelectedIndexChange: (index: number) => void
  onItemSelect: (result: SearchResult) => void
  onItemOpen: (result: SearchResult) => void
  onItemContextMenu?: (result: SearchResult, x: number, y: number) => void
}

/**
 * Search results list with virtual scrolling and keyboard navigation
 */
export function SearchResults({
  results,
  query,
  isLoading,
  hasMore,
  onLoadMore,
  selectedIndex,
  onSelectedIndexChange,
  onItemSelect,
  onItemOpen,
  onItemContextMenu,
}: SearchResultsProps) {
  const { t } = useTranslation()
  const containerRef = useRef<HTMLDivElement>(null)
  const loadMoreRef = useRef<HTMLDivElement>(null)

  // Handle infinite scroll
  useEffect(() => {
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0].isIntersecting && hasMore && !isLoading) {
          onLoadMore()
        }
      },
      { threshold: 0.1 }
    )

    if (loadMoreRef.current) {
      observer.observe(loadMoreRef.current)
    }

    return () => observer.disconnect()
  }, [hasMore, isLoading, onLoadMore])

  // Scroll selected item into view
  useEffect(() => {
    if (containerRef.current && selectedIndex >= 0) {
      const selectedElement = containerRef.current.querySelector(
        `[data-index="${selectedIndex}"]`
      ) as HTMLElement

      if (selectedElement) {
        selectedElement.scrollIntoView({
          block: 'nearest',
          behavior: 'smooth',
        })
      }
    }
  }, [selectedIndex])

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (results.length === 0) return

      switch (e.key) {
        case 'ArrowDown':
          e.preventDefault()
          onSelectedIndexChange(Math.min(selectedIndex + 1, results.length - 1))
          break
        case 'ArrowUp':
          e.preventDefault()
          onSelectedIndexChange(Math.max(selectedIndex - 1, 0))
          break
        case 'Enter':
          e.preventDefault()
          if (selectedIndex >= 0 && selectedIndex < results.length) {
            onItemOpen(results[selectedIndex])
          }
          break
      }
    },
    [results, selectedIndex, onSelectedIndexChange, onItemOpen]
  )

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])

  // Empty state
  if (!query.trim()) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="flex flex-col items-center justify-center py-20 text-gray-500 dark:text-gray-400"
      >
        <svg
          className="w-16 h-16 mb-4 text-gray-300 dark:text-gray-600"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
          />
        </svg>
        <p className="text-lg font-medium">{t('search.emptyTitle')}</p>
        <p className="text-sm mt-2">{t('search.emptySubtitle')}</p>
      </motion.div>
    )
  }

  // No results state
  if (!isLoading && results.length === 0) {
    return (
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        className="flex flex-col items-center justify-center py-20 text-gray-500 dark:text-gray-400"
      >
        <svg
          className="w-16 h-16 mb-4 text-gray-300 dark:text-gray-600"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M9.172 16.172a4 4 0 015.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <p className="text-lg font-medium">{t('search.noResults')}</p>
        <p className="text-sm mt-2">{t('search.noResultsFor', { query })}</p>
      </motion.div>
    )
  }

  return (
    <div
      ref={containerRef}
      className="flex-1 overflow-y-auto overflow-x-hidden"
      role="listbox"
      aria-label={t('search.resultsAria')}
    >
      <AnimatePresence>
        {results.map((result, index) => (
          <div
            key={result.entry.id ?? index}
            data-index={index}
            draggable={result.entry.id !== null}
            onDragStart={(e) => {
              if (result.entry.id !== null) {
                e.dataTransfer.setData('text/plain', String(result.entry.id))
                e.dataTransfer.effectAllowed = 'link'
              }
            }}
            onDragEnd={(e) => {
              e.currentTarget.classList.remove('opacity-50')
            }}
            onDrag={(e) => {
              e.currentTarget.classList.add('opacity-50')
            }}
            onContextMenu={(e) => {
              if (onItemContextMenu) {
                e.preventDefault()
                onSelectedIndexChange(index)
                onItemContextMenu(result, e.clientX, e.clientY)
              }
            }}
          >
            <SearchResultItem
              id={String(result.entry.id)}
              lnkPath={result.entry.lnk_path}
              targetPath={result.entry.target_path}
              tags={result.entry.tags}
              notes={result.entry.notes}
              frequency={result.entry.frequency}
              lastOpened={result.entry.last_opened ? new Date(result.entry.last_opened).toISOString() : null}
              query={query}
              isSelected={index === selectedIndex}
              onClick={() => {
                onSelectedIndexChange(index)
                onItemSelect(result)
              }}
              onDoubleClick={() => onItemOpen(result)}
            />
          </div>
        ))}
      </AnimatePresence>

      {/* Load more indicator */}
      <div ref={loadMoreRef} className="py-4 flex justify-center">
        {isLoading && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="flex items-center gap-2 text-sm text-gray-500 dark:text-gray-400"
          >
            <svg
              className="w-4 h-4 animate-spin"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 2.042.784 3.902 2.058 5.291l1.942-1.942z"
              />
            </svg>
            <span>{t('search.loadingMore')}</span>
          </motion.div>
        )}

        {!hasMore && results.length > 0 && (
          <div className="text-sm text-gray-400 dark:text-gray-600">
            {t('search.totalResults', { count: results.length })}
          </div>
        )}
      </div>
    </div>
  )
}