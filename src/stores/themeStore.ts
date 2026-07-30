import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeMode = 'system' | 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<ThemeMode>('system')
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

  function apply() {
    document.documentElement.classList.toggle('dark', effectiveTheme.value === 'dark')
  }

  function cleanup() {
    mediaQuery?.removeEventListener('change', systemChanged)
  }

  return { theme, effectiveTheme, init, toggle, set, apply, cleanup }
})
