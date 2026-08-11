import { motion, AnimatePresence } from 'framer-motion'
import { SortMethod, SORT_METHOD_LABELS, ScoreBreakdown } from '../types'

interface SortingIndicatorProps {
  /** Current sort method */
  method: SortMethod
  /** Number of results */
  resultCount: number
  /** Whether results are loading */
  isLoading?: boolean
  /** Show score breakdown on hover (debug mode) */
  scoreBreakdown?: ScoreBreakdown
  /** Show the indicator */
  show?: boolean
}

/**
 * Sorting indicator showing current sort method in the search bar
 */
export function SortingIndicator({
  method,
  resultCount,
  isLoading = false,
  scoreBreakdown,
  show = true,
}: SortingIndicatorProps) {
  const methodColors: Record<SortMethod, string> = {
    relevance: 'text-purple-600 dark:text-purple-400',
    most_used: 'text-blue-600 dark:text-blue-400',
    recently_opened: 'text-green-600 dark:text-green-400',
    alphabetical: 'text-gray-600 dark:text-gray-400',
    custom: 'text-orange-600 dark:text-orange-400',
  }

  const methodIcons: Record<SortMethod, string> = {
    relevance: '🎯',
    most_used: '📊',
    recently_opened: '🕐',
    alphabetical: '🔤',
    custom: '⚙️',
  }

  return (
    <AnimatePresence>
      {show && (
        <motion.div
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          exit={{ opacity: 0, scale: 0.95 }}
          transition={{ duration: 0.15 }}
          className="flex items-center gap-2 px-2 py-1 bg-gray-100 dark:bg-gray-800 rounded-md text-xs group relative"
        >
          {/* Method icon */}
          <span className="text-base">{methodIcons[method]}</span>

          {/* Method label */}
          <span className={`font-medium ${methodColors[method]}`}>
            {SORT_METHOD_LABELS[method]}
          </span>

          {/* Separator */}
          <span className="text-gray-300 dark:text-gray-600">|</span>

          {/* Result count or loading indicator */}
          {isLoading ? (
            <motion.div
              animate={{ opacity: [0.5, 1, 0.5] }}
              transition={{ duration: 1, repeat: Infinity }}
              className="text-gray-500"
            >
              Loading...
            </motion.div>
          ) : (
            <span className="text-gray-500 dark:text-gray-400">
              {resultCount} result{resultCount !== 1 ? 's' : ''}
            </span>
          )}

          {/* Score breakdown tooltip (debug mode) */}
          {scoreBreakdown && (
            <div className="absolute bottom-full mb-2 left-0 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-50">
              <div className="bg-gray-900 dark:bg-gray-700 text-white text-xs rounded-lg shadow-lg p-3 min-w-[160px]">
                <div className="font-medium mb-2 text-gray-300">Score Breakdown</div>
                <div className="space-y-1">
                  <div className="flex justify-between">
                    <span className="text-blue-400">Frequency:</span>
                    <span className="tabular-nums">
                      {scoreBreakdown.frequency_score.toFixed(3)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-green-400">Recency:</span>
                    <span className="tabular-nums">
                      {scoreBreakdown.recency_score.toFixed(3)}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span className="text-purple-400">Relevance:</span>
                    <span className="tabular-nums">
                      {scoreBreakdown.relevance_score.toFixed(3)}
                    </span>
                  </div>
                  <div className="border-t border-gray-600 mt-2 pt-2 flex justify-between font-medium">
                    <span>Total:</span>
                    <span className="tabular-nums">
                      {scoreBreakdown.total_score.toFixed(3)}
                    </span>
                  </div>
                </div>
              </div>
            </div>
          )}
        </motion.div>
      )}
    </AnimatePresence>
  )
}

export default SortingIndicator