<template>
  <div v-if="password" class="strength-meter">
    <div class="strength-bar">
      <div class="strength-fill" :class="levelClass" :style="{ width: score + '%' }"></div>
    </div>
    <span class="strength-label" :class="levelClass">{{ label }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18nStore } from '../stores/i18nStore'

const props = defineProps<{ password: string }>()

const i18n = useI18nStore()

const WEAK_PATTERNS = [
  /^123456$/i, /^12345678$/i, /^123456789$/i, /^password$/i,
  /^qwerty$/i, /^abc123$/i, /^admin123$/i, /^passw0rd$/i,
  /^letmein$/i, /^welcome$/i, /^monkey$/i, /^dragon$/i,
  /^master$/i, /^123456Aa$/i, /^Aa123456$/i,
]

const score = computed(() => {
  const pwd = props.password
  if (!pwd) return 0

  let s = 0

  s += Math.min(pwd.length * 2, 40)

  if (/[A-Z]/.test(pwd)) s += 10
  if (/[a-z]/.test(pwd)) s += 10
  if (/[0-9]/.test(pwd)) s += 10
  if (/[^A-Za-z0-9]/.test(pwd)) s += 15

  if (pwd.length >= 12) s += 5
  if (pwd.length >= 16) s += 5
  if (pwd.length >= 20) s += 5

  for (const pattern of WEAK_PATTERNS) {
    if (pattern.test(pwd)) {
      s = Math.min(s, 20)
      break
    }
  }
  const digits = (pwd.match(/\d/g) || []).length
  if (pwd.length >= 6 && digits === pwd.length && pwd.length <= 20) {
    s = Math.min(s, 15)
  }

  return Math.min(s, 100)
})

type Level = 'very-weak' | 'weak' | 'fair' | 'strong' | 'very-strong'

const level = computed<Level>(() => {
  const s = score.value
  if (s <= 20) return 'very-weak'
  if (s <= 40) return 'weak'
  if (s <= 60) return 'fair'
  if (s <= 80) return 'strong'
  return 'very-strong'
})

const levelClass = computed(() => 'level-' + level.value)

const label = computed(() => {
  const key = 'strength.' + level.value.replace('-', '_')
  return i18n.t(key)
})
</script>

<style scoped>
.strength-meter {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-top: 0.25rem;
}
.strength-bar {
  flex: 1;
  height: 4px;
  background: var(--border);
  border-radius: 2px;
  overflow: hidden;
}
.strength-fill {
  height: 100%;
  border-radius: 2px;
  transition: width 0.2s ease, background 0.2s ease;
}
.strength-label {
  font-size: 0.6875rem;
  white-space: nowrap;
  min-width: 5em;
  text-align: right;
}
.level-very-weak { background: #e53e3e; color: #e53e3e; }
.level-weak { background: #dd6b20; color: #dd6b20; }
.level-fair { background: #d69e2e; color: #d69e2e; }
.level-strong { background: #38a169; color: #38a169; }
.level-very-strong { background: #2f855a; color: #2f855a; }
</style>
