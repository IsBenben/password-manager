import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

const ACCENT_KEY = 'pm_accent'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemeMode>('system')
  const accentColor = ref('')
  let mediaQuery: MediaQueryList | null = null

  const effectiveTheme = computed(() => {
    if (theme.value !== 'system') return theme.value
    return mediaQuery?.matches ? 'dark' : 'light'
  })

  function systemChanged() {
    if (theme.value === 'system') {
      apply()
    }
  }

  function init() {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    mediaQuery.addEventListener('change', systemChanged)
    const saved = localStorage.getItem('pm_theme') as ThemeMode | null
    if (saved === 'light' || saved === 'dark' || saved === 'system') {
      theme.value = saved
    }
    const savedAccent = localStorage.getItem(ACCENT_KEY)
    if (savedAccent) {
      accentColor.value = savedAccent
    }
    apply()
  }

  function toggle() {
    if (theme.value === 'system') {
      theme.value = effectiveTheme.value
    } else {
      theme.value = theme.value === 'dark' ? 'light' : 'dark'
    }
    localStorage.setItem('pm_theme', theme.value)
    apply()
  }

  function set(t: ThemeMode) {
    theme.value = t
    localStorage.setItem('pm_theme', t)
    apply()
  }

  function setAccentColor(color: string) {
    accentColor.value = color
    if (color) {
      localStorage.setItem(ACCENT_KEY, color)
    } else {
      localStorage.removeItem(ACCENT_KEY)
    }
    apply()
  }

  function apply() {
    document.documentElement.classList.toggle('dark', effectiveTheme.value === 'dark')
    if (accentColor.value) {
      document.documentElement.style.setProperty('--primary', accentColor.value)
    } else {
      document.documentElement.style.removeProperty('--primary')
    }
  }

  function cleanup() {
    mediaQuery?.removeEventListener('change', systemChanged)
  }

  return { theme, effectiveTheme, accentColor, init, toggle, set, setAccentColor, apply, cleanup }
})
