import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'
import LanguageDetector from 'i18next-browser-languagedetector'

// Import translations
import enTranslation from './locales/en/translation.json'
import zhTranslation from './locales/zh/translation.json'
import frTranslation from './locales/fr/translation.json'
import ruTranslation from './locales/ru/translation.json'
import arTranslation from './locales/ar/translation.json'

// Configure i18next
i18n
  // Detect user language
  .use(LanguageDetector)
  // Pass the i18n instance to react-i18next
  .use(initReactI18next)
  // Initialize i18next
  .init({
    resources: {
      en: {
        translation: enTranslation,
      },
      zh: {
        translation: zhTranslation,
      },
      fr: {
        translation: frTranslation,
      },
      ru: {
        translation: ruTranslation,
      },
      ar: {
        translation: arTranslation,
      },
    },
    fallbackLng: 'en', // Default language
    debug: false, // Set to true in development
    
    // Language detection configuration
    detection: {
      // Order of language detection methods
      order: ['querystring', 'localStorage', 'navigator', 'htmlTag'],
      
      // Keys for storing language preference
      lookupQuerystring: 'lng',
      lookupLocalStorage: 'i18nextLng',
      
      // Cache the detected language
      caches: ['localStorage'],
      
      // Exclude cache from localStorage
      excludeCacheFor: ['cimode'],
      
      // HTML attribute for language
      htmlTag: document.documentElement,
      
      // Support for language codes like 'en-US' -> 'en'
      convertDetectedLanguage: (lng: string) => {
        // Map common language codes
        const languageMap: Record<string, string> = {
          'zh-CN': 'zh',
          'zh-TW': 'zh',
          'zh-HK': 'zh',
          'zh-Hans': 'zh',
          'zh-Hant': 'zh',
          'en-US': 'en',
          'en-GB': 'en',
          'en-AU': 'en',
          'fr-FR': 'fr',
          'fr-CA': 'fr',
          'ru-RU': 'ru',
          'ar-SA': 'ar',
          'ar-EG': 'ar',
        }
        
        // Return mapped language or first part of code
        return languageMap[lng] || lng.split('-')[0] || 'en'
      },
    },
    
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    
    // React configuration
    react: {
      useSuspense: false, // Disable suspense for SSR compatibility
    },
  })

export default i18n

// Export supported languages for UI
export const SUPPORTED_LANGUAGES = [
  { code: 'en', name: 'English', nativeName: 'English' },
  { code: 'zh', name: 'Chinese', nativeName: '中文' },
  { code: 'fr', name: 'French', nativeName: 'Français' },
  { code: 'ru', name: 'Russian', nativeName: 'Русский' },
  { code: 'ar', name: 'Arabic', nativeName: 'العربية' },
] as const

export type LanguageCode = typeof SUPPORTED_LANGUAGES[number]['code']