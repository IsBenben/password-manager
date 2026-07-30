<template>
  <aside class="sidebar">
    <div class="sidebar-header">
      <h2>{{ i18n.t('app.title') }}</h2>
    </div>
    <nav class="sidebar-nav">
      <a class="nav-item" :class="{ active: activeFilter == null }" @click="$emit('filter')">
        <span class="nav-icon">&#128273;</span> {{ i18n.t('nav.all_passwords') }}
      </a>
      <a class="nav-item" :class="{ active: activeFilter === '__fav__' }" @click="$emit('filter', '__fav__')">
        <span class="nav-icon">&#9733;</span> {{ i18n.t('nav.favorites') }}
      </a>
      <div class="nav-divider"></div>
      <div class="nav-section-label">{{ i18n.t('nav.category') }}</div>
      <a
        v-for="cat in store.categories"
        :key="cat.name"
        class="nav-item nav-cat"
        :class="{ active: activeFilter === cat.name }"
        @click="$emit('filter', cat.name)"
      >
        <span class="nav-icon">{{ cat.name ? '#' : '' }}</span>
        <span>{{ cat.name || i18n.t('nav.uncategorized') }}</span>
        <span class="nav-count">{{ cat.count }}</span>
      </a>
      <div class="nav-divider"></div>
      <router-link to="/settings" class="nav-item">
        <span class="nav-icon">&#9881;</span> {{ i18n.t('nav.settings') }}
      </router-link>
    </nav>
    <div class="sidebar-footer">
      <button class="btn-logout" @click="lock">{{ i18n.t('nav.lock') }}</button>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router'
import { usePasswordStore } from '../stores/passwordStore'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'

defineProps<{
  activeFilter?: string | null
}>()

defineEmits<{
  filter: [cat?: string]
}>()

const router = useRouter()
const store = usePasswordStore()
const auth = useAuthStore()
const i18n = useI18nStore()

function lock() {
  auth.clearSession()
  router.push('/')
}
</script>

<style scoped>
.sidebar {
  width: 240px; background: var(--card-bg);
  border-right: 1px solid var(--border); display: flex;
  flex-direction: column; flex-shrink: 0;
}
.sidebar-header { padding: 1.25rem; border-bottom: 1px solid var(--border); }
.sidebar-header h2 { margin: 0; font-size: 1.125rem; }
.sidebar-nav { flex: 1; padding: 0.75rem; overflow-y: auto; }
.nav-item {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.5rem 0.75rem; border-radius: 6px;
  color: var(--text); cursor: pointer; font-size: 0.875rem; text-decoration: none;
}
.nav-item:hover, .nav-item.active { background: var(--hover-bg); }
.nav-item.active { color: var(--primary); font-weight: 500; }
.nav-icon { font-size: 1rem; width: 1.25rem; text-align: center; }
.nav-divider { height: 1px; background: var(--border); margin: 0.5rem 0.75rem; }
.nav-section-label { font-size: 0.6875rem; color: var(--text-secondary); padding: 0.25rem 0.75rem; text-transform: uppercase; letter-spacing: 0.05em; }
.nav-cat { padding-left: 1rem; }
.nav-count { margin-left: auto; font-size: 0.75rem; color: var(--text-secondary); }
.sidebar-footer { padding: 1rem; border-top: 1px solid var(--border); }
.btn-logout {
  width: 100%; padding: 0.5rem; background: none;
  border: 1px solid var(--border); border-radius: 6px; color: var(--text-secondary); cursor: pointer;
}
.btn-logout:hover { border-color: var(--primary); color: var(--primary); }
</style>
