/**
 * Target type for LNK shortcuts
 */
export type LnkTarget = {
  type: 'File' | 'Folder' | 'Url' | 'Unknown'
  path: string
}

/**
 * Entry representing a single LNK shortcut
 */
export interface Entry {
  id: number | null
  lnk_path: string
  target_path: string
  target_type: LnkTarget
  parameters?: string
  working_dir?: string
  description?: string
  icon_location?: string
  icon_index?: number
  tags?: string
  notes?: string
  frequency: number
  last_opened?: number
  created_at: number
  updated_at: number
  group_id?: number
  expires_at?: number
}

/**
 * Group for organizing entries
 */
export interface Group {
  id: number | null
  name: string
  color: string
  created_at: number
  updated_at: number
}

/**
 * Group with entry count for display
 */
export interface GroupWithCount extends Group {
  entry_count: number
}

/**
 * Group export format (for sharing)
 */
export interface GroupExport {
  group: Group
  entry_ids: number[]
  exported_at: number
}

/**
 * LNK File metadata structure
 */
export interface LnkFile {
  id: string
  path: string
  name: string
  targetPath: string
  arguments?: string
  workingDirectory?: string
  description?: string
  iconLocation?: string
  hotKey?: number
  showCommand: 'normal' | 'maximized' | 'minimized'
  creationTime: Date
  modificationTime: Date
  accessTime: Date
  fileSize: number
}

/**
 * Parsed LNK file properties returned by the parse_lnk_file backend command.
 * Used for auto-completing entry fields when a user uploads a .lnk file.
 */
export interface ParsedLnkProperties {
  target_path: string
  arguments?: string
  working_directory?: string
  description?: string
  icon_location?: string
  icon_index?: number
}

/**
 * Batch operation type
 */
export type BatchOperation = 'delete' | 'repair' | 'export' | 'update'

/**
 * Batch job status
 */
export type BatchJobStatus = 'pending' | 'running' | 'completed' | 'failed'

/**
 * Batch job item
 */
export interface BatchJob {
  id: string
  operation: BatchOperation
  files: LnkFile[]
  status: BatchJobStatus
  progress: number
  errors?: BatchJobError[]
  startTime?: Date
  endTime?: Date
}

/**
 * Batch job error
 */
export interface BatchJobError {
  filePath: string
  error: string
  code?: string
}

/**
 * Application settings
 */
export interface AppSettings {
  theme: 'light' | 'dark' | 'system'
  language: 'en' | 'zh-CN'
  defaultOpenPath?: string
  confirmBeforeDelete: boolean
  showHiddenFiles: boolean
}

/**
 * Window state
 */
export interface WindowState {
  isMaximized: boolean
  width: number
  height: number
  x: number
  y: number
}

/**
 * Search result with relevance score
 */
export interface SearchResult {
  entry: Entry
  score: number
  snippet?: string
}

/**
 * Paginated search results
 */
export interface PaginatedResults {
  results: SearchResult[]
  total_count: number
  offset: number
  limit: number
}

// ============================================================================
// Expiration Types
// ============================================================================

/**
 * Expiration status for an entry
 */
export type ExpirationStatus =
  | { type: 'Expired'; expired_at: number }
  | { type: 'ExpiringSoon'; expires_at: number; days_remaining: number }
  | { type: 'NotExpiring' }

/**
 * Configuration for expiration reminders
 */
export interface ExpirationConfig {
  /** Days before expiration to show warning */
  warning_days: number
  /** Enable automatic notifications */
  enable_notifications: boolean
  /** Auto-delete expired entries */
  auto_delete_expired: boolean
  /** Check interval in hours */
  check_interval_hours: number
}

/**
 * Expiration counts for dashboard display
 */
export interface ExpirationCounts {
  /** Number of expired entries */
  expired: number
  /** Number of entries expiring soon */
  expiring_soon: number
}

/**
 * Entry with expiration info
 */
export interface ExpiringSoonEntry {
  entry: Entry
  days_remaining: number
}

/**
 * Notification payload for expiration events
 */
export interface ExpirationNotificationPayload {
  entry_id: number
  entry_name: string
  status: 'expired' | 'expiring_soon'
  message: string
}

// ============================================================================
// Sorting Types
// ============================================================================

/**
 * Sort method for search results
 */
export type SortMethod =
  | 'relevance'
  | 'most_used'
  | 'recently_opened'
  | 'alphabetical'
  | 'custom'

/**
 * Sorting weights for custom sorting
 */
export interface SortingWeights {
  frequency_weight: number
  recency_weight: number
  relevance_weight: number
}

/**
 * Sorting configuration
 */
export interface SortingConfig {
  method: SortMethod
  weights: SortingWeights
  frequency_half_life: number
  debug_mode: boolean
}

/**
 * Score breakdown for debug display
 */
export interface ScoreBreakdown {
  frequency_score: number
  recency_score: number
  relevance_score: number
  total_score: number
}

/**
 * Time window for grouping results
 */
export type TimeWindow = 'hour' | 'day' | 'week' | 'month' | 'older'

/**
 * Sort method display labels
 */
export const SORT_METHOD_LABELS: Record<SortMethod, string> = {
  relevance: 'Relevance',
  most_used: 'Most Used',
  recently_opened: 'Recently Opened',
  alphabetical: 'Alphabetical',
  custom: 'Custom',
}

/**
 * Default sorting weights
 */
export const DEFAULT_SORTING_WEIGHTS: SortingWeights = {
  frequency_weight: 0.3,
  recency_weight: 0.2,
  relevance_weight: 0.5,
}

/**
 * Validate sorting weights sum to 1.0
 */
export function validateSortingWeights(weights: SortingWeights): boolean {
  const sum =
    weights.frequency_weight + weights.recency_weight + weights.relevance_weight
  return Math.abs(sum - 1.0) < 0.01
}

/**
 * Normalize sorting weights to sum to 1.0
 */
export function normalizeSortingWeights(weights: SortingWeights): SortingWeights {
  const sum =
    weights.frequency_weight + weights.recency_weight + weights.relevance_weight
  if (sum === 0) return DEFAULT_SORTING_WEIGHTS
  return {
    frequency_weight: weights.frequency_weight / sum,
    recency_weight: weights.recency_weight / sum,
    relevance_weight: weights.relevance_weight / sum,
  }
}