<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>{{ i18n.t('app.title') }}</h2>
      </div>
      <nav>
        <router-link to="/list" class="nav-item active">
          <span class="nav-icon">&#128273;</span> {{ i18n.t('nav.all_passwords') }}
        </router-link>
        <router-link to="/settings" class="nav-item">
          <span class="nav-icon">&#9881;</span> {{ i18n.t('nav.settings') }}
        </router-link>
      </nav>
      <div class="sidebar-stats">
        <span class="stats-label">{{ i18n.t('list.total_sites') }}</span>
        <span class="stats-value">{{ store.entries.length }}</span>
      </div>
      <div class="sidebar-footer">
        <button class="btn-logout" @click="lock">{{ i18n.t('nav.lock') }}</button>
      </div>
    </aside>

    <main class="main-content">
      <header class="top-bar">
        <div class="search-box">
          <input
            v-model="searchQuery"
            type="text"
            :placeholder="i18n.t('list.search_placeholder')"
            @input="onSearch"
          />
        </div>
        <button class="btn-primary" @click="showAddForm = true">{{ i18n.t('list.add') }}</button>
      </header>

      <div class="content-body">
        <div v-if="loading" class="loading">{{ i18n.t('list.loading') }}</div>
        <div v-else-if="store.entries.length === 0" class="empty">
          <p v-if="searchQuery">{{ i18n.t('list.empty_search') }}</p>
          <p v-else>{{ i18n.t('list.empty') }}</p>
          <button v-if="searchQuery" class="clear-search" @click="clearSearch">{{ i18n.t('list.clear_search') }}</button>
        </div>
        <div v-else class="entries-list">
          <div
            v-for="entry in store.entries"
            :key="entry.id"
            class="entry-card"
            @click="viewEntry(entry.id)"
          >
            <div class="entry-main">
              <div class="entry-site">{{ entry.site_url || i18n.t('list.untitled') }}</div>
              <div class="entry-username">{{ autofillLabel(entry) }}</div>
            </div>
            <div class="entry-meta">
              <span class="entry-date">{{ formatDate(entry.updated_at) }}</span>
            </div>
          </div>
        </div>
      </div>
    </main>

    <PasswordForm
      v-if="showAddForm"
      :visible="showAddForm"
      @close="showAddForm = false"
      @saved="onSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { usePasswordStore } from '../stores/passwordStore'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'
import PasswordForm from '../components/PasswordForm.vue'

const router = useRouter()
const store = usePasswordStore()
const auth = useAuthStore()
const i18n = useI18nStore()

const searchQuery = ref('')
const loading = ref(true)
const showAddForm = ref(false)

onMounted(async () => {
  if (!auth.checkSession()) {
    router.push('/')
    return
  }
  await store.fetchEntries()
  loading.value = false
})

let searchTimer: ReturnType<typeof setTimeout>
function onSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(() => {
    store.fetchEntries(searchQuery.value || undefined)
  }, 300)
}

function viewEntry(id: string) {
  router.push(`/entry/${id}`)
}

function clearSearch() {
  searchQuery.value = ''
  store.fetchEntries()
}

function onSaved() {
  showAddForm.value = false
  store.fetchEntries(searchQuery.value || undefined)
}

function lock() {
  auth.clearSession()
  router.push('/')
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString()
}

function autofillLabel(entry: any): string {
  if (entry.autofill_mode === 'primary_email') return i18n.t('detail.see_primary_email')
  if (entry.autofill_mode === 'phone') return i18n.t('detail.see_phone')
  if (entry.autofill_mode === 'none') return ''
  return entry.username
}
</script>

<style scoped>
.layout { display: flex; height: 100vh; }
.sidebar {
  width: 240px; background: var(--card-bg);
  border-right: 1px solid var(--border); display: flex;
  flex-direction: column; flex-shrink: 0;
}
.sidebar-header { padding: 1.25rem; border-bottom: 1px solid var(--border); }
.sidebar-header h2 { margin: 0; font-size: 1.125rem; }
nav { flex: 1; padding: 0.75rem; }
.nav-item {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.625rem 0.75rem; border-radius: 6px;
  color: var(--text); text-decoration: none; font-size: 0.9375rem;
}
.nav-item:hover, .nav-item.active { background: var(--hover-bg); }
.nav-icon { font-size: 1.125rem; }
.sidebar-stats { padding: 0.75rem 1.25rem; display: flex; justify-content: space-between; align-items: center; font-size: 0.8125rem; }
.sidebar-stats .stats-label { color: var(--text-secondary); }
.sidebar-stats .stats-value { font-weight: 600; color: var(--primary); }
.sidebar-footer { padding: 1rem; border-top: 1px solid var(--border); }
.btn-logout {
  width: 100%; padding: 0.5rem; background: none;
  border: 1px solid var(--border); border-radius: 6px; color: var(--text); cursor: pointer;
}
.main-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.top-bar {
  display: flex; align-items: center; gap: 1rem;
  padding: 1rem 1.5rem; border-bottom: 1px solid var(--border);
}
.search-box { flex: 1; }
.search-box input {
  width: 100%; padding: 0.5rem 0.75rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.9375rem; background: var(--bg); color: var(--text); box-sizing: border-box;
}
.search-box input:focus { outline: none; border-color: var(--primary); }
.btn-primary {
  padding: 0.5rem 1rem; background: var(--primary); color: #fff;
  border: none; border-radius: 6px; font-size: 0.9375rem; cursor: pointer; white-space: nowrap;
}
.content-body { flex: 1; overflow-y: auto; padding: 1rem 1.5rem; }
.loading, .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); }
.entries-list { display: grid; gap: 0.5rem; }
.entry-card {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.875rem 1rem; background: var(--card-bg);
  border: 1px solid var(--border); border-radius: 8px; cursor: pointer; transition: border-color 0.15s;
}
.entry-card:hover { border-color: var(--primary); }
.entry-site { font-weight: 500; margin-bottom: 0.125rem; }
.entry-username { font-size: 0.8125rem; color: var(--text-secondary); }
.entry-date { font-size: 0.75rem; color: var(--text-secondary); }
.clear-search {
  display: block; margin-top: 0.5rem; padding: 0.375rem 0.75rem;
  background: none; border: 1px solid var(--primary); border-radius: 4px;
  color: var(--primary); cursor: pointer; font-size: 0.875rem;
}
.clear-search:hover { background: var(--primary); color: #fff; }
</style>
