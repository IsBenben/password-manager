<template>
  <div id="app-root">
    <div class="lang-switch" @click="i18n.toggleLang()" :title="i18n.t('app.switch_language')">
      {{ i18n.langLabel }}
    </div>
    <router-view />
  </div>
</template>

<script setup lang="ts">
import { useConfigStore } from './stores/configStore'
import { useI18nStore } from './stores/i18nStore'
import { useAuthStore } from './stores/authStore'
import { onMounted } from 'vue'

const configStore = useConfigStore()
const authStore = useAuthStore()
const i18n = useI18nStore()

onMounted(async () => {
  i18n.initLang()
  await configStore.loadConfig()
  document.documentElement.style.setProperty('--app-font', configStore.config.font_family)
  authStore.setSessionTimeout(configStore.config.session_timeout_minutes)
})
</script>

<style scoped>
.lang-switch {
  position: fixed;
  top: 0.75rem;
  right: 0.75rem;
  z-index: 999;
  padding: 0.25rem 0.625rem;
  font-size: 0.75rem;
  background: var(--card-bg);
  border: 1px solid var(--border);
  border-radius: 4px;
  cursor: pointer;
  color: var(--text-secondary);
  user-select: none;
}
.lang-switch:hover {
  border-color: var(--primary);
  color: var(--primary);
}
</style>
