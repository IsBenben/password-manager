<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="modal">
      <h2>{{ i18n.t('generator.title') }}</h2>

      <div class="gen-result">
        <input :value="generatedPwd" readonly class="gen-input" @click="selectAll" />
        <button class="btn-copy" @click="copyPwd">{{ copied ? i18n.t('generator.copied') : i18n.t('generator.copy') }}</button>
      </div>

      <div class="gen-tabs">
        <button :class="{ active: mode === 'random' }" @click="mode = 'random'" ref="randomTabRef">{{ i18n.t('generator.random') }}</button>
        <button :class="{ active: mode === 'phrase' }" @click="mode = 'phrase'">{{ i18n.t('generator.phrase') }}</button>
      </div>

      <div v-if="mode === 'random'" class="gen-controls">
        <div class="field">
          <label>{{ i18n.t('generator.length') }}: {{ length }}</label>
          <input v-model.number="length" type="range" min="4" max="64" />
        </div>

        <label class="checkbox-row">
          <input v-model="useUpper" type="checkbox" />
          {{ i18n.t('generator.include_upper') }}
        </label>
        <label class="checkbox-row">
          <input v-model="useLower" type="checkbox" />
          {{ i18n.t('generator.include_lower') }}
        </label>
        <label class="checkbox-row">
          <input v-model="useDigits" type="checkbox" />
          {{ i18n.t('generator.include_digits') }}
        </label>
        <label class="checkbox-row">
          <input v-model="useSymbols" type="checkbox" />
          {{ i18n.t('generator.include_symbols') }}
        </label>
        <label class="checkbox-row">
          <input v-model="excludeConfusing" type="checkbox" />
          {{ i18n.t('generator.exclude_confusing') }}
        </label>
      </div>

      <div v-if="mode === 'phrase'" class="gen-controls">
        <div class="field">
          <label>{{ i18n.t('generator.word_count') }}: {{ wordCount }}</label>
          <input v-model.number="wordCount" type="range" min="2" max="10" />
        </div>
        <div class="field">
          <label>{{ i18n.t('generator.separator') }}</label>
          <select v-model="separator" class="gen-select">
            <option value="-">- ({{ i18n.t('generator.sep_hyphen') }})</option>
            <option value=" "> ({{ i18n.t('generator.sep_space') }})</option>
            <option value="_">_ ({{ i18n.t('generator.sep_underscore') }})</option>
            <option value="">{{ i18n.t('generator.sep_none') }}</option>
          </select>
        </div>
        <label class="checkbox-row">
          <input v-model="capitalize" type="checkbox" />
          {{ i18n.t('generator.capitalize') }}
        </label>
        <label class="checkbox-row">
          <input v-model="appendNumber" type="checkbox" />
          {{ i18n.t('generator.append_number') }}
        </label>
      </div>

      <div class="modal-actions">
        <button type="button" class="btn-cancel" @click="$emit('close')">{{ i18n.t('form.cancel') }}</button>
        <button type="button" class="btn-primary" @click="doGenerate">{{ i18n.t('generator.generate') }}</button>
        <button type="button" class="btn-save" @click="usePassword">{{ i18n.t('form.save') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick } from 'vue'
import { usePasswordStore } from '../stores/passwordStore'
import { useI18nStore } from '../stores/i18nStore'

const emit = defineEmits<{
  close: []
  select: [password: string]
}>()

const store = usePasswordStore()
const i18n = useI18nStore()

const length = ref(16)
const useUpper = ref(true)
const useLower = ref(true)
const useDigits = ref(true)
const useSymbols = ref(true)
const excludeConfusing = ref(false)
const generatedPwd = ref('')
const copied = ref(false)
const mode = ref<'random' | 'phrase'>('random')
const wordCount = ref(4)
const separator = ref('-')
const capitalize = ref(false)
const appendNumber = ref(false)
const randomTabRef = ref<HTMLButtonElement | null>(null)

onMounted(async () => {
  await nextTick()
  requestAnimationFrame(() => {
    setTimeout(() => {
      randomTabRef.value?.focus()
    }, 100)
  })
})

function selectAll(e: Event) {
  (e.target as HTMLInputElement).select()
}

async function doGenerate() {
  try {
    if (mode.value === 'phrase') {
      generatedPwd.value = await store.generatePassphrase(wordCount.value, separator.value, capitalize.value, appendNumber.value)
    } else {
      generatedPwd.value = await store.generatePassword(
        length.value, useUpper.value, useLower.value,
        useDigits.value, useSymbols.value, excludeConfusing.value,
      )
    }
  } catch {
    generatedPwd.value = i18n.t('generator.error_generating')
  }
}

async function copyPwd() {
  if (!generatedPwd.value) return
  try {
    await navigator.clipboard.writeText(generatedPwd.value)
    copied.value = true
    setTimeout(() => (copied.value = false), 2000)
  } catch {
    // fallback
  }
}

function usePassword() {
  if (generatedPwd.value) {
    emit('select', generatedPwd.value)
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.4);
  display: flex; align-items: center; justify-content: center; z-index: 200;
}
.modal {
  background: var(--card-bg); border-radius: 12px; padding: 2rem;
  width: 100%; max-width: 400px;
}
h2 { margin: 0 0 1.25rem; }
.gen-result { display: flex; gap: 0.5rem; margin-bottom: 1rem; }
.gen-input {
  flex: 1; padding: 0.5rem 0.75rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.9375rem; font-family: Consolas, monospace;
  background: var(--bg); color: var(--text);
}
.gen-input:focus { outline: none; border-color: var(--primary); }
.btn-copy {
  padding: 0.5rem 0.75rem; background: var(--hover-bg); border: 1px solid var(--border);
  border-radius: 6px; cursor: pointer; font-size: 0.8125rem; white-space: nowrap; color: inherit;
}
.btn-copy:hover { border-color: var(--primary); color: var(--primary); }
.gen-controls { display: flex; flex-direction: column; gap: 0.5rem; margin-bottom: 1rem; }
.gen-tabs { display: flex; gap: 0.25rem; margin-bottom: 0.75rem; }
.gen-tabs button {
  flex: 1; padding: 0.375rem 0.5rem; border: 1px solid var(--border);
  border-radius: 6px; background: var(--bg); color: var(--text); cursor: pointer;
  font-size: 0.8125rem;
}
.gen-tabs button.active { background: var(--primary); color: #fff; border-color: var(--primary); }
.gen-tabs button:hover:not(.active) { border-color: var(--primary); }
.gen-select {
  width: 100%; padding: 0.375rem 0.5rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.875rem; background: var(--bg); color: var(--text);
}
.gen-select:focus { outline: none; border-color: var(--primary); }
.field label { font-size: 0.875rem; font-weight: 500; }
.field input[type="range"] { width: 100%; }
.checkbox-row {
  display: flex; align-items: center; gap: 0.5rem;
  font-size: 0.875rem; cursor: pointer;
}
.modal-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
.btn-cancel, .btn-primary, .btn-save {
  padding: 0.5rem 1rem; border: none; border-radius: 6px; cursor: pointer; font-size: 0.875rem;
}
.btn-cancel { background: var(--hover-bg); color: var(--text); }
.btn-primary { background: var(--hover-bg); color: var(--text); border: 1px solid var(--border); }
.btn-save { background: var(--primary); color: #fff; }
</style>
