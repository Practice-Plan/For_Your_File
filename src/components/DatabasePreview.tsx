import { useEffect, useState, useRef, useCallback, startTransition } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useTranslation } from 'react-i18next'

type TableName = 'entries' | 'groups' | 'entry_groups'

type DatabasePreviewBatch = {
  table: TableName
  columns: string[]
  rows: unknown[][]
  total_count: number
  offset: number
  limit: number
  has_more: boolean
}

type ProgressEvent = {
  table: TableName
  loaded: number
  total: number
}

const tables: TableName[] = ['entries', 'groups', 'entry_groups']
const batchSize = 200
const maxRenderedRows = 1000
const maxRowsPerPage = 5000 // safety ceiling to prevent OOM

export function DatabasePreview() {
  const { t } = useTranslation()
  const [activeTable, setActiveTable] = useState<TableName>('entries')
  const [columns, setColumns] = useState<string[]>([])
  const [rows, setRows] = useState<unknown[][]>([])
  const [total, setTotal] = useState(0)
  const [loaded, setLoaded] = useState(0)
  const [offset, setOffset] = useState(0)
  const [hasMore, setHasMore] = useState(false)
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [loadErrorCount, setLoadErrorCount] = useState(0)

  // Guard: prevent overlapping requests and stale responses from racing.
  const activeRequestId = useRef(0)
  const mountedRef = useRef(true)

  useEffect(() => {
    mountedRef.current = true
    return () => {
      mountedRef.current = false
    }
  }, [])

  useEffect(() => {
    const unlisten = listen<ProgressEvent>('database-preview-progress', (event) => {
      if (!mountedRef.current) return
      if (event.payload.table === activeTable) {
        setLoaded(event.payload.loaded)
        setTotal(event.payload.total)
      }
    })
    return () => {
      unlisten.then((stop) => stop())
    }
  }, [activeTable])

  const loadTable = useCallback(async (table: TableName, nextOffset = 0) => {
    if (isLoading) return // prevent overlapping
    setIsLoading(true)
    setError(null)
    setRows([])
    setColumns([])
    setLoaded(0)
    setTotal(0)

    const requestId = ++activeRequestId.current

    try {
      const batch = await invoke<DatabasePreviewBatch>('get_database_preview_batch', {
        table,
        offset: nextOffset,
        limit: batchSize,
      })

      // Stale response guard: ignore results from superseded requests.
      if (!mountedRef.current || requestId !== activeRequestId.current) return

      setColumns(batch.columns)
      setTotal(batch.total_count)
      setOffset(batch.offset)
      setLoaded(batch.offset + batch.rows.length)
      setHasMore(batch.has_more)

      // Memory safety: cap the number of rows we keep in state.
      const cappedRows = batch.rows.length > maxRowsPerPage
        ? batch.rows.slice(0, maxRowsPerPage)
        : batch.rows
      startTransition(() => setRows(cappedRows))
    } catch (err) {
      if (!mountedRef.current || requestId !== activeRequestId.current) return
      const errMsg = String(err)
      setError(errMsg)
      setLoadErrorCount((c) => c + 1)
      // Clear rows on error so the UI doesn't show stale data
      setRows([])
    } finally {
      if (mountedRef.current && requestId === activeRequestId.current) {
        setIsLoading(false)
      }
    }
  }, [isLoading])

  useEffect(() => {
    void loadTable(activeTable)
  }, [activeTable, loadTable])

  const closeWindow = () => {
    void getCurrentWindow().close()
  }

  // Retry: reset error count and reload
  const handleRetry = () => {
    setLoadErrorCount(0)
    void loadTable(activeTable, offset)
  }

  const progress = total > 0 ? Math.min(100, Math.round((loaded / total) * 100)) : isLoading ? 0 : 100
  const visibleRows = rows.slice(0, maxRenderedRows)
  const isRecoverable = loadErrorCount < 3 // suggest retry up to 3 times

  return (
    <div className="h-screen flex flex-col bg-white dark:bg-dark-bg text-gray-900 dark:text-gray-100">
      <header className="h-12 shrink-0 flex items-center justify-between px-4 border-b border-gray-200 dark:border-dark-border bg-gray-50 dark:bg-dark-surface">
        <div>
          <h1 className="text-sm font-semibold">{t('databasePreview.title')}</h1>
          <p className="text-xs text-gray-500 dark:text-gray-400">
            {isLoading ? t('databasePreview.loading') : t('databasePreview.ready')}
          </p>
        </div>
        <button onClick={closeWindow} className="px-3 py-1.5 text-sm rounded hover:bg-gray-200 dark:hover:bg-gray-700">
          {t('common.close')}
        </button>
      </header>

      <div className="px-4 pt-4 shrink-0">
        <div className="flex gap-2 border-b border-gray-200 dark:border-dark-border">
          {tables.map((table) => (
            <button
              key={table}
              onClick={() => setActiveTable(table)}
              className={`px-3 py-2 text-sm border-b-2 ${activeTable === table ? 'border-primary-500 text-primary-600' : 'border-transparent text-gray-500'}`}
            >
              {t(`databasePreview.${table}`)}
            </button>
          ))}
        </div>
        <div className="mt-3 h-2 overflow-hidden rounded bg-gray-200 dark:bg-gray-700">
          <div className="h-full bg-primary-500 transition-[width] duration-150" style={{ width: `${progress}%` }} />
        </div>
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          {t('databasePreview.loadingProgress', { loaded, total })} ({progress}%)
        </p>
      </div>

      {error ? (
        <div className="m-4 p-4 rounded border border-red-200 bg-red-50 text-sm text-red-800 dark:border-red-800 dark:bg-red-900/20 dark:text-red-200">
          <p className="font-medium">{t('databasePreview.error')}: {error}</p>
          {isRecoverable ? (
            <button onClick={handleRetry} className="mt-3 px-3 py-1.5 rounded bg-red-600 text-white hover:bg-red-700 transition-colors">
              {t('common.retry', 'Retry')}
            </button>
          ) : (
            <p className="mt-2 text-xs text-red-600 dark:text-red-400">
              {t('databasePreview.tooManyRetries', 'Too many retries. Close and reopen the window to try again.')}
            </p>
          )}
        </div>
      ) : (
        <div className="flex-1 min-h-0 overflow-auto p-4">
          {visibleRows.length === 0 && !isLoading ? (
            <p className="py-12 text-center text-sm text-gray-500">{t('databasePreview.empty')}</p>
          ) : (
            <table className="w-full text-left text-xs">
              <thead className="sticky top-0 bg-gray-100 dark:bg-dark-surface">
                <tr>{columns.map((column) => <th key={column} className="px-3 py-2 font-semibold whitespace-nowrap">{column}</th>)}</tr>
              </thead>
              <tbody>
                {visibleRows.map((row, rowIndex) => (
                  <tr key={rowIndex} className="border-b border-gray-100 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-gray-800">
                    {row.map((value, columnIndex) => (
                      <td key={columnIndex} className="max-w-xs truncate px-3 py-2" title={String(value ?? '')}>
                        {value === null ? <span className="text-gray-400">NULL</span> : String(value)}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>
          )}
          {rows.length > maxRenderedRows && (
            <p className="py-3 text-center text-xs text-gray-500">
              {t('databasePreview.renderLimit', { count: maxRenderedRows, total: rows.length })}
            </p>
          )}
          <div className="flex justify-between items-center mt-3 text-xs">
            <button
              disabled={offset === 0 || isLoading}
              onClick={() => void loadTable(activeTable, Math.max(0, offset - batchSize))}
              className="px-3 py-1.5 rounded border border-gray-200 disabled:opacity-40 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            >
              {t('databasePreview.previous')}
            </button>
            <span className="text-gray-500 dark:text-gray-400">{offset + 1}-{Math.min(offset + rows.length, total)} / {total}</span>
            <button
              disabled={!hasMore || isLoading}
              onClick={() => void loadTable(activeTable, offset + rows.length)}
              className="px-3 py-1.5 rounded border border-gray-200 disabled:opacity-40 dark:border-dark-border hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors"
            >
              {t('databasePreview.next')}
            </button>
          </div>
        </div>
      )}
    </div>
  )
}
