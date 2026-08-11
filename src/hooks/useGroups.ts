/**
 * Custom hook for managing groups
 */
import { useState, useCallback, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/core'
import type { Group, GroupWithCount, Entry } from '../types'

interface UseGroupsReturn {
  groups: GroupWithCount[]
  isLoading: boolean
  error: string | null
  createGroup: (name: string, color: string) => Promise<Group | null>
  updateGroup: (id: number, name?: string, color?: string) => Promise<Group | null>
  deleteGroup: (id: number) => Promise<boolean>
  getGroupEntries: (groupId: number) => Promise<Entry[]>
  addEntryToGroup: (entryId: number, groupId: number) => Promise<boolean>
  removeEntryFromGroup: (entryId: number, groupId: number) => Promise<boolean>
  getEntryGroups: (entryId: number) => Promise<Group[]>
  refreshGroups: () => Promise<void>
  exportGroup: (groupId: number) => Promise<string | null>
  importGroup: (exportData: string) => Promise<Group | null>
}

export function useGroups(): UseGroupsReturn {
  const [groups, setGroups] = useState<GroupWithCount[]>([])
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)

  // Load groups on mount
  useEffect(() => {
    refreshGroups()
  }, [])

  // Refresh groups from backend
  const refreshGroups = useCallback(async () => {
    setIsLoading(true)
    setError(null)

    try {
      const result = await invoke<GroupWithCount[]>('list_groups')
      setGroups(result)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
    } finally {
      setIsLoading(false)
    }
  }, [])

  // Create a new group
  const createGroup = useCallback(async (name: string, color: string): Promise<Group | null> => {
    try {
      const group = await invoke<Group>('create_group', { name, color })
      await refreshGroups()
      return group
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    }
  }, [refreshGroups])

  // Update an existing group
  const updateGroup = useCallback(async (
    id: number,
    name?: string,
    color?: string
  ): Promise<Group | null> => {
    try {
      const group = await invoke<Group>('update_group', { id, name, color })
      await refreshGroups()
      return group
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    }
  }, [refreshGroups])

  // Delete a group
  const deleteGroup = useCallback(async (id: number): Promise<boolean> => {
    try {
      const success = await invoke<boolean>('delete_group', { id })
      if (success) {
        await refreshGroups()
      }
      return success
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return false
    }
  }, [refreshGroups])

  // Get entries in a group
  const getGroupEntries = useCallback(async (groupId: number): Promise<Entry[]> => {
    try {
      return await invoke<Entry[]>('get_group_entries', { groupId })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return []
    }
  }, [])

  // Add entry to group
  const addEntryToGroup = useCallback(async (entryId: number, groupId: number): Promise<boolean> => {
    try {
      return await invoke<boolean>('add_entry_to_group', { entryId, groupId })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return false
    }
  }, [])

  // Remove entry from group
  const removeEntryFromGroup = useCallback(async (entryId: number, groupId: number): Promise<boolean> => {
    try {
      return await invoke<boolean>('remove_entry_from_group', { entryId, groupId })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return false
    }
  }, [])

  // Get groups for an entry
  const getEntryGroups = useCallback(async (entryId: number): Promise<Group[]> => {
    try {
      return await invoke<Group[]>('get_entry_groups', { entryId })
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return []
    }
  }, [])

  // Export group as JSON
  const exportGroup = useCallback(async (groupId: number): Promise<string | null> => {
    try {
      const data = await invoke<{ group: Group; entry_ids: number[]; exported_at: number }>('export_group', { groupId })
      return JSON.stringify(data, null, 2)
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    }
  }, [])

  // Import group from JSON
  const importGroup = useCallback(async (exportData: string): Promise<Group | null> => {
    try {
      const { group, entry_ids } = JSON.parse(exportData)
      const result = await invoke<Group>('import_group', {
        name: group.name,
        color: group.color,
        entryIds: entry_ids
      })
      await refreshGroups()
      return result
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err))
      return null
    }
  }, [refreshGroups])

  return {
    groups,
    isLoading,
    error,
    createGroup,
    updateGroup,
    deleteGroup,
    getGroupEntries,
    addEntryToGroup,
    removeEntryFromGroup,
    getEntryGroups,
    refreshGroups,
    exportGroup,
    importGroup,
  }
}