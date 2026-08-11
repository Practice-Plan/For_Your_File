/**
 * Highlight matched keywords in text by wrapping them in a span
 *
 * @param text - The text to highlight
 * @param query - The search query (space-separated keywords)
 * @returns React element with highlighted keywords
 */
export function highlightText(text: string, query: string): React.ReactNode {
  if (!query.trim()) {
    return text
  }

  const keywords = query.toLowerCase().trim().split(/\s+/).filter(Boolean)
  if (keywords.length === 0) {
    return text
  }

  const regexPattern = keywords
    .map(keyword => keyword.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'))
    .join('|')
  const regex = new RegExp(`(${regexPattern})`, 'gi')

  const parts = text.split(regex)

  return parts.map((part, index) => {
    if (index % 2 === 1) {
      return (
        <mark
          key={index}
          className="bg-primary-100 dark:bg-primary-900 text-primary-900 dark:text-primary-100 px-0.5 rounded"
        >
          {part}
        </mark>
      )
    }
    return part
  })
}

/**
 * Format a date to relative time (e.g., "2 hours ago", "3 days ago")
 *
 * @param date - The date to format
 * @returns Relative time string
 */
export function formatRelativeTime(date: Date | string | null | undefined): string {
  if (!date) {
    return 'Never'
  }

  const now = new Date()
  const past = new Date(date)
  const diffMs = now.getTime() - past.getTime()
  const diffSeconds = Math.floor(diffMs / 1000)
  const diffMinutes = Math.floor(diffSeconds / 60)
  const diffHours = Math.floor(diffMinutes / 60)
  const diffDays = Math.floor(diffHours / 24)
  const diffWeeks = Math.floor(diffDays / 7)
  const diffMonths = Math.floor(diffDays / 30)
  const diffYears = Math.floor(diffDays / 365)

  if (diffSeconds < 60) {
    return 'Just now'
  }
  if (diffMinutes < 60) {
    return `${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`
  }
  if (diffHours < 24) {
    return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`
  }
  if (diffDays < 7) {
    return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`
  }
  if (diffWeeks < 4) {
    return `${diffWeeks} week${diffWeeks > 1 ? 's' : ''} ago`
  }
  if (diffMonths < 12) {
    return `${diffMonths} month${diffMonths > 1 ? 's' : ''} ago`
  }
  return `${diffYears} year${diffYears > 1 ? 's' : ''} ago`
}

/**
 * Format usage frequency for display
 *
 * @param frequency - The frequency count
 * @returns Object with display text and intensity level (0-4)
 */
export function formatFrequency(frequency: number): {
  text: string
  intensity: number
  barWidth: string
} {
  if (frequency === 0) {
    return { text: 'Never used', intensity: 0, barWidth: '0%' }
  }
  if (frequency <= 5) {
    return { text: `${frequency}× used`, intensity: 1, barWidth: '25%' }
  }
  if (frequency <= 20) {
    return { text: `${frequency}× used`, intensity: 2, barWidth: '50%' }
  }
  if (frequency <= 50) {
    return { text: `${frequency}× used`, intensity: 3, barWidth: '75%' }
  }
  return { text: `${frequency}× used`, intensity: 4, barWidth: '100%' }
}

/**
 * Parse tags string into array
 *
 * @param tagsString - Comma-separated tags string
 * @returns Array of tags
 */
export function parseTags(tagsString: string | null | undefined): string[] {
  if (!tagsString) {
    return []
  }
  return tagsString
    .split(',')
    .map(tag => tag.trim())
    .filter(Boolean)
}

/**
 * Get file name from path
 *
 * @param path - Full file path
 * @returns File name
 */
export function getFileName(path: string): string {
  return path.split(/[\\/]/).pop() || path
}

/**
 * Truncate path to show only relevant parts
 *
 * @param path - Full path
 * @param maxLength - Maximum length
 * @returns Truncated path
 */
export function truncatePath(path: string, maxLength: number = 60): string {
  if (path.length <= maxLength) {
    return path
  }

  const fileName = getFileName(path)
  const parts = path.split(/[\\/]/)

  // Keep the filename and show as much of the path as possible
  if (fileName.length >= maxLength - 3) {
    return '...' + fileName.slice(-(maxLength - 3))
  }

  const remaining = maxLength - fileName.length - 4 // 4 for '.../'
  let truncated = ''

  for (let i = 0; i < parts.length - 1; i++) {
    const part = parts[i]
    if (truncated.length + part.length + 1 > remaining) {
      break
    }
    truncated += (truncated ? '\\' : '') + part
  }

  return truncated + '\\...\\' + fileName
}