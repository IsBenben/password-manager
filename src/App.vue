<template>
  <div id="app-root">
    <div class="corner-actions">
      <button class="corner-btn" @click="themeStore.toggle()" :title="i18n.t(themeStore.theme === 'dark' ? 'app.theme_light' : 'app.theme_dark')">
        {{ themeStore.theme === 'dark' ? '\u2600' : '\u263E' }}
      </button>
      <button class="corner-btn" @click="i18n.toggleLang()" :title="i18n.t('app.switch_language')">
        {{ i18n.langLabel }}
      </button>
    </div>
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from './stores/configStore'
import { useI18nStore } from './stores/i18nStore'
import { useAuthStore } from './stores/authStore'
import { useThemeStore } from './stores/themeStore'
import { onMounted } from 'vue'

const configStore = useConfigStore()
const authStore = useAuthStore()
const i18n = useI18nStore()
const themeStore = useThemeStore()

onMounted(async () => {
  themeStore.init()
  i18n.initLang()
  await configStore.loadConfig()
  document.documentElement.style.setProperty('--app-font', configStore.config.font_family)
  authStore.setSessionTimeout(configStore.config.session_timeout_minutes)
})
</script>

<style scoped>
.corner-actions {
  position: fixed;
  top: 0.75rem;
  right: 0.75rem;
  z-index: 999;
  display: flex;
  gap: 0.375rem;
}
.corner-btn {
  padding: 0.25rem 0.5rem;
  font-size: 0.75rem;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-secondary);
  user-select: none;
  line-height: 1.4;
}
.corner-btn:hover {
  border-color: var(--primary);
  color: var(--primary);
}
</style>
