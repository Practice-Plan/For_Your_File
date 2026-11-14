/**
 * Appearance Settings (v0.0.5)
 *
 * Accent color, font scale and motion preferences. Changes apply immediately
 * and are persisted in localStorage (see src/utils/appearance.ts).
 */

import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  ACCENT_ORDER,
  applyAppearance,
  loadAppearance,
  saveAppearance,
  type AccentName,
  type AppearanceSettings as AppearanceConfig,
  type FontScale,
} from '../utils/appearance'

const FONT_SCALES: FontScale[] = ['small', 'normal', 'large']

export function AppearanceSettings() {
  const { t } = useTranslation()
  const [settings, setSettings] = useState<AppearanceConfig>(() => loadAppearance())

  const update = (next: Partial<AppearanceConfig>) => {
    const merged = { ...settings, ...next }
    setSettings(merged)
    saveAppearance(merged)
    applyAppearance(merged)
  }

  const reset = () => {
    update({ accent: 'blue', fontScale: 'normal', reduceMotion: false })
  }

  return (
    <div className="p-6 space-y-6">
      <div>
        <h3 className="text-sm font-semibold text-gray-900 dark:text-gray-100">
          {t('appearance.title')}
        </h3>
        <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
          {t('appearance.description')}
        </p>
      </div>

      {/* Accent color */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('appearance.accent')}
        </label>
        <div className="flex flex-wrap gap-2">
          {ACCENT_ORDER.map((accent) => (
            <button
              key={accent}
              onClick={() => update({ accent })}
              title={t(`appearance.colors.${accent}`)}
              aria-label={t(`appearance.colors.${accent}`)}
              aria-pressed={settings.accent === accent}
              className={`flex items-center gap-2 px-3 py-1.5 rounded-md border text-xs transition-colors ${
                settings.accent === accent
                  ? 'border-primary-500 bg-primary-50 dark:bg-primary-950/40 text-primary-700 dark:text-primary-300'
                  : 'border-gray-200 dark:border-gray-700 text-gray-600 dark:text-gray-400 hover:border-gray-300 dark:hover:border-gray-600'
              }`}
            >
              <span
                className="w-4 h-4 rounded-full border border-black/10"
                style={{ backgroundColor: `rgb(${accentColor(accent)})` }}
              />
              {t(`appearance.colors.${accent}`)}
            </button>
          ))}
        </div>
      </div>

      {/* Font scale */}
      <div>
        <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
          {t('appearance.fontSize')}
        </label>
        <div className="inline-flex rounded-md border border-gray-200 dark:border-gray-700 overflow-hidden">
          {FONT_SCALES.map((scale) => (
            <button
              key={scale}
              onClick={() => update({ fontScale: scale })}
              aria-pressed={settings.fontScale === scale}
              className={`px-3 py-1.5 text-xs transition-colors ${
                settings.fontScale === scale
                  ? 'bg-primary-500 text-white'
                  : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700'
              }`}
            >
              {t(`appearance.fontScales.${scale}`)}
            </button>
          ))}
        </div>
      </div>

      {/* Reduce motion */}
      <div className="flex items-start justify-between gap-4">
        <div>
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            {t('appearance.reduceMotion')}
          </label>
          <p className="mt-1 text-xs text-gray-500 dark:text-gray-400">
            {t('appearance.reduceMotionDescription')}
          </p>
        </div>
        <button
          role="switch"
          aria-checked={settings.reduceMotion}
          onClick={() => update({ reduceMotion: !settings.reduceMotion })}
          className={`relative flex-shrink-0 w-10 h-5 rounded-full transition-colors ${
            settings.reduceMotion ? 'bg-primary-500' : 'bg-gray-300 dark:bg-gray-600'
          }`}
        >
          <span
            className={`absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white transition-transform ${
              settings.reduceMotion ? 'translate-x-5' : 'translate-x-0'
            }`}
          />
        </button>
      </div>

      <div className="pt-2 border-t border-gray-200 dark:border-gray-700">
        <button
          onClick={reset}
          className="text-xs text-primary-600 dark:text-primary-400 hover:underline"
        >
          {t('appearance.reset')}
        </button>
      </div>
    </div>
  )
}

function accentColor(accent: AccentName): string {
  // 500 shade of each palette, used for the swatch dot.
  const shades: Record<AccentName, string> = {
    blue: '59 130 246',
    violet: '139 92 246',
    emerald: '16 185 129',
    amber: '245 158 11',
    rose: '244 63 94',
  }
  return shades[accent]
}
