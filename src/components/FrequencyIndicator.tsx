import { motion } from 'framer-motion'

interface FrequencyIndicatorProps {
  frequency: number
  maxFrequency?: number
  showLabel?: boolean
  className?: string
}

export function FrequencyIndicator({
  frequency,
  maxFrequency = 100,
  showLabel = true,
  className = '',
}: FrequencyIndicatorProps) {
  const percentage = Math.min((frequency / maxFrequency) * 100, 100)

  const getColorClass = () => {
    if (frequency >= 50) return 'bg-green-500'
    if (frequency >= 20) return 'bg-blue-500'
    if (frequency >= 10) return 'bg-yellow-500'
    return 'bg-gray-400'
  }

  return (
    <div className={`flex items-center gap-2 ${className}`}>
      {showLabel && (
        <span className="text-xs text-gray-600 dark:text-gray-400 min-w-[3rem]">
          {frequency}x
        </span>
      )}
      <div className="flex-1 h-1.5 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
        <motion.div
          initial={{ width: 0 }}
          animate={{ width: `${percentage}%` }}
          transition={{ duration: 0.5, ease: 'easeOut' }}
          className={`h-full rounded-full ${getColorClass()}`}
        />
      </div>
    </div>
  )
}

export function FrequencyBadge({ frequency }: { frequency: number }) {
  const getLabel = () => {
    if (frequency >= 50) return 'Frequently Used'
    if (frequency >= 20) return 'Often Used'
    if (frequency >= 10) return 'Sometimes Used'
    return 'Rarely Used'
  }

  const getColorClass = () => {
    if (frequency >= 50) return 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200'
    if (frequency >= 20) return 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200'
    if (frequency >= 10) return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200'
    return 'bg-gray-100 text-gray-800 dark:bg-gray-800 dark:text-gray-200'
  }

  if (frequency === 0) return null

  return (
    <span className={`px-2 py-0.5 text-xs rounded ${getColorClass()}`}>
      {getLabel()}
    </span>
  )
}