import { useEffect, useCallback } from 'react'
import { listen } from '@tauri-apps/api/event'

/**
 * Protocol action types
 */
export type ProtocolAction = 'add' | 'open' | 'search' | 'settings'

/**
 * Protocol request from deep link or CLI
 */
export interface ProtocolRequest {
  action: ProtocolAction
  path?: string
  id?: string
  query?: string
}

/**
 * Protocol handler callbacks
 */
export interface ProtocolHandlers {
  onAdd?: (path: string) => void | Promise<void>
  onOpen?: (id: string) => void | Promise<void>
  onSearch?: (query: string) => void | Promise<void>
  onSettings?: () => void | Promise<void>
}

/**
 * Hook to listen for protocol events from Tauri backend
 *
 * Handles deep link activations and CLI arguments via the protocol-request event.
 *
 * @example
 * ```tsx
 * useProtocol({
 *   onAdd: (path) => console.log('Add:', path),
 *   onOpen: (id) => console.log('Open:', id),
 *   onSearch: (query) => setSearchQuery(query),
 *   onSettings: () => setShowSettings(true),
 * })
 * ```
 */
export function useProtocol(handlers: ProtocolHandlers) {
  const handleRequest = useCallback(
    async (request: ProtocolRequest) => {
      console.log('[Protocol] Received request:', request)

      try {
        switch (request.action) {
          case 'add':
            if (request.path && handlers.onAdd) {
              await handlers.onAdd(request.path)
            }
            break

          case 'open':
            if (request.id && handlers.onOpen) {
              await handlers.onOpen(request.id)
            }
            break

          case 'search':
            if (request.query && handlers.onSearch) {
              await handlers.onSearch(request.query)
            }
            break

          case 'settings':
            if (handlers.onSettings) {
              await handlers.onSettings()
            }
            break

          default:
            console.warn('[Protocol] Unknown action:', request.action)
        }
      } catch (error) {
        console.error('[Protocol] Error handling request:', error)
      }
    },
    [handlers]
  )

  useEffect(() => {
    let unlisten: (() => void) | null = null

    // Listen for protocol events from Tauri backend
    listen<ProtocolRequest>('protocol-request', (event) => {
      handleRequest(event.payload)
    }).then((fn) => {
      unlisten = fn
    })

    return () => {
      if (unlisten) {
        unlisten()
      }
    }
  }, [handleRequest])

  return {
    handleRequest,
  }
}

/**
 * Utility to generate filemgmt:// URLs
 */
export const ProtocolUrl = {
  add: (path: string): string => {
    const encoded = encodeURIComponent(path)
    return `filemgmt://add?path=${encoded}`
  },

  open: (id: string | number): string => {
    return `filemgmt://open?id=${id}`
  },

  search: (query: string): string => {
    const encoded = encodeURIComponent(query)
    return `filemgmt://search?q=${encoded}`
  },

  settings: (): string => {
    return 'filemgmt://settings'
  },
}