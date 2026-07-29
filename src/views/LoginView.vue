<template>
  <div class="login-container">
    <div class="login-card">
      <h1>{{ isSetup ? i18n.t('login.setup_title') : i18n.t('login.unlock_title') }}</h1>
      <p class="subtitle">{{ isSetup ? i18n.t('login.setup_subtitle') : i18n.t('login.unlock_subtitle') }}</p>

      <form @submit.prevent="handleSubmit">
        <div class="field">
          <label for="password">{{ i18n.t('login.password_label') }}</label>
          <input
            id="password"
            v-model="password"
            type="password"
            :placeholder="i18n.t('login.password_placeholder')"
            autofocus
            @input="checkNonAscii"
          />
          <p v-if="nonAsciiWarning" class="warning">{{ nonAsciiWarning }}</p>
        </div>

        <div v-if="isSetup" class="field">
          <label for="confirm">{{ i18n.t('login.confirm_label') }}</label>
          <input
            id="confirm"
            v-model="confirm"
            type="password"
            :placeholder="i18n.t('login.confirm_placeholder')"
          />
        </div>

        <p v-if="error" class="error">{{ error }}</p>

        <button type="submit" :disabled="!password || loading">
          {{ loading ? i18n.t('login.processing') : isSetup ? i18n.t('login.create') : i18n.t('login.unlock') }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'

const router = useRouter()
const auth = useAuthStore()
const i18n = useI18nStore()

const password = ref('')
const confirm = ref('')
const isSetup = ref(false)
const loading = ref(false)
const error = ref('')
const nonAsciiWarning = ref('')

onMounted(async () => {
  const initialized = await auth.checkInit()
  isSetup.value = !initialized
})

function checkNonAscii() {
  if (/[^\x00-\x7F]/.test(password.value)) {
    nonAsciiWarning.value = i18n.t('login.non_ascii_warning')
  } else {
    nonAsciiWarning.value = ''
  }
}

function validate(pwd: string): boolean {
  if (pwd.length < 12) return false
  if (!/[A-Z]/.test(pwd)) return false
  if (!/[a-z]/.test(pwd)) return false
  if (!/[0-9]/.test(pwd)) return false
  if (!/[^A-Za-z0-9]/.test(pwd)) return false
  return true
}

async function handleSubmit() {
  error.value = ''
  nonAsciiWarning.value = ''
  loading.value = true

  try {
    if (isSetup.value) {
      if (password.value !== confirm.value) {
        error.value = i18n.t('login.error_match')
        loading.value = false
        return
      }
      if (!validate(password.value)) {
        error.value = i18n.t('login.error_strength')
        loading.value = false
        return
      }
      await auth.initPassword(password.value)
      auth.currentPassword = password.value
      auth.isVerified = true
    } else {
      await auth.verifyPassword(password.value)
    }
    router.push('/list')
  } catch {
    error.value = i18n.t('login.error_incorrect')
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.login-container {
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 100vh;
  padding: 2rem;
}
.login-card {
  background: var(--card-bg);
  border-radius: 12px;
  padding: 2.5rem;
  width: 100%;
  max-width: 420px;
  box-shadow: var(--shadow);
}
h1 { margin: 0 0 0.5rem; font-size: 1.5rem; }
.subtitle { color: var(--text-secondary); margin: 0 0 1.5rem; }
.field { margin-bottom: 1rem; }
label { display: block; margin-bottom: 0.25rem; font-size: 0.875rem; font-weight: 500; }
input {
  width: 100%; padding: 0.625rem 0.75rem;
  border: 1px solid var(--border); border-radius: 6px;
  font-size: 0.9375rem; background: var(--bg); color: var(--text); box-sizing: border-box;
}
input:focus { outline: none; border-color: var(--primary); }
button {
  width: 100%; padding: 0.75rem; background: var(--primary);
  color: #fff; border: none; border-radius: 6px; font-size: 1rem; cursor: pointer; margin-top: 0.5rem;
}
button:disabled { opacity: 0.6; cursor: not-allowed; }
.error { color: var(--danger); font-size: 0.875rem; margin: 0.5rem 0; }
.warning { color: #cc8800; font-size: 0.75rem; margin: 0.25rem 0 0; line-height: 1.3; }
</style>
