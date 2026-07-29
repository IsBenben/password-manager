import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface Config {
  git_remote: string
  font_family: string
  session_timeout_minutes: number
}

export const useConfigStore = defineStore('config', () => {
  const config = ref<Config>({
    git_remote: '',
    font_family: 'system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif',
    session_timeout_minutes: 30,
  })

  async function loadConfig() {
    try {
      config.value = await invoke('get_config')
    } catch {
      // use defaults
    }
  }

  async function updateConfig(newConfig: Config) {
    await invoke('update_config', { config: newConfig })
    config.value = newConfig
  }

  return { config, loadConfig, updateConfig }
})
