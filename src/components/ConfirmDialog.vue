<template>
  <Teleport to="body">
    <div v-if="state.visible" class="modal-overlay" @click.self="cancel">
      <div class="modal dialog-modal">
        <h3>{{ state.opts.title }}</h3>
        <p v-if="state.opts.message" class="dialog-message">{{ state.opts.message }}</p>
        <input
          v-if="state.opts.showInput"
          v-model="state.inputValue"
          :type="state.opts.inputType || 'text'"
          :placeholder="state.opts.inputPlaceholder"
          class="dialog-input"
          ref="inputRef"
          @keydown.enter="confirm"
        />
        <div class="dialog-actions">
          <button class="btn-cancel" @click="cancel">{{ state.opts.cancelText || i18n.t('dialog.cancel') }}</button>
          <button class="btn-dialog-confirm" @click="confirm">{{ state.opts.confirmText || i18n.t('dialog.confirm') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { watch, ref, nextTick } from 'vue'
import { dialogState, confirmDialog, cancelDialog } from '../stores/dialogStore'
import { useI18nStore } from '../stores/i18nStore'

const state = dialogState()
const i18n = useI18nStore()
const inputRef = ref<HTMLInputElement>()

watch(() => state.visible, (v) => {
  if (v && state.opts.showInput) {
    nextTick(() => inputRef.value?.focus())
  }
})

function confirm() { confirmDialog() }
function cancel() { cancelDialog() }
</script>

<style>
.modal-overlay {
  position: fixed; inset: 0; z-index: 9998;
  background: rgba(0, 0, 0, 0.4);
  display: flex; align-items: center; justify-content: center;
}
.modal {
  background: var(--card-bg); border: 1px solid var(--border);
  border-radius: 12px; padding: 1.5rem;
  box-shadow: var(--shadow); min-width: 320px;
}
</style>

<style scoped>
.dialog-modal { max-width: 400px; }
.dialog-message { margin: 0.5rem 0 1rem; color: var(--text-secondary); font-size: 0.9375rem; line-height: 1.5; }
.dialog-input { width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--border); border-radius: 6px; background: var(--bg); color: var(--text); font-size: 0.875rem; margin-bottom: 1rem; outline: none; }
.dialog-input:focus { border-color: var(--primary); }
.dialog-actions { display: flex; gap: 0.5rem; justify-content: flex-end; }
.dialog-actions button { padding: 0.5rem 1rem; border-radius: 6px; cursor: pointer; font-size: 0.875rem; border: none; }
.btn-cancel { background: var(--hover-bg); border: 1px solid var(--border) !important; color: var(--text); }
.btn-cancel:hover { border-color: var(--primary); color: var(--primary); }
.btn-dialog-confirm { background: var(--primary); color: #fff; }
.btn-dialog-confirm:hover { opacity: 0.85; }
</style>
