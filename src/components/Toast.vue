<template>
  <Teleport to="body">
    <div class="toast-container">
      <TransitionGroup name="toast">
        <div v-for="t in toasts" :key="t.id" :class="['toast', t.type]">
          {{ t.message }}
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '../stores/toastStore'

const { toasts } = useToast()
</script>

<style>
.toast-container {
  position: fixed;
  bottom: 1.5rem;
  right: 1.5rem;
  z-index: 99999;
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  pointer-events: none;
}
.toast {
  padding: 0.625rem 1rem;
  border-radius: 8px;
  font-size: 0.875rem;
  box-shadow: 0 2px 12px rgba(0,0,0,0.15);
  pointer-events: auto;
  max-width: 360px;
  word-break: break-word;
}
.toast.info { background: var(--card-bg); border: 1px solid var(--border); color: var(--text); }
.toast.success { background: var(--success); color: #fff; }
.toast.error { background: var(--danger); color: #fff; }

.toast-enter-active { transition: all 0.25s ease-out; }
.toast-leave-active { transition: all 0.2s ease-in; }
.toast-enter-from { opacity: 0; transform: translateY(0.75rem); }
.toast-leave-to { opacity: 0; transform: translateX(1rem); }
</style>
