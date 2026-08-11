import { useCallback, useEffect, useState } from 'react'
import { motion } from 'framer-motion'
import { SortingWeights, validateSortingWeights } from '../types'

interface WeightAdjusterProps {
  /** Current weights */
  weights: SortingWeights
  /** Callback when weights change */
  onChange: (weights: SortingWeights) => void
  /** Show labels above sliders */
  showLabels?: boolean
  /** Show total sum indicator */
  showTotal?: boolean
  /** Compact mode for smaller display */
  compact?: boolean
}

/**
 * Weight adjuster component with sliders for frequency, recency, and relevance
 */
export function WeightAdjuster({
  weights,
  onChange,
  showLabels = true,
  showTotal = true,
  compact = false,
}: WeightAdjusterProps) {
  const [localWeights, setLocalWeights] = useState(weights)
  const [isValid, setIsValid] = useState(true)

  // Sync with external weights
  useEffect(() => {
    setLocalWeights(weights)
  }, [weights])

  // Validate weights whenever they change
  useEffect(() => {
    setIsValid(validateSortingWeights(localWeights))
  }, [localWeights])

  const handleWeightChange = useCallback(
    (key: keyof SortingWeights, value: number) => {
      const newWeights = { ...localWeights, [key]: value }

      // Normalize weights so they sum to 1.0
      const sum =
        newWeights.frequency_weight +
        newWeights.recency_weight +
        newWeights.relevance_weight

      if (sum > 0) {
        const normalized = {
          frequency_weight: newWeights.frequency_weight / sum,
          recency_weight: newWeights.recency_weight / sum,
          relevance_weight: newWeights.relevance_weight / sum,
        }
        setLocalWeights(normalized)
        onChange(normalized)
      }
    },
    [localWeights, onChange]
  )

  const total =
    localWeights.frequency_weight +
    localWeights.recency_weight +
    localWeights.relevance_weight

  const weightConfigs: Array<{
      key: keyof SortingWeights
      label: string
      color: string
      icon: string
    }> = [
      {
        key: 'frequency_weight',
        label: 'Frequency',
        color: 'bg-blue-500',
        icon: '📊',
      },
      {
        key: 'recency_weight',
        label: 'Recency',
        color: 'bg-green-500',
        icon: '🕐',
      },
      {
        key: 'relevance_weight',
        label: 'Relevance',
        color: 'bg-purple-500',
        icon: '🎯',
      },
    ]

  return (
    <div className={`space-y-${compact ? '2' : '4'}`}>
      {weightConfigs.map((config) => (
        <div key={config.key} className="space-y-1">
          {showLabels && (
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-600 dark:text-gray-400">
                {config.icon} {config.label}
              </span>
              <span className="font-medium tabular-nums">
                {localWeights[config.key].toFixed(2)}
              </span>
            </div>
          )}

          <div className="relative">
            {/* Background track */}
            <div className="h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              {/* Filled portion */}
              <motion.div
                className={`h-full ${config.color} rounded-full`}
                initial={false}
                animate={{
                  width: `${localWeights[config.key] * 100}%`,
                }}
                transition={{ duration: 0.15 }}
              />
            </div>

            {/* Slider input */}
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={localWeights[config.key]}
              onChange={(e) =>
                handleWeightChange(config.key, parseFloat(e.target.value))
              }
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
              aria-label={config.label}
            />
          </div>
        </div>
      ))}

      {/* Total indicator */}
      {showTotal && (
        <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between text-sm">
            <span className="text-gray-500 dark:text-gray-400">Total</span>
            <div className="flex items-center gap-1.5">
              <span
                className={`font-medium tabular-nums ${
                  isValid ? 'text-green-600 dark:text-green-400' : 'text-red-500'
                }`}
              >
                {total.toFixed(2)}
              </span>
              {isValid && (
                <svg
                  className="w-4 h-4 text-green-500"
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
            </div>
          </div>

          {/* Validation message */}
          {!isValid && (
            <motion.p
              initial={{ opacity: 0, y: -4 }}
              animate={{ opacity: 1, y: 0 }}
              className="text-xs text-red-500 mt-1"
            >
              Weights should sum to 1.0
            </motion.p>
          )}
        </div>
      )}
    </div>
  )
}

export default WeightAdjuster