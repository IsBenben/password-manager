import { defineStore } from 'pinia'
import { ref } from 'vue'

export type Theme = 'light' | 'dark'

export const useThemeStore = defineStore('theme', () => {
  const theme = ref<Theme>('light')

  function init() {
    const saved = localStorage.getItem('pm_theme') as Theme | null
    if (saved === 'dark') {
      theme.value = 'dark'
    }
    apply()
  }

  function toggle() {
    theme.value = theme.value === 'light' ? 'dark' : 'light'
    localStorage.setItem('pm_theme', theme.value)
    apply()
  }

  function set(t: Theme) {
    theme.value = t
    localStorage.setItem('pm_theme', t)
    apply()
  }

  function apply() {
    document.documentElement.classList.toggle('dark', theme.value === 'dark')
  }

  return { theme, init, toggle, set, apply }
})
