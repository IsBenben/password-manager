<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>{{ i18n.t('app.title') }}</h2>
      </div>
      <nav>
        <router-link to="/list" class="nav-item">
          <span class="nav-icon">&#128273;</span> {{ i18n.t('nav.all_passwords') }}
        </router-link>
        <router-link to="/settings" class="nav-item">
          <span class="nav-icon">&#9881;</span> {{ i18n.t('nav.settings') }}
        </router-link>
      </nav>
    </aside>

    <main class="main-content">
      <header class="top-bar">
        <button class="btn-back" @click="goBack">{{ i18n.t('detail.back') }}</button>
        <div class="top-actions">
          <button class="btn-edit" @click="showEditForm = true">{{ i18n.t('detail.edit') }}</button>
          <button class="btn-delete" @click="handleDelete">{{ i18n.t('detail.delete') }}</button>
        </div>
      </header>

      <div v-if="loading" class="loading">{{ i18n.t('list.loading') }}</div>
      <div v-else-if="!entry" class="empty">{{ i18n.t('detail.not_found') }}</div>
      <div v-else class="detail-content">
        <div class="detail-card">
          <div class="detail-row">
            <label>{{ i18n.t('detail.site_url') }}</label>
            <div class="value">
              {{ entry.site_url }}
              <p v-if="entry.site_url && !entry.site_url.toLowerCase().startsWith('https://')" class="warning-inline">{{ i18n.t('form.warning_http') }}</p>
            </div>
          </div>
          <div class="detail-row">
            <label>{{ i18n.t('detail.username') }}</label>
            <div class="value">{{ entry.username }}</div>
          </div>
          <div class="detail-row">
            <label>{{ i18n.t('detail.autofill_mode') }}</label>
            <div class="value">{{ autofillModeLabel }}</div>
          </div>
          <div class="detail-row">
            <label>{{ i18n.t('detail.password') }}</label>
            <div class="value sensitive">
              <span v-if="showPassword">{{ entry.password }}</span>
              <span v-else>&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;</span>
              <button class="btn-reveal" @click="revealPassword">
                {{ showPassword ? i18n.t('detail.hide') : i18n.t('detail.show') }}
              </button>
            </div>
          </div>
          <div v-if="emailsInfo.length > 0" class="detail-row">
            <label>{{ i18n.t('detail.emails') }}</label>
            <div class="value sensitive">
              <template v-if="showEmails">
                <div v-for="(em, i) in emailsInfo" :key="i" class="email-line">
                  {{ em.email }}
                  <span v-if="em.is_primary" class="badge-primary">{{ i18n.t('detail.primary') }}</span>
                </div>
              </template>
              <span v-else>&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;</span>
              <button class="btn-reveal" @click="showEmails = !showEmails">
                {{ showEmails ? i18n.t('detail.hide') : i18n.t('detail.show') }}
              </button>
            </div>
          </div>
          <div v-if="entry.phone" class="detail-row">
            <label>{{ i18n.t('detail.phone') }}</label>
            <div class="value sensitive">
              <span v-if="showPhone">{{ entry.phone }}</span>
              <span v-else>&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;</span>
              <button class="btn-reveal" @click="showPhone = !showPhone">
                {{ showPhone ? i18n.t('detail.hide') : i18n.t('detail.show') }}
              </button>
            </div>
          </div>
          <div v-if="entry.twofa_secret" class="detail-row">
            <label>{{ i18n.t('detail.totp_secret') }}</label>
            <div class="value sensitive">
              <span v-if="showTotp">{{ entry.twofa_secret }}</span>
              <span v-else>&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;&#8226;</span>
              <button class="btn-reveal" @click="showTotp = !showTotp">
                {{ showTotp ? i18n.t('detail.hide') : i18n.t('detail.show') }}
              </button>
            </div>
          </div>
          <div v-if="entry.twofa_secret" class="detail-row totp-row">
            <label>{{ i18n.t('detail.totp_code') }}</label>
            <div class="totp-area">
              <div class="totp-codes">
                <span class="totp-code">{{ showTotpCode ? currentTotp : '••••••' }}</span>
                <span class="totp-next">{{ i18n.t('detail.totp_next') }}: {{ showTotpCode ? nextTotp : '••••••' }}</span>
              </div>
              <div class="totp-bar">
                <div class="totp-bar-fill" :style="{ width: totpProgress + '%' }"></div>
              </div>
              <span class="totp-remaining">{{ remaining }}{{ i18n.t('detail.seconds') }}</span>
            </div>
            <div class="totp-actions">
              <button class="btn-reveal" @click="showTotpCode = !showTotpCode">
                {{ showTotpCode ? i18n.t('detail.hide') : i18n.t('detail.show') }}
              </button>
            </div>
          </div>
          <div v-if="entry.note" class="detail-row">
            <label>{{ i18n.t('detail.note') }}</label>
            <div class="value">{{ entry.note }}</div>
          </div>
        </div>
      </div>

      <PasswordForm
        v-if="showEditForm && entry"
        :visible="showEditForm"
        :entry-id="entry.id"
        :edit-data="editData"
        @close="showEditForm = false"
        @saved="refreshEntry"
      />
    </main>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { usePasswordStore, parseEmails, type NewEntry, type EmailInfo } from '../stores/passwordStore'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'
import PasswordForm from '../components/PasswordForm.vue'

const route = useRoute()
const router = useRouter()
const store = usePasswordStore()
const auth = useAuthStore()
const i18n = useI18nStore()

const entry = ref<any>(null)
const loading = ref(true)
const showPassword = ref(false)
const showEmails = ref(false)
const showPhone = ref(false)
const showTotp = ref(false)
const showTotpCode = ref(false)
const showEditForm = ref(false)
const currentTotp = ref('')
const nextTotp = ref('')
const remaining = ref(30)
const totpProgress = ref(100)
let totpTimer: ReturnType<typeof setInterval> | null = null

const emailsInfo = computed<EmailInfo[]>(() => parseEmails(entry.value?.emails_raw ?? null))

const autofillModeLabel = computed(() => {
  if (!entry.value) return ''
  const m = entry.value.autofill_mode
  if (m === 'primary_email') return i18n.t('detail.see_primary_email')
  if (m === 'phone') return i18n.t('detail.see_phone')
  if (m === 'none') return i18n.t('detail.autofill_none')
  return i18n.t('detail.autofill_default')
})

const editData = computed<NewEntry | null>(() => {
  if (!entry.value) return null
  return {
    site_url: entry.value.site_url,
    username: entry.value.username,
    password: '',
    emails_raw: entry.value.emails_raw,
    phone: entry.value.phone,
    twofa_secret: entry.value.twofa_secret,
    note: entry.value.note,
    autofill_mode: entry.value.autofill_mode || 'default',
  }
})

async function decryptEntry() {
  if (!auth.checkSession()) {
    router.push('/')
    return
  }
  try {
    entry.value = await store.getEntry(route.params.id as string, auth.currentPassword)
  } catch {
    const pwd = prompt(i18n.t('detail.prompt_password'))
    if (pwd) {
      try {
        await auth.verifyPassword(pwd)
        entry.value = await store.getEntry(route.params.id as string, auth.currentPassword)
      } catch {
        alert(i18n.t('detail.error_incorrect'))
        router.push('/list')
      }
    } else {
      router.push('/list')
    }
  }
}

function goBack() { router.push('/list') }
function revealPassword() { showPassword.value = !showPassword.value }

async function updateTotp() {
  if (!entry.value?.twofa_secret) return
  try {
    const now = Math.floor(Date.now() / 1000)
    remaining.value = 30 - (now % 30)
    totpProgress.value = ((now % 30) / 30) * 100
    currentTotp.value = await store.generateTotp(entry.value.twofa_secret, 0)
    nextTotp.value = await store.generateTotp(entry.value.twofa_secret, 1)
  } catch {
    currentTotp.value = i18n.t('detail.error_generating')
    nextTotp.value = i18n.t('detail.error_generating')
  }
}

function startTotpTimer() {
  updateTotp()
  totpTimer = setInterval(updateTotp, 1000)
}

onMounted(async () => {
  loading.value = true
  await decryptEntry()
  loading.value = false
  if (entry.value?.twofa_secret) {
    startTotpTimer()
  }
})

onUnmounted(() => {
  if (totpTimer) clearInterval(totpTimer)
})

async function handleDelete() {
  if (!confirm(i18n.t('detail.confirm_delete'))) return
  await store.deleteEntry(route.params.id as string)
  router.push('/list')
}

async function refreshEntry() {
  showEditForm.value = false
  await decryptEntry()
}
</script>

<style scoped>
.layout { display: flex; height: 100vh; }
.sidebar {
  width: 240px; background: var(--card-bg);
  border-right: 1px solid var(--border); display: flex; flex-direction: column; flex-shrink: 0;
}
.sidebar-header { padding: 1.25rem; border-bottom: 1px solid var(--border); }
.sidebar-header h2 { margin: 0; font-size: 1.125rem; }
nav { flex: 1; padding: 0.75rem; }
.nav-item {
  display: flex; align-items: center; gap: 0.5rem; padding: 0.625rem 0.75rem;
  border-radius: 6px; color: var(--text); text-decoration: none; font-size: 0.9375rem;
}
.nav-item:hover { background: var(--hover-bg); }
.main-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.top-bar {
  display: flex; align-items: center; justify-content: space-between;
  padding: 1rem 1.5rem; border-bottom: 1px solid var(--border);
}
.btn-back {
  padding: 0.5rem 0.75rem; background: none; border: 1px solid var(--border);
  border-radius: 6px; cursor: pointer; color: var(--text);
}
.top-actions { display: flex; gap: 0.5rem; }
.btn-edit, .btn-delete { padding: 0.5rem 0.75rem; border: none; border-radius: 6px; cursor: pointer; font-size: 0.875rem; }
.btn-edit { background: var(--primary); color: #fff; }
.btn-delete { background: var(--danger); color: #fff; }
.loading, .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); }
.detail-content { flex: 1; overflow-y: auto; padding: 1.5rem; }
.detail-card { background: var(--card-bg); border: 1px solid var(--border); border-radius: 8px; padding: 1.5rem; }
.detail-row { display: flex; align-items: center; gap: 1rem; padding: 0.75rem 0; border-bottom: 1px solid var(--border); }
.detail-row:last-child { border-bottom: none; }
.detail-row label { width: 120px; font-size: 0.8125rem; font-weight: 500; color: var(--text-secondary); flex-shrink: 0; }
.value { flex: 1; font-size: 0.9375rem; }
.sensitive { display: flex; align-items: center; gap: 0.5rem; }
.btn-reveal { padding: 0.25rem 0.5rem; font-size: 0.75rem; background: none; border: 1px solid var(--border); border-radius: 4px; cursor: pointer; color: var(--primary); white-space: nowrap; }
.email-line { font-size: 0.875rem; line-height: 1.6; }
.badge-primary { display: inline-block; font-size: 0.625rem; background: var(--primary); color: #fff; padding: 0.0625rem 0.375rem; border-radius: 3px; margin-left: 0.25rem; vertical-align: middle; }
.totp-row { display: flex; align-items: flex-start; gap: 1rem; padding: 0.75rem 0; border-bottom: 1px solid var(--border); }
.totp-area { flex: 1; display: flex; align-items: center; gap: 0.5rem; }
.totp-codes { display: flex; flex-direction: column; min-width: 140px; }
.totp-code { font-family: Consolas, monospace; font-size: 1.25rem; letter-spacing: 0.25rem; }
.totp-next { font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.125rem; }
.totp-bar { width: 48px; height: 4px; background: var(--border); border-radius: 2px; overflow: hidden; flex-shrink: 0; }
.totp-bar-fill { height: 100%; background: var(--primary); border-radius: 2px; transition: width 0.3s linear; }
.totp-remaining { font-size: 0.8125rem; font-weight: 600; color: var(--primary); min-width: 24px; text-align: right; }
.warning-inline { color: #cc8800; font-size: 0.75rem; margin: 0.25rem 0 0; line-height: 1.3; }
.totp-actions { flex-shrink: 0; }
</style>
