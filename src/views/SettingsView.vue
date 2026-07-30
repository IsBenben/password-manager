<template>
  <div class="layout">
    <AppSidebar @filter="navigateToList" />

    <main class="main-content">
      <header class="top-bar">
        <h2>{{ i18n.t('settings.title') }}</h2>
      </header>
      <div class="settings-body">
        <section class="setting-section">
          <h3>{{ i18n.t('settings.master_password') }}</h3>
          <form @submit.prevent="handleChangePassword" class="setting-form">
            <div class="field">
              <label>{{ i18n.t('settings.current_password') }}</label>
              <input v-model="changePwd.old" type="password" />
            </div>
            <div class="field">
              <label>{{ i18n.t('settings.new_password') }}</label>
              <input v-model="changePwd.new" type="password" />
            </div>
            <div class="field">
              <label>{{ i18n.t('settings.confirm_password') }}</label>
              <input v-model="changePwd.confirm" type="password" />
            </div>
            <p v-if="pwdError" class="error">{{ pwdError }}</p>
            <button type="submit" class="btn-primary" :disabled="changingPwd">
              {{ changingPwd ? i18n.t('settings.changing') : i18n.t('settings.change_password') }}
            </button>
          </form>
        </section>

        <section class="setting-section">
          <h3>{{ i18n.t('settings.git_sync') }}</h3>
          <form @submit.prevent="handleGitConfig" class="setting-form">
            <div class="field">
              <label>{{ i18n.t('settings.git_remote') }}</label>
              <input v-model="gitRemote" type="text" :placeholder="i18n.t('settings.git_remote_placeholder')" />
            </div>
            <p v-if="gitError" class="error">{{ gitError }}</p>
            <div class="btn-group">
              <button type="submit" class="btn-primary">{{ i18n.t('settings.save_config') }}</button>
              <button type="button" class="btn-secondary" @click="handleGitPush">{{ i18n.t('settings.push_to_git') }}</button>
              <button type="button" class="btn-secondary" @click="handleGitPull">{{ i18n.t('settings.pull_from_git') }}</button>
            </div>
          </form>
        </section>

        <section class="setting-section">
          <h3>{{ i18n.t('settings.appearance') }}</h3>
          <div class="setting-form">
            <div class="field">
              <label>{{ i18n.t('settings.theme') }}</label>
              <div class="radio-group">
                <label class="radio-row">
                  <input v-model="themeMode" type="radio" value="system" @change="updateTheme" />
                  {{ i18n.t('settings.theme_system') }}
                </label>
                <label class="radio-row">
                  <input v-model="themeMode" type="radio" value="light" @change="updateTheme" />
                  {{ i18n.t('settings.theme_light') }}
                </label>
                <label class="radio-row">
                  <input v-model="themeMode" type="radio" value="dark" @change="updateTheme" />
                  {{ i18n.t('settings.theme_dark') }}
                </label>
              </div>
            </div>
            <div class="field">
              <label>{{ i18n.t('settings.accent_color') }}</label>
              <div class="accent-picker">
                <button
                  v-for="c in accentPresets"
                  :key="c"
                  class="accent-swatch"
                  :class="{ active: themeStore.accentColor === c }"
                  :style="{ background: c }"
                  @click="setAccent(c)"
                />
                <label class="accent-custom" :style="{ borderColor: themeStore.accentColor && !accentPresets.includes(themeStore.accentColor) ? themeStore.accentColor : undefined }">
                  <input type="color" :value="themeStore.accentColor || '#0071e3'" @input="setAccent(($event.target as HTMLInputElement).value)" />
                </label>
                <button v-if="themeStore.accentColor" class="accent-reset" @click="setAccent('')">{{ i18n.t('settings.accent_reset') }}</button>
              </div>
            </div>
            <div class="field">
              <label>{{ i18n.t('settings.font_family') }}</label>
              <input v-model="fontFamily" type="text" :placeholder="i18n.t('settings.font_placeholder')" @change="updateFont" />
            </div>
          </div>
        </section>

        <section class="setting-section">
          <h3>{{ i18n.t('settings.session') }}</h3>
          <form @submit.prevent="handleSessionConfig" class="setting-form">
            <div class="field">
              <label>{{ i18n.t('settings.session_timeout') }}</label>
              <input v-model.number="sessionTimeout" type="number" min="1" max="1440" />
            </div>
            <p v-if="sessionError" class="error">{{ sessionError }}</p>
            <button type="submit" class="btn-primary">{{ i18n.t('settings.save_session') }}</button>
          </form>
        </section>

        <section class="setting-section">
          <h3>{{ i18n.t('settings.import_export') }}</h3>
          <form class="setting-form">
            <div class="field">
              <label>{{ i18n.t('settings.export_path') }}</label>
              <input v-model="exportPath" type="text" :placeholder="i18n.t('settings.export_placeholder')" />
            </div>
            <div class="btn-group">
              <button type="button" class="btn-secondary" @click="handleExport('json')">{{ i18n.t('settings.export_json') }}</button>
              <button type="button" class="btn-secondary" @click="handleExport('csv')">{{ i18n.t('settings.export_csv') }}</button>
            </div>
          </form>
          <form class="setting-form" style="margin-top: 1rem;">
            <div class="field">
              <label>{{ i18n.t('settings.import_path') }}</label>
              <input v-model="importPath" type="text" :placeholder="i18n.t('settings.import_placeholder')" />
            </div>
            <div class="btn-group">
              <button type="button" class="btn-secondary" @click="handleImport('json')">{{ i18n.t('settings.import_json') }}</button>
              <button type="button" class="btn-secondary" @click="handleImport('csv')">{{ i18n.t('settings.import_csv') }}</button>
            </div>
          </form>
        </section>

        <section class="setting-section">
          <h3>{{ i18n.t('settings.about') }}</h3>
          <p class="about-text">{{ i18n.t('app.title') }} {{ i18n.t('app.version') }}</p>
          <p class="about-text">{{ i18n.t('settings.data_path') }}</p>
        </section>
      </div>
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/authStore'
import { useConfigStore } from '../stores/configStore'
import { useThemeStore } from '../stores/themeStore'
import { usePasswordStore } from '../stores/passwordStore'
import { useI18nStore } from '../stores/i18nStore'
import AppSidebar from '../components/AppSidebar.vue'
import { showPrompt } from '../stores/dialogStore'
import { useToast } from '../stores/toastStore'
import { invoke } from '@tauri-apps/api/core'

const router = useRouter()
const auth = useAuthStore()
const configStore = useConfigStore()
const passwordStore = usePasswordStore()
const i18n = useI18nStore()
const themeStore = useThemeStore()
const toast = useToast()

const themeMode = ref(themeStore.theme)
function updateTheme() {
  themeStore.set(themeMode.value)
}

const accentPresets = ['#0071e3', '#0a84ff', '#34c759', '#ff9f0a', '#ff3b30', '#bf5af2', '#ff375f', '#64d2ff']
function setAccent(color: string) {
  themeStore.setAccentColor(color)
}

async function updateFont() {
  if (fontFamily.value === configStore.config.font_family) return
  await configStore.updateConfig({
    git_remote: configStore.config.git_remote,
    font_family: fontFamily.value,
    session_timeout_minutes: configStore.config.session_timeout_minutes,
  })
  document.documentElement.style.setProperty('--app-font', fontFamily.value)
  toast.success(i18n.t('settings.success_config'))
}

function navigateToList(cat?: string) {
  if (cat === '__fav__') {
    router.push('/list?fav=1')
  } else if (cat !== undefined) {
    router.push('/list?cat=' + encodeURIComponent(cat))
  } else {
    router.push('/list')
  }
}

const changePwd = ref({ old: '', new: '', confirm: '' })
const pwdError = ref('')
const changingPwd = ref(false)

const gitRemote = ref('')
const fontFamily = ref('')
const gitError = ref('')

const sessionTimeout = ref(30)
const sessionError = ref('')

const exportPath = ref('')
const importPath = ref('')

onMounted(async () => {
  if (!auth.checkSession()) {
    router.push('/')
    return
  }
  gitRemote.value = configStore.config.git_remote
  fontFamily.value = configStore.config.font_family
  sessionTimeout.value = configStore.config.session_timeout_minutes
})

async function handleChangePassword() {
  pwdError.value = ''

  if (changePwd.value.new !== changePwd.value.confirm) {
    pwdError.value = i18n.t('settings.error_match')
    return
  }
  if (changePwd.value.new.length < 12) {
    pwdError.value = i18n.t('settings.error_length')
    return
  }

  changingPwd.value = true
  try {
    await passwordStore.changeMasterPassword(changePwd.value.old, changePwd.value.new)
    auth.currentPassword = changePwd.value.new
    toast.success(i18n.t('settings.success_changed'))
    changePwd.value = { old: '', new: '', confirm: '' }
  } catch (e: any) {
    pwdError.value = typeof e === 'string' ? e : i18n.t('settings.error_change')
  } finally {
    changingPwd.value = false
  }
}

async function handleGitConfig() {
  gitError.value = ''
  if (gitRemote.value === configStore.config.git_remote) return
  try {
    await configStore.updateConfig({
      git_remote: gitRemote.value,
      font_family: configStore.config.font_family,
      session_timeout_minutes: configStore.config.session_timeout_minutes,
    })
    toast.success(i18n.t('settings.success_config'))
  } catch (e: any) {
    gitError.value = typeof e === 'string' ? e : i18n.t('settings.error_config')
  }
}

async function handleGitPush() {
  gitError.value = ''
  try {
    const result: string = await invoke('git_push', { message: i18n.t('settings.git_commit_message') })
    toast.success(result)
  } catch (e: any) {
    gitError.value = typeof e === 'string' ? e : i18n.t('settings.push_failed')
  }
}

async function handleGitPull() {
  gitError.value = ''
  try {
    const result: string = await invoke('git_pull')
    toast.success(result)
  } catch (e: any) {
    gitError.value = typeof e === 'string' ? e : i18n.t('settings.pull_failed')
  }
}

async function handleSessionConfig() {
  sessionError.value = ''
  if (sessionTimeout.value < 1 || sessionTimeout.value > 1440) {
    sessionError.value = i18n.t('settings.error_timeout_range')
    return
  }
  if (sessionTimeout.value === configStore.config.session_timeout_minutes) return
  try {
    await configStore.updateConfig({
      git_remote: configStore.config.git_remote,
      font_family: configStore.config.font_family,
      session_timeout_minutes: sessionTimeout.value,
    })
    auth.setSessionTimeout(sessionTimeout.value)
    toast.success(i18n.t('settings.success_session_timeout'))
  } catch (e: any) {
    sessionError.value = typeof e === 'string' ? e : i18n.t('settings.error_session_save')
  }
}

async function handleExport(fmt: 'json' | 'csv') {
  if (!exportPath.value) {
    toast.error(i18n.t('settings.error_export_path'))
    return
  }
  try {
    const cmd = fmt === 'csv' ? 'export_csv' : 'export_json'
    const args: Record<string, unknown> = { path: exportPath.value }
    if (fmt === 'csv') args.password = auth.currentPassword
    await invoke(cmd, args)
    toast.success(i18n.t('toast.exported', exportPath.value))
  } catch (e: any) {
    toast.error(typeof e === 'string' ? e : i18n.t('settings.error_export'))
  }
}

async function handleImport(fmt: 'json' | 'csv') {
  if (!importPath.value) {
    toast.error(i18n.t('settings.error_import_path'))
    return
  }
  const pwd = await showPrompt({
    title: i18n.t('settings.prompt_import_password'),
    message: '',
    inputType: 'password',
    confirmText: i18n.t('dialog.confirm'),
    cancelText: i18n.t('dialog.cancel'),
  })
  if (!pwd) return
  try {
    const cmd = fmt === 'csv' ? 'import_csv' : 'import_json'
    const result: string = await invoke(cmd, { path: importPath.value, password: pwd })
    const count = result.match(/\d+/)?.[0] || ''
    toast.success(i18n.t('toast.imported', count))
    await passwordStore.fetchEntries()
  } catch (e: any) {
    toast.error(typeof e === 'string' ? e : i18n.t('settings.error_import'))
  }
}
</script>

<style scoped>
.layout { display: flex; height: 100vh; }
.main-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.top-bar { display: flex; align-items: center; gap: 1rem; padding: 1rem 1.5rem; border-bottom: 1px solid var(--border); }
.top-bar h2 { margin: 0; font-size: 1.25rem; }
.settings-body { flex: 1; overflow-y: auto; padding: 1.5rem; min-height: 0; }
.setting-section {
  background: var(--card-bg); border: 1px solid var(--border);
  border-radius: 8px; padding: 1.5rem; margin-bottom: 1rem;
}
.setting-section h3 { margin: 0 0 1rem; font-size: 1rem; }
.setting-form { max-width: 400px; }
.field { margin-bottom: 0.75rem; }
label { display: block; margin-bottom: 0.25rem; font-size: 0.8125rem; font-weight: 500; }
input {
  width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.9375rem; background: var(--bg); color: var(--text); box-sizing: border-box;
}
input:focus { outline: none; border-color: var(--primary); }
.error { color: var(--danger); font-size: 0.875rem; }
.success { color: var(--success); font-size: 0.875rem; }
.btn-primary, .btn-secondary { padding: 0.5rem 1rem; border: none; border-radius: 6px; cursor: pointer; font-size: 0.875rem; margin-top: 0.5rem; }
.btn-primary { background: var(--primary); color: #fff; }
.btn-secondary { background: var(--hover-bg); color: var(--text); margin-left: 0.5rem; }
.btn-group { display: flex; flex-wrap: wrap; gap: 0.5rem; }
.btn-group button { margin: 0; }
.radio-group { display: flex; flex-wrap: wrap; gap: 0.25rem 1rem; margin-top: 0.25rem; }
.radio-row { display: flex; align-items: center; gap: 0.375rem; font-size: 0.875rem; font-weight: 400; cursor: pointer; }
.radio-row input[type="radio"] { width: auto; margin: 0; }
.accent-picker { display: flex; flex-wrap: wrap; align-items: center; gap: 0.5rem; margin-top: 0.25rem; }
.accent-swatch {
  width: 1.5rem; height: 1.5rem; border-radius: 50%; border: 2px solid transparent;
  cursor: pointer; padding: 0; transition: border-color 0.15s;
}
.accent-swatch.active { border-color: var(--primary); }
.accent-swatch:hover { border-color: var(--text-secondary); }
.accent-custom {
  display: flex; align-items: center; justify-content: center;
  width: 1.5rem; height: 1.5rem; border-radius: 50%; border: 2px dashed var(--border);
  cursor: pointer; overflow: hidden;
}
.accent-custom input[type="color"] {
  width: 2rem; height: 2rem; padding: 0; border: none; cursor: pointer;
}
.accent-reset {
  padding: 0.25rem 0.5rem; font-size: 0.75rem; background: none;
  border: 1px solid var(--border); border-radius: 4px; cursor: pointer; color: var(--text-secondary);
}
.accent-reset:hover { border-color: var(--primary); color: var(--primary); }
.about-text { font-size: 0.875rem; color: var(--text-secondary); margin: 0.25rem 0; }
</style>
