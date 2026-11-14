/**
 * Component Packages (v0.0.5)
 *
 * Lists built-in and imported component packages and lets the user enable,
 * disable, import, export, delete and read the authoring guide.
 */

import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { invoke } from '@tauri-apps/api/core'
import { open, save } from '@tauri-apps/plugin-dialog'
import { reportError, toErrorMessage } from '../utils/reportError'
import i18n from '../i18n'

interface ComponentSpec {
  id: string
  kind: string
  title: string
  title_key?: string | null
  config: unknown
}

interface ComponentPackage {
  schema: number
  id: string
  name: string
  name_key?: string | null
  version: string
  description: string
  description_key?: string | null
  author: string
  kind: string
  components: ComponentSpec[]
  builtin: boolean
  enabled: boolean
}

export function ComponentPackages() {
  const { t } = useTranslation()
  const [packages, setPackages] = useState<ComponentPackage[]>([])
  const [loading, setLoading] = useState(true)
  const [status, setStatus] = useState<string | null>(null)
  const [guide, setGuide] = useState<string | null>(null)

  // 有文案键时优先查表，否则使用包内自带的名称。
  const label = (key?: string | null, fallback = '') => (key ? t(key) : fallback)

  const load = useCallback(async () => {
    setLoading(true)
    try {
      const list = await invoke<ComponentPackage[]>('list_component_packages')
      setPackages(list)
    } catch (error) {
      void reportError(toErrorMessage(error, t))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    void load()
  }, [load])

  const toggle = async (pkg: ComponentPackage) => {
    try {
      await invoke('set_component_package_enabled', { id: pkg.id, enabled: !pkg.enabled })
      setPackages((prev) =>
        prev.map((item) => (item.id === pkg.id ? { ...item, enabled: !item.enabled } : item)),
      )
    } catch (error) {
      void reportError(toErrorMessage(error, t))
    }
  }

  const importPackage = async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: t('components.dialogFilter'), extensions: ['json'] }],
      })
      if (typeof selected !== 'string') return
      const imported = await invoke<ComponentPackage>('import_component_package', {
        path: selected,
      })
      setStatus(t('components.importSuccess', { name: label(imported.name_key, imported.name) }))
      await load()
    } catch (error) {
      const message = toErrorMessage(error, t)
      setStatus(message)
      void reportError(message)
    }
  }

  const exportPackage = async (pkg: ComponentPackage) => {
    try {
      const target = await save({
        defaultPath: `${pkg.id}.json`,
        filters: [{ name: t('components.dialogFilter'), extensions: ['json'] }],
      })
      if (!target) return
      await invoke('export_component_package', { id: pkg.id, path: target })
      setStatus(t('components.exportSuccess', { name: label(pkg.name_key, pkg.name) }))
    } catch (error) {
      const message = toErrorMessage(error, t)
      setStatus(message)
      void reportError(message)
    }
  }

  const removePackage = async (pkg: ComponentPackage) => {
    if (!window.confirm(t('components.deleteConfirm', { name: label(pkg.name_key, pkg.name) }))) return
    try {
      await invoke('delete_component_package', { id: pkg.id })
      await load()
    } catch (error) {
      const message = toErrorMessage(error, t)
      setStatus(message)
      void reportError(message)
    }
  }

  const openGuide = async () => {
    if (guide !== null) {
      setGuide(null)
      return
    }
    try {
      setGuide(
        await invoke<string>('get_component_creation_guide', { locale: i18n.language }),
      )
    } catch (error) {
      void reportError(toErrorMessage(error, t))
    }
  }

  return (
    <div className="p-6 space-y-4">
      <div className="flex items-start justify-between gap-4">
        <div>
          <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
            {t('components.title')}
          </h3>
          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
            {t('components.description')}
          </p>
        </div>
        <div className="flex gap-2 flex-shrink-0">
          <button
            onClick={importPackage}
            className="px-3 py-1.5 text-xs rounded-md bg-primary-500 text-white hover:bg-primary-600 transition-colors"
          >
            {t('components.import')}
          </button>
          <button
            onClick={openGuide}
            className="px-3 py-1.5 text-xs rounded-md border border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-300 hover:border-gray-300 dark:hover:border-gray-600 transition-colors"
          >
            {t('components.guide')}
          </button>
        </div>
      </div>

      {status && (
        <p className="text-xs text-gray-600 dark:text-gray-400 break-words">{status}</p>
      )}

      {loading ? (
        <p className="text-xs text-gray-500 dark:text-gray-400">{t('common.loading')}</p>
      ) : packages.length === 0 ? (
        <p className="text-xs text-gray-500 dark:text-gray-400">{t('components.empty')}</p>
      ) : (
        <ul className="space-y-2">
          {packages.map((pkg) => (
            <li
              key={pkg.id}
              className="rounded-md border border-gray-200 dark:border-gray-700 bg-gray-50/60 dark:bg-gray-900/30 p-3"
            >
              <div className="flex items-start justify-between gap-3">
                <div className="min-w-0">
                  <div className="flex items-center gap-2 flex-wrap">
                    <span className="text-sm font-medium text-gray-900 dark:text-gray-100 truncate">
                      {label(pkg.name_key, pkg.name)}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-gray-200 dark:bg-gray-700 text-gray-600 dark:text-gray-300">
                      v{pkg.version}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded bg-primary-100 dark:bg-primary-950/50 text-primary-700 dark:text-primary-300">
                      {pkg.kind || 'bundle'}
                    </span>
                    {pkg.builtin && (
                      <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-100 dark:bg-emerald-950/50 text-emerald-700 dark:text-emerald-300">
                        {t('components.builtin')}
                      </span>
                    )}
                  </div>
                  <p className="mt-1 text-xs text-gray-500 dark:text-gray-400 break-words">
                    {label(pkg.description_key, pkg.description) || pkg.id}
                  </p>
                  <p className="mt-1 text-[11px] text-gray-400 dark:text-gray-500">
                    {pkg.id} ·{' '}
                    {t('components.componentCount', { count: pkg.components.length })}
                  </p>
                </div>
                <div className="flex items-center gap-2 flex-shrink-0">
                  <button
                    onClick={() => toggle(pkg)}
                    className={`px-2 py-1 text-[11px] rounded border transition-colors ${
                      pkg.enabled
                        ? 'border-primary-500 text-primary-600 dark:text-primary-400'
                        : 'border-gray-300 dark:border-gray-600 text-gray-500 dark:text-gray-400'
                    }`}
                  >
                    {pkg.enabled ? t('components.enabled') : t('components.disabled')}
                  </button>
                  <button
                    onClick={() => exportPackage(pkg)}
                    className="px-2 py-1 text-[11px] rounded border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-300 hover:border-gray-400"
                  >
                    {t('components.export')}
                  </button>
                  {!pkg.builtin && (
                    <button
                      onClick={() => removePackage(pkg)}
                      className="px-2 py-1 text-[11px] rounded border border-red-200 dark:border-red-900 text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/30"
                    >
                      {t('components.delete')}
                    </button>
                  )}
                </div>
              </div>
            </li>
          ))}
        </ul>
      )}

      {guide !== null && (
        <div className="rounded-md border border-gray-200 dark:border-gray-700">
          <div className="flex items-center justify-between px-3 py-2 border-b border-gray-200 dark:border-gray-700">
            <span className="text-xs font-medium text-gray-700 dark:text-gray-300">
              {t('components.guideTitle')}
            </span>
            <button
              onClick={() => setGuide(null)}
              className="text-xs text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              {t('common.close')}
            </button>
          </div>
          <pre className="max-h-64 overflow-auto p-3 text-[11px] leading-relaxed text-gray-600 dark:text-gray-300 whitespace-pre-wrap select-text">
            {guide}
          </pre>
        </div>
      )}
    </div>
  )
}
