<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <h2>{{ isEdit ? i18n.t('form.edit_title') : i18n.t('form.add_title') }}</h2>
      <form @submit.prevent="handleSave">
        <div class="field">
          <label>{{ i18n.t('form.site_url') }}</label>
          <input v-model="form.site_url" type="text" :placeholder="i18n.t('form.site_url_placeholder')" />
          <p v-if="httpsWarning" class="warning">{{ httpsWarning }}</p>
        </div>
        <div class="field">
          <label>{{ i18n.t('form.username') }}</label>
          <input v-model="form.username" type="text" :placeholder="i18n.t('form.username_placeholder')" />
        </div>
        <div class="field">
          <label>{{ i18n.t('form.password') }}</label>
          <div class="pwd-row">
            <input v-model="form.password" type="password" :placeholder="i18n.t('form.password_placeholder')" />
            <button type="button" class="btn-gen" @click="showGenerator = true">{{ i18n.t('form.gen_password') }}</button>
          </div>
          <p v-if="weakPwdWarning" class="warning">{{ weakPwdWarning }}</p>
        </div>
        <div class="field">
          <label>{{ i18n.t('form.email') }}</label>
          <div v-for="(em, i) in emailList" :key="i" class="email-row">
            <input v-model="em.email" type="email" :placeholder="i18n.t('form.email_placeholder')" />
            <button type="button" class="btn-sm" @click="togglePrimary(i)" :title="i18n.t('form.set_primary')">
              {{ em.is_primary ? '*' : '' }}
            </button>
            <button type="button" class="btn-sm btn-remove" @click="removeEmail(i)" :title="i18n.t('form.remove')">x</button>
          </div>
          <button type="button" class="btn-add-email" @click="addEmail">{{ i18n.t('form.add_email') }}</button>
        </div>
        <div class="field">
          <label>{{ i18n.t('form.phone') }}</label>
          <input v-model="form.phone" type="tel" :placeholder="i18n.t('form.phone_placeholder')" />
        </div>
        <div class="field">
          <label>{{ i18n.t('detail.autofill_mode') }}</label>
          <div class="radio-group">
            <label class="radio-row">
              <input v-model="autofillMode" type="radio" value="default" />
              {{ i18n.t('detail.autofill_default') }}
            </label>
            <label class="radio-row">
              <input v-model="autofillMode" type="radio" value="primary_email" />
              {{ i18n.t('detail.autofill_primary_email') }}
            </label>
            <label class="radio-row">
              <input v-model="autofillMode" type="radio" value="phone" />
              {{ i18n.t('detail.autofill_phone') }}
            </label>
            <label class="radio-row">
              <input v-model="autofillMode" type="radio" value="none" />
              {{ i18n.t('detail.autofill_none') }}
            </label>
          </div>
        </div>
        <div class="field">
          <label>{{ i18n.t('form.totp_secret') }}</label>
          <input v-model="form.twofa_secret" type="text" :placeholder="i18n.t('form.totp_secret_placeholder')" />
        </div>
        <div class="field">
          <label>{{ i18n.t('form.note') }}</label>
          <textarea v-model="form.note" rows="3" :placeholder="i18n.t('form.note_placeholder')"></textarea>
        </div>
        <p v-if="error" class="error">{{ error }}</p>
        <div class="modal-actions">
          <button type="button" class="btn-cancel" @click="$emit('close')">{{ i18n.t('form.cancel') }}</button>
          <button type="submit" class="btn-save" :disabled="saving">
            {{ saving ? i18n.t('form.saving') : i18n.t('form.save') }}
          </button>
        </div>
      </form>

      <PasswordGenerator
        v-if="showGenerator"
        @close="showGenerator = false"
        @select="onGenSelect"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch, computed } from 'vue'
import { usePasswordStore, type NewEntry, type EmailInfo, serializeEmails, parseEmails } from '../stores/passwordStore'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'
import PasswordGenerator from './PasswordGenerator.vue'

const props = defineProps<{
  visible: boolean
  entryId?: string
  editData?: NewEntry | null
}>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const store = usePasswordStore()
const auth = useAuthStore()
const i18n = useI18nStore()

const saving = ref(false)
const error = ref('')
const showGenerator = ref(false)

const form = reactive<NewEntry>({
  site_url: '',
  username: '',
  password: '',
  emails_raw: null,
  phone: null,
  twofa_secret: null,
  note: '',
  autofill_mode: 'default',
})

const emailList = reactive<EmailInfo[]>([])

const autofillMode = ref('default')

const isEdit = !!props.entryId

const WEAK_PATTERNS = [
  /^123456$/i, /^12345678$/i, /^123456789$/i, /^password$/i,
  /^qwerty$/i, /^abc123$/i, /^admin123$/i, /^passw0rd$/i,
  /^letmein$/i, /^welcome$/i, /^monkey$/i, /^dragon$/i,
  /^master$/i, /^123456Aa$/i, /^Aa123456$/i,
]

function isWeakPassword(pwd: string): boolean {
  if (!pwd || pwd.length < 6) return false
  for (const pattern of WEAK_PATTERNS) {
    if (pattern.test(pwd)) return true
  }
  const digits = (pwd.match(/\d/g) || []).length
  if (pwd.length >= 6 && digits === pwd.length) return true
  return false
}

const httpsWarning = computed(() => {
  const url = form.site_url.trim()
  if (!url) return ''
  if (!/^https?:\/\//i.test(url)) return ''
  if (!url.toLowerCase().startsWith('https://')) {
    return i18n.t('form.warning_http')
  }
  return ''
})

const weakPwdWarning = computed(() => {
  if (isWeakPassword(form.password)) {
    return i18n.t('form.warning_weak_password')
  }
  return ''
})

function syncEmails() {
  form.emails_raw = emailList.length > 0 ? serializeEmails(emailList) : null
}

function resolveAutofillMode(data: NewEntry): string {
  const m = data.autofill_mode
  if (m && m !== '') return m
  return data.username ? 'default' : 'none'
}

function addEmail() {
  emailList.push({ email: '', is_primary: emailList.length === 0 })
}

function removeEmail(i: number) {
  const wasPrimary = emailList[i].is_primary
  emailList.splice(i, 1)
  if (wasPrimary && emailList.length > 0) emailList[0].is_primary = true
  syncEmails()
}

function togglePrimary(i: number) {
  emailList.forEach((e, idx) => (e.is_primary = idx === i))
  syncEmails()
}

function onGenSelect(pwd: string) {
  form.password = pwd
  showGenerator.value = false
}

watch(
  () => props.editData,
  (data) => {
    if (data) {
      form.site_url = data.site_url
      form.username = data.username
      form.password = ''
      form.phone = data.phone
      form.twofa_secret = data.twofa_secret
      form.note = data.note
      const parsed = parseEmails(data.emails_raw)
      emailList.length = 0
      parsed.forEach((e) => emailList.push({ ...e }))
      autofillMode.value = resolveAutofillMode(data)
      syncEmails()
    }
  },
  { immediate: true }
)

watch(emailList, syncEmails, { deep: true })

watch(autofillMode, (val) => { form.autofill_mode = val })

async function handleSave() {
  error.value = ''
  if (!auth.checkSession()) {
    error.value = i18n.t('form.session_expired')
    return
  }

  saving.value = true
  try {
    const entry: NewEntry = {
      site_url: form.site_url,
      username: form.username,
      password: form.password,
      emails_raw: emailList.length > 0 ? serializeEmails(emailList) : null,
      phone: form.phone || null,
      twofa_secret: form.twofa_secret || null,
      note: form.note,
      autofill_mode: autofillMode.value,
    }

    if (isEdit && props.entryId) {
      await store.editEntry(props.entryId, entry, auth.currentPassword)
    } else {
      await store.addEntry(entry, auth.currentPassword)
    }
    emit('saved')
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : i18n.t('form.error_save')
  } finally {
    saving.value = false
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center; z-index: 100;
}
.modal {
  background: var(--card-bg); border-radius: 12px; padding: 2rem;
  width: 100%; max-width: 500px; max-height: 90vh; overflow-y: auto;
}
h2 { margin: 0 0 1.25rem; }
.field { margin-bottom: 0.875rem; }
label { display: block; margin-bottom: 0.25rem; font-size: 0.8125rem; font-weight: 500; }
input, textarea {
  width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.9375rem; background: var(--bg); color: var(--text);
  box-sizing: border-box; font-family: inherit;
}
input:focus, textarea:focus { outline: none; border-color: var(--primary); }
textarea { resize: vertical; }
.pwd-row { display: flex; gap: 0.375rem; }
.pwd-row input { flex: 1; }
.btn-gen {
  padding: 0.5rem 0.625rem; background: var(--hover-bg); border: 1px solid var(--border);
  border-radius: 6px; cursor: pointer; font-size: 0.8125rem; white-space: nowrap;
}
.email-row { display: flex; gap: 0.25rem; margin-bottom: 0.25rem; }
.email-row input { flex: 1; }
.btn-sm {
  width: 28px; height: 28px; padding: 0; background: var(--hover-bg);
  border: 1px solid var(--border); border-radius: 4px; cursor: pointer;
  font-size: 0.8125rem; flex-shrink: 0;
}
.btn-remove { color: var(--danger); }
.btn-add-email {
  margin-top: 0.25rem; padding: 0.25rem 0.5rem; background: none;
  border: 1px dashed var(--border); border-radius: 4px; cursor: pointer;
  font-size: 0.8125rem; color: var(--primary);
}
.error { color: var(--danger); font-size: 0.875rem; margin: 0.5rem 0; }
.warning { color: #cc8800; font-size: 0.75rem; margin: 0.25rem 0 0; line-height: 1.3; }
.radio-group { display: flex; flex-wrap: wrap; gap: 0.25rem 1rem; margin-top: 0.25rem; }
.radio-row { display: flex; align-items: center; gap: 0.375rem; font-size: 0.875rem; font-weight: 400; cursor: pointer; }
.radio-row input[type="radio"] { width: auto; margin: 0; }
.modal-actions { display: flex; justify-content: flex-end; gap: 0.5rem; margin-top: 1rem; }
.btn-cancel, .btn-save { padding: 0.5rem 1rem; border: none; border-radius: 6px; cursor: pointer; font-size: 0.9375rem; }
.btn-cancel { background: var(--hover-bg); color: var(--text); }
.btn-save { background: var(--primary); color: #fff; }
.btn-save:disabled { opacity: 0.6; cursor: not-allowed; }
</style>
