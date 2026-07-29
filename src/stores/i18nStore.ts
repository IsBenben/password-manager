import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { en } from '../i18n/en'
import { zh } from '../i18n/zh'

export type Lang = 'en' | 'zh'
type DeepNested = { [key: string]: string | DeepNested }

const messages: Record<Lang, DeepNested> = { en, zh }

function getNested(obj: DeepNested, path: string): string {
  const keys = path.split('.')
  let current: any = obj
  for (const key of keys) {
    if (current && typeof current === 'object' && key in current) {
      current = current[key]
    } else {
      return path
    }
  }
  return typeof current === 'string' ? current : path
}

export const useI18nStore = defineStore('i18n', () => {
  const lang = ref<Lang>('en')

  const langLabel = computed(() => (lang.value === 'en' ? 'English' : '中文'))

  function setLang(l: Lang) {
    lang.value = l
    localStorage.setItem('pm_lang', l)
  }

  function toggleLang() {
    setLang(lang.value === 'en' ? 'zh' : 'en')
  }

  function initLang() {
    const saved = localStorage.getItem('pm_lang') as Lang | null
    if (saved && (saved === 'en' || saved === 'zh')) {
      lang.value = saved
    }
  }

  function t(path: string): string {
    return getNested(messages[lang.value], path)
  }

  return { lang, langLabel, setLang, toggleLang, initLang, t }
})
