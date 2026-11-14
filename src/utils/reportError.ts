/**
 * Error reporting (v0.0.5)
 *
 * All user-facing errors must be shown by PPC. The Rust side exposes
 * `ppc_report_error`, which forwards the message through PPC's `WINDOW ERROR`
 * command; there is intentionally no in-app fallback dialog.
 */

import { invoke } from '@tauri-apps/api/core'
import type { TFunction } from 'i18next'
import { isAppError, translateAppError } from './appError'

let lastMessage = ''
let lastAt = 0
let reporting = false

/**
 * Report an error to PPC. Duplicate messages within two seconds are dropped so
 * a failing render loop cannot flood PPC.
 */
export async function reportError(message: string): Promise<void> {
  const text = (message || 'Unknown error').slice(0, 2000)
  const now = Date.now()
  if (text === lastMessage && now - lastAt < 2000) return
  if (reporting) return

  lastMessage = text
  lastAt = now
  reporting = true

  try {
    const shown = await invoke<boolean>('ppc_report_error', { message: text })
    if (!shown) {
      console.error('PPC unavailable, error not displayed:', text)
    }
  } catch (error) {
    console.error('Failed to report error through PPC:', error, text)
  } finally {
    reporting = false
  }
}

/** Convert an unknown catch value into a readable message. */
export function toErrorMessage(error: unknown, t?: TFunction): string {
  if (isAppError(error)) {
    return t ? translateAppError(error, t) : error.code
  }
  if (typeof error === 'string') return error
  if (error instanceof Error) return error.message
  try {
    return JSON.stringify(error)
  } catch {
    return String(error)
  }
}

/**
 * Install global handlers so uncaught errors and unhandled promise rejections
 * are routed to PPC instead of dying silently in the webview console.
 */
export function installGlobalErrorReporting(): void {
  if (typeof window === 'undefined') return

  window.addEventListener('error', (event) => {
    void reportError(event.message || 'Unexpected UI error')
  })

  window.addEventListener('unhandledrejection', (event) => {
    void reportError(toErrorMessage(event.reason))
  })
}
