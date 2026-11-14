/**
 * Structured backend errors.
 *
 * The Rust side returns `{ code, params }` instead of sentence text so the UI
 * can render the message in the current language. Codes map to
 * `errors.<code>` in the translation resources.
 */

import type { TFunction } from 'i18next'

export interface AppErrorShape {
  code: string
  params?: Record<string, string | number>
}

export function isAppError(value: unknown): value is AppErrorShape {
  return !!value && typeof value === 'object' && typeof (value as AppErrorShape).code === 'string'
}

/** Translate an error code; unknown codes fall back to a generic message. */
export function translateAppError(error: unknown, t: TFunction): string {
  if (!isAppError(error)) return ''
  const key = `errors.${error.code}`
  const text = t(key, error.params ?? {})
  return text === key ? `${t('errors.unexpected')} (${error.code})` : text
}
