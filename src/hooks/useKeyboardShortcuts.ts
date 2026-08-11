import { useEffect, useCallback } from 'react'

interface KeyboardShortcutsConfig {
  onFocusSearch?: () => void
  onClearSearch?: () => void
  onNavigateUp?: () => void
  onNavigateDown?: () => void
  onOpenSelected?: () => void
  onNavigateNext?: () => void
  onNavigatePrev?: () => void
  enabled?: boolean
}

/**
 * Hook for managing keyboard shortcuts
 *
 * Ctrl+K: Focus search box
 * Escape: Clear search and blur
 * Up/Down: Navigate results
 * Enter: Open selected result
 * Tab: Navigate between elements
 */
export function useKeyboardShortcuts(config: KeyboardShortcutsConfig) {
  const {
    onFocusSearch,
    onClearSearch,
    onNavigateUp,
    onNavigateDown,
    onOpenSelected,
    onNavigateNext,
    onNavigatePrev,
    enabled = true,
  } = config

  const handleKeyDown = useCallback(
    (e: KeyboardEvent) => {
      if (!enabled) return

      // Check if we're in an input field (except for Ctrl+K)
      const isInputFocused =
        document.activeElement?.tagName === 'INPUT' ||
        document.activeElement?.tagName === 'TEXTAREA'

      // Ctrl+K or Cmd+K: Focus search
      if ((e.ctrlKey || e.metaKey) && e.key === 'k') {
        e.preventDefault()
        onFocusSearch?.()
        return
      }

      // Escape: Clear search
      if (e.key === 'Escape') {
        e.preventDefault()
        onClearSearch?.()
        return
      }

      // Don't handle navigation if we're in an input field
      if (isInputFocused) return

      // Arrow Up: Navigate up
      if (e.key === 'ArrowUp') {
        e.preventDefault()
        onNavigateUp?.()
        return
      }

      // Arrow Down: Navigate down
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        onNavigateDown?.()
        return
      }

      // Enter: Open selected
      if (e.key === 'Enter') {
        e.preventDefault()
        onOpenSelected?.()
        return
      }

      // Tab: Navigate next
      if (e.key === 'Tab' && !e.shiftKey) {
        onNavigateNext?.()
        return
      }

      // Shift+Tab: Navigate prev
      if (e.key === 'Tab' && e.shiftKey) {
        e.preventDefault()
        onNavigatePrev?.()
        return
      }
    },
    [
      enabled,
      onFocusSearch,
      onClearSearch,
      onNavigateUp,
      onNavigateDown,
      onOpenSelected,
      onNavigateNext,
      onNavigatePrev,
    ]
  )

  useEffect(() => {
    if (enabled) {
      window.addEventListener('keydown', handleKeyDown)
      return () => window.removeEventListener('keydown', handleKeyDown)
    }
  }, [enabled, handleKeyDown])

  return {
    // Expose helper for manual focus
    focusSearch: onFocusSearch,
  }
}