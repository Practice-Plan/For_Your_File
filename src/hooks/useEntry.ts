import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Entry } from '../types'

interface UseEntryReturn {
  entry: Entry | null
  loading: boolean
  error: string | null
  updateEntry: (updates: Partial<Entry>) => Promise<void>
  refreshEntry: () => Promise<void>
  deleteEntry: () => Promise<void>
}

export function useEntry(entryId: number | null): UseEntryReturn {
  const [entry, setEntry] = useState<Entry | null>(null)
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchEntry = useCallback(async () => {
    if (entryId === null) {
      setEntry(null)
      return
    }

    setLoading(true)
    setError(null)

    try {
      // get_entry returns Option<Entry>: null when not found, Entry object when found
      const result = await invoke<Entry | null>('get_entry', { id: entryId })
      if (result) {
        setEntry(result)
      } else {
        setEntry(null)
        setError('Entry not found')
      }
    } catch (err) {
      const msg = typeof err === 'string' ? err : (err instanceof Error ? err.message : 'Failed to fetch entry')
      setError(msg)
      console.error('Failed to fetch entry:', err)
    } finally {
      setLoading(false)
    }
  }, [entryId])

  const updateEntry = useCallback(
    async (updates: Partial<Entry>) => {
      if (!entry?.id) return

      setLoading(true)
      setError(null)

      try {
        const updatedEntry = { ...entry, ...updates }
        // The Rust update_entry command requires both `id` and `entry` parameters.
        // It returns the updated entry with correct server-side timestamps (seconds).
        const result = await invoke<Entry>('update_entry', { id: entry.id, entry: updatedEntry })
        setEntry(result)
      } catch (err) {
        const msg = typeof err === 'string' ? err : (err instanceof Error ? err.message : 'Failed to update entry')
        setError(msg)
        console.error('Failed to update entry:', err)
        throw err
      } finally {
        setLoading(false)
      }
    },
    [entry]
  )

  const deleteEntry = useCallback(async () => {
    if (!entry?.id) return

    setLoading(true)
    setError(null)

    try {
      await invoke('delete_entry', { id: entry.id })
      setEntry(null)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to delete entry')
      console.error('Failed to delete entry:', err)
      throw err
    } finally {
      setLoading(false)
    }
  }, [entry])

  useEffect(() => {
    fetchEntry()
  }, [fetchEntry])

  return {
    entry,
    loading,
    error,
    updateEntry,
    refreshEntry: fetchEntry,
    deleteEntry,
  }
}

interface UseEntriesReturn {
  entries: Entry[]
  loading: boolean
  error: string | null
  refreshEntries: () => Promise<void>
  searchEntries: (query: string) => Promise<void>
}

export function useEntries(): UseEntriesReturn {
  const [entries, setEntries] = useState<Entry[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const fetchEntries = useCallback(async () => {
    setLoading(true)
    setError(null)

    try {
      const result = await invoke<Entry[]>('get_all_entries')
      setEntries(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch entries')
      console.error('Failed to fetch entries:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  const searchEntries = useCallback(async (query: string) => {
    if (!query.trim()) {
      fetchEntries()
      return
    }

    setLoading(true)
    setError(null)

    try {
      // search_entries returns PaginatedEntries { results, total_count, offset, limit }
      const response = await invoke<{ results: Entry[]; total_count: number; offset: number; limit: number }>('search_entries', { query })
      setEntries(response.results)
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to search entries')
      console.error('Failed to search entries:', err)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchEntries()
  }, [])

  return {
    entries,
    loading,
    error,
    refreshEntries: fetchEntries,
    searchEntries,
  }
}

interface UseCreateEntryReturn {
  createEntry: (entry: Omit<Entry, 'id' | 'created_at' | 'updated_at'>) => Promise<Entry>
  loading: boolean
  error: string | null
}

export function useCreateEntry(): UseCreateEntryReturn {
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const createEntry = useCallback(
    async (entryData: Omit<Entry, 'id' | 'created_at' | 'updated_at'>): Promise<Entry> => {
      setLoading(true)
      setError(null)

      try {
        const now = Date.now()
        const newEntry: Entry = {
          ...entryData,
          id: null,
          created_at: now,
          updated_at: now,
        }

        const result = await invoke<Entry>('create_entry', { entry: newEntry })
        return result
      } catch (err) {
        const errorMsg = err instanceof Error ? err.message : 'Failed to create entry'
        setError(errorMsg)
        console.error('Failed to create entry:', err)
        throw err
      } finally {
        setLoading(false)
      }
    },
    []
  )

  return { createEntry, loading, error }
}