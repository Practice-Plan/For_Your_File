import { useState, useRef, useEffect } from 'react'
import { motion, AnimatePresence } from 'framer-motion'
import {
  SortMethod,
  SortingWeights,
  SORT_METHOD_LABELS,
  validateSortingWeights,
} from '../types'
import { WeightAdjuster } from './WeightAdjuster'

interface SortOptionsProps {
  /** Current sort method */
  currentMethod: SortMethod
  /** Current custom weights */
  weights: SortingWeights
  /** Callback when sort method changes */
  onMethodChange: (method: SortMethod) => void
  /** Callback when weights change */
  onWeightsChange: (weights: SortingWeights) => void
  /** Whether to show debug mode toggle */
  showDebugToggle?: boolean
  /** Current debug mode state */
  debugMode?: boolean
  /** Callback when debug mode changes */
  onDebugModeChange?: (enabled: boolean) => void
}

/**
 * Sort options dropdown with method selector and weight adjuster
 */
export function SortOptions({
  currentMethod,
  weights,
  onMethodChange,
  onWeightsChange,
  showDebugToggle = false,
  debugMode = false,
  onDebugModeChange,
}: SortOptionsProps) {
  const [isOpen, setIsOpen] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)

  // Close on click outside
  useEffect(() => {
    function handleClickOutside(event: MouseEvent) {
      if (
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        setIsOpen(false)
      }
    }

    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const handleMethodSelect = (method: SortMethod) => {
    onMethodChange(method)
    setIsOpen(false)
  }

  const handleWeightChange = (newWeights: SortingWeights) => {
    if (validateSortingWeights(newWeights)) {
      onWeightsChange(newWeights)
    }
  }

  const methods: SortMethod[] = [
    'relevance',
    'most_used',
    'recently_opened',
    'alphabetical',
    'custom',
  ]

  return (
    <div ref={containerRef} className="relative">
      {/* Trigger button */}
      <button
        type="button"
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center gap-2 px-3 py-1.5 text-sm bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
        aria-expanded={isOpen}
        aria-haspopup="listbox"
      >
        <svg
          className="w-4 h-4 text-gray-500"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M3 4h13M3 8h9m-9 4h6m4 0l4-4m0 0l4 4m-4-4v12"
          />
        </svg>
        <span className="font-medium">{SORT_METHOD_LABELS[currentMethod]}</span>
        <svg
          className={`w-4 h-4 text-gray-400 transition-transform ${isOpen ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M19 9l-7 7-7-7"
          />
        </svg>
      </button>

      {/* Dropdown */}
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ opacity: 0, y: -8 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.15 }}
            className="absolute top-full mt-2 right-0 w-64 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg overflow-hidden z-50"
            role="listbox"
          >
            {/* Method list */}
            <div className="py-1">
              {methods.map((method) => (
                <button
                  key={method}
                  type="button"
                  onClick={() => handleMethodSelect(method)}
                  className={`w-full flex items-center justify-between px-4 py-2 text-sm transition-colors ${
                    currentMethod === method
                      ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400'
                      : 'hover:bg-gray-50 dark:hover:bg-gray-700'
                  }`}
                  role="option"
                  aria-selected={currentMethod === method}
                >
                  <span>{SORT_METHOD_LABELS[method]}</span>
                  {currentMethod === method && (
                    <svg
                      className="w-4 h-4"
                      fill="currentColor"
                      viewBox="0 0 20 20"
                    >
                      <path
                        fillRule="evenodd"
                        d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z"
                        clipRule="evenodd"
                      />
                    </svg>
                  )}
                </button>
              ))}
            </div>

            {/* Custom weights panel */}
            {currentMethod === 'custom' && (
              <div className="border-t border-gray-200 dark:border-gray-700 p-3">
                <WeightAdjuster
                  weights={weights}
                  onChange={handleWeightChange}
                  showLabels={true}
                />
              </div>
            )}

            {/* Debug toggle */}
            {showDebugToggle && onDebugModeChange && (
              <div className="border-t border-gray-200 dark:border-gray-700 px-4 py-2">
                <label className="flex items-center gap-2 text-sm text-gray-600 dark:text-gray-400 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={debugMode}
                    onChange={(e) => onDebugModeChange(e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span>Show score breakdown</span>
                </label>
              </div>
            )}
          </motion.div>
        )}
      </AnimatePresence>
    </div>
  )
}

export default SortOptions