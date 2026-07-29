import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useAuthStore = defineStore('auth', () => {
  const isInitialized = ref(false)
  const isVerified = ref(false)
  const sessionExpiry = ref(0)
  const currentPassword = ref('')
  const sessionTimeoutMinutes = ref(30)

  function getSessionDuration(): number {
    return sessionTimeoutMinutes.value * 60 * 1000
  }

  function setSessionTimeout(minutes: number) {
    sessionTimeoutMinutes.value = minutes
  }

  async function checkInit() {
    try {
      isInitialized.value = await invoke('is_initialized')
      return isInitialized.value
    } catch {
      isInitialized.value = false
      return false
    }
  }

  async function initPassword(password: string) {
    await invoke('init_password', { password })
    isInitialized.value = true
  }

  async function verifyPassword(password: string) {
    await invoke('verify_password', { password })
    currentPassword.value = password
    isVerified.value = true
    sessionExpiry.value = Date.now() + getSessionDuration()
  }

  function checkSession() {
    if (!isVerified.value) return false
    if (Date.now() > sessionExpiry.value) {
      isVerified.value = false
      currentPassword.value = ''
      return false
    }
    return true
  }

  function clearSession() {
    isVerified.value = false
    currentPassword.value = ''
    sessionExpiry.value = 0
  }

  return {
    isInitialized,
    isVerified,
    currentPassword,
    sessionTimeoutMinutes,
    checkInit,
    initPassword,
    verifyPassword,
    checkSession,
    clearSession,
    setSessionTimeout,
  }
})
