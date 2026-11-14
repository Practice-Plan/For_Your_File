/**
 * Appearance settings (v0.0.5)
 *
 * Personalization is stored in localStorage and applied by writing CSS
 * variables consumed by the Tailwind `primary` palette. Keeping the palette in
 * CSS variables means no rebuild is needed when the accent changes.
 */

export type AccentName = 'blue' | 'violet' | 'emerald' | 'amber' | 'rose'
export type FontScale = 'small' | 'normal' | 'large'

export interface AppearanceSettings {
  accent: AccentName
  fontScale: FontScale
  reduceMotion: boolean
}

export const APPEARANCE_STORAGE_KEY = 'app-appearance'

export const DEFAULT_APPEARANCE: AppearanceSettings = {
  accent: 'blue',
  fontScale: 'normal',
  reduceMotion: false,
}

/** Accent shades as "R G B" triplets (Tailwind-compatible 50–950 scale). */
export const ACCENT_PALETTES: Record<AccentName, string[]> = {
  blue: [
    '239 246 255', '219 234 254', '191 219 254', '147 197 253', '96 165 250',
    '59 130 246', '37 99 235', '29 78 216', '30 64 175', '30 58 138', '23 37 84',
  ],
  violet: [
    '245 243 255', '237 233 254', '221 214 254', '196 181 253', '167 139 250',
    '139 92 246', '124 58 237', '109 40 217', '91 33 182', '76 29 149', '46 16 101',
  ],
  emerald: [
    '236 253 245', '209 250 229', '167 243 208', '110 231 183', '52 211 153',
    '16 185 129', '5 150 105', '4 120 87', '6 95 70', '6 78 59', '2 44 34',
  ],
  amber: [
    '255 251 235', '254 243 199', '253 230 138', '252 211 77', '251 191 36',
    '245 158 11', '217 119 6', '180 83 9', '146 64 14', '120 53 15', '69 26 3',
  ],
  rose: [
    '255 241 242', '255 228 230', '254 205 211', '253 164 175', '251 113 133',
    '244 63 94', '225 29 72', '190 18 60', '159 18 57', '136 19 55', '76 5 25',
  ],
}

export const ACCENT_ORDER: AccentName[] = ['blue', 'violet', 'emerald', 'amber', 'rose']

export const FONT_SIZES: Record<FontScale, string> = {
  small: '13px',
  normal: '14px',
  large: '16px',
}

export function loadAppearance(): AppearanceSettings {
  try {
    const raw = localStorage.getItem(APPEARANCE_STORAGE_KEY)
    if (!raw) return DEFAULT_APPEARANCE
    const parsed = JSON.parse(raw) as Partial<AppearanceSettings>
    return normalizeAppearance(parsed)
  } catch {
    return DEFAULT_APPEARANCE
  }
}

export function normalizeAppearance(input: Partial<AppearanceSettings>): AppearanceSettings {
  const accent = ACCENT_ORDER.includes(input.accent as AccentName)
    ? (input.accent as AccentName)
    : DEFAULT_APPEARANCE.accent
  const fontScale =
    input.fontScale === 'small' || input.fontScale === 'large' || input.fontScale === 'normal'
      ? input.fontScale
      : DEFAULT_APPEARANCE.fontScale
  return {
    accent,
    fontScale,
    reduceMotion: input.reduceMotion === true,
  }
}

export function saveAppearance(settings: AppearanceSettings): void {
  try {
    localStorage.setItem(APPEARANCE_STORAGE_KEY, JSON.stringify(settings))
  } catch {
    // Storage may be unavailable; the in-memory settings still apply.
  }
}

export function applyAppearance(settings: AppearanceSettings): void {
  if (typeof document === 'undefined') return
  const root = document.documentElement
  const shades = ACCENT_PALETTES[settings.accent] ?? ACCENT_PALETTES.blue
  const steps = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950]

  steps.forEach((step, index) => {
    root.style.setProperty(`--primary-${step}`, shades[index])
  })

  root.style.setProperty('font-size', FONT_SIZES[settings.fontScale] ?? FONT_SIZES.normal)
  root.classList.toggle('reduce-motion', settings.reduceMotion)
}
