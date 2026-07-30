<template>
  <div class="layout" @keydown="handleKeydown" tabindex="-1">
    <AppSidebar :activeFilter="activeFilter" @filter="setFilter" />

    <main class="main-content">
      <header class="top-bar">
        <div class="search-box">
          <div class="search-input-wrap">
            <input
              v-model="searchQuery"
              type="text"
              :placeholder="i18n.t('list.search_placeholder')"
              @input="onSearch"
              ref="searchInputRef"
            />
            <button v-if="searchQuery" class="search-clear" @click="clearSearch" :title="i18n.t('list.clear_search')">&#x2715;</button>
          </div>
          <div class="search-info">
            <span v-if="searchQuery">{{ i18n.t('list.search_count', String(sortedEntries.length), String(totalCount)) }}</span>
            <span v-else>{{ i18n.t('list.total_sites') }}: {{ totalCount }}</span>
          </div>
        </div>
        <div class="sort-controls">
          <select v-model="sortBy" class="sort-select">
            <option value="date">{{ i18n.t('list.sort_date') }}</option>
            <option value="name">{{ i18n.t('list.sort_name') }}</option>
            <option value="site">{{ i18n.t('list.sort_site') }}</option>
          </select>
          <button class="btn-sort-order" @click="sortOrder = sortOrder === 'asc' ? 'desc' : 'asc'" :title="sortOrder === 'asc' ? '↑' : '↓'">
            {{ sortOrder === 'asc' ? '↑' : '↓' }}
          </button>
        </div>
        <button class="btn-primary" @click="showAddForm = true">{{ i18n.t('list.add') }}</button>
      </header>

      <div class="content-body">
        <div v-if="loading" class="skeleton-list">
          <div v-for="n in 6" :key="n" class="skeleton-card">
            <div class="skeleton-line w-60"></div>
            <div class="skeleton-line w-40"></div>
          </div>
        </div>
        <div v-else-if="sortedEntries.length === 0" class="empty-state">
          <div class="empty-icon">&#128273;</div>
          <p v-if="searchQuery" class="empty-title">{{ i18n.t('list.empty_search') }}</p>
          <p v-else class="empty-title">{{ i18n.t('list.empty') }}</p>
          <p class="empty-hint">{{ i18n.t('list.empty_hint') }}</p>
        </div>
        <div v-else class="entries-list">
          <div
            v-for="entry in sortedEntries"
            :key="entry.id"
            class="entry-card"
            @click="viewEntry(entry.id)"
          >
            <button class="btn-star" :class="{ starred: entry.favorite }" @click.stop="toggleFav(entry)">&#9733;</button>
            <div class="entry-main">
              <div class="entry-site">{{ entry.site_url || i18n.t('list.untitled') }}</div>
              <div class="entry-username">{{ autofillLabel(entry) }}</div>
              <div v-if="entry.category" class="entry-tags">
                <span v-for="t in entry.category.split(/\s+/)" :key="t" class="entry-category">{{ t }}</span>
              </div>
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
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { usePasswordStore, type PasswordEntry } from '../stores/passwordStore'
import { useAuthStore } from '../stores/authStore'
import { useI18nStore } from '../stores/i18nStore'
import PasswordForm from '../components/PasswordForm.vue'
import AppSidebar from '../components/AppSidebar.vue'
import { useToast } from '../stores/toastStore'

const router = useRouter()
const route = useRoute()
const store = usePasswordStore()
const auth = useAuthStore()
const i18n = useI18nStore()
const toast = useToast()

const searchQuery = ref('')
const searchInputRef = ref<HTMLInputElement | null>(null)
const loading = ref(true)
const showAddForm = ref(false)
const totalCount = ref(0)
const activeFilter = ref<string | null>(null)
const sortBy = ref<'name' | 'date' | 'site'>('date')
const sortOrder = ref<'asc' | 'desc'>('desc')

const sortedEntries = computed(() => {
  const list = [...store.entries]
  list.sort((a, b) => {
    if (a.favorite !== b.favorite) return a.favorite ? -1 : 1
    let cmp = 0
    if (sortBy.value === 'name') cmp = a.username.localeCompare(b.username)
    else if (sortBy.value === 'site') cmp = a.site_url.localeCompare(b.site_url)
    else cmp = a.updated_at - b.updated_at
    return sortOrder.value === 'asc' ? cmp : -cmp
  })
  return list
})

onMounted(async () => {
  if (!auth.checkSession()) {
    router.push('/')
    return
  }
  await store.fetchEntries()
  totalCount.value = store.entries.length
  await store.fetchCategories()

  const fav = route.query.fav
  const cat = route.query.cat
  if (fav === '1') {
    activeFilter.value = '__fav__'
  } else if (typeof cat === 'string') {
    activeFilter.value = cat
  }
  if (activeFilter.value !== null) {
    await store.fetchEntries(undefined, filterCategory(), filterFavorite())
  }

  loading.value = false
})

function handleKeydown(e: KeyboardEvent) {
  if (e.ctrlKey || e.metaKey) {
    if (e.key === 'f') {
      e.preventDefault()
      searchInputRef.value?.focus()
      return
    }
    if (e.key === 'n') {
      e.preventDefault()
      showAddForm.value = true
      return
    }
  }
  if (e.key === 'Escape') {
    if (searchQuery.value) {
      clearSearch()
    } else if (document.activeElement instanceof HTMLElement) {
      document.activeElement.blur()
    }
  }
}

onMounted(() => window.addEventListener('keydown', handleKeydown, { capture: true }))
onUnmounted(() => window.removeEventListener('keydown', handleKeydown, { capture: true }))

let searchTimer: ReturnType<typeof setTimeout>
async function doSearch() {
  await store.fetchEntries(searchQuery.value || undefined, filterCategory(), filterFavorite())
}
function onSearch() {
  clearTimeout(searchTimer)
  searchTimer = setTimeout(doSearch, 300)
}

function filterCategory(): string | undefined {
  if (activeFilter.value === null || activeFilter.value === '__fav__') return undefined
  return activeFilter.value
}

function filterFavorite(): boolean | undefined {
  return activeFilter.value === '__fav__' ? true : undefined
}

async function applyFilter() {
  loading.value = true
  await store.fetchEntries(searchQuery.value || undefined, filterCategory(), filterFavorite())
  totalCount.value = store.entries.length
  loading.value = false
}

function setFilter(cat?: string) {
  activeFilter.value = cat !== undefined ? cat : null
  searchQuery.value = ''
  applyFilter()
}

function viewEntry(id: string) {
  router.push(`/entry/${id}`)
}

function clearSearch() {
  searchQuery.value = ''
  applyFilter()
}

async function toggleFav(entry: any) {
  await store.toggleFavorite(entry.id)
  entry.favorite = !entry.favorite
  toast.success(i18n.t(entry.favorite ? 'toast.fav_on' : 'toast.fav_off'))
  await store.fetchCategories()
}

async function onSaved() {
  showAddForm.value = false
  await store.fetchEntries()
  totalCount.value = store.entries.length
  await store.fetchCategories()
  if (searchQuery.value || activeFilter.value !== null) {
    await store.fetchEntries(searchQuery.value || undefined, filterCategory(), filterFavorite())
  }
}

function formatDate(ts: number): string {
  return new Date(ts * 1000).toLocaleDateString()
}

function autofillLabel(entry: PasswordEntry): string {
  if (entry.autofill_mode === 'primary_email') return i18n.t('detail.see_primary_email')
  if (entry.autofill_mode === 'phone') return i18n.t('detail.see_phone')
  if (entry.autofill_mode === 'none') return ''
  return entry.username
}
</script>

<style scoped>
.layout { display: flex; height: 100vh; }
.main-content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.top-bar {
  display: flex; align-items: flex-start; gap: 1rem;
  padding: 1rem 1.5rem; border-bottom: 1px solid var(--border);
}
.search-box { flex: 1; }
.search-input-wrap { position: relative; display: flex; align-items: center; }
.search-input-wrap input {
  width: 100%; padding: 0.5rem 2rem 0.5rem 0.75rem; border: 1px solid var(--border);
  border-radius: 6px; font-size: 0.9375rem; background: var(--bg); color: var(--text); box-sizing: border-box;
}
.search-input-wrap input:focus { outline: none; border-color: var(--primary); }
.search-clear {
  position: absolute; right: 0.375rem; top: 50%; transform: translateY(-50%);
  width: 1.25rem; height: 1.25rem; padding: 0; border: none; border-radius: 50%;
  background: var(--text-secondary); color: var(--bg); font-size: 0.625rem;
  line-height: 1.25rem; text-align: center; cursor: pointer;
}
.search-clear:hover { background: var(--primary); }
.search-info { font-size: 0.75rem; color: var(--text-secondary); margin-top: 0.125rem; }
.btn-primary {
  padding: 0.5rem 1rem; background: var(--primary); color: #fff;
  border: none; border-radius: 6px; font-size: 0.9375rem; cursor: pointer; white-space: nowrap;
}
.content-body { flex: 1; overflow-y: auto; padding: 1rem 1.5rem; }
.loading, .empty { display: flex; align-items: center; justify-content: center; height: 200px; color: var(--text-secondary); }
.entries-list { display: grid; gap: 0.5rem; }
.entry-card {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0.75rem 1rem; background: var(--card-bg);
  border: 1px solid var(--border); border-radius: 8px; cursor: pointer; transition: border-color 0.15s;
}
.entry-card:hover { border-color: var(--primary); }
.btn-star {
  flex-shrink: 0; width: 1.5rem; height: 1.5rem; padding: 0; border: none;
  background: none; cursor: pointer; font-size: 1rem; line-height: 1;
  color: var(--text-secondary); transition: color 0.15s;
}
.btn-star:hover { color: #d69e2e; }
.btn-star.starred { color: #d69e2e; }
.entry-main { flex: 1; min-width: 0; }
.entry-site { font-weight: 500; margin-bottom: 0.125rem; }
.entry-site a { color: inherit; text-decoration: none; }
.entry-site a:hover { text-decoration: underline; color: var(--primary); }
.entry-username { font-size: 0.8125rem; color: var(--text-secondary); }
.entry-tags { display: flex; gap: 0.25rem; flex-wrap: wrap; margin-top: 0.125rem; }
.entry-category { display: inline-block; font-size: 0.625rem; padding: 0.0625rem 0.375rem; border-radius: 3px; background: var(--hover-bg); color: var(--text-secondary); }
.entry-date { font-size: 0.75rem; color: var(--text-secondary); }
.sort-controls { display: flex; align-items: center; gap: 0.25rem; }
.sort-select {
  padding: 0.375rem 0.5rem; border: 1px solid var(--border); border-radius: 6px;
  font-size: 0.8125rem; background: var(--bg); color: var(--text); cursor: pointer;
}
.sort-select:focus { outline: none; border-color: var(--primary); }
.btn-sort-order {
  width: 28px; height: 28px; padding: 0; border: 1px solid var(--border);
  border-radius: 6px; background: var(--bg); color: var(--text); cursor: pointer; font-size: 0.875rem;
}
.btn-sort-order:hover { border-color: var(--primary); color: var(--primary); }
.skeleton-list { display: grid; gap: 0.5rem; padding: 0.5rem 0; }
.skeleton-card {
  padding: 1rem; background: var(--card-bg); border: 1px solid var(--border);
  border-radius: 8px; display: flex; flex-direction: column; gap: 0.5rem;
}
.skeleton-line {
  height: 14px; border-radius: 4px;
  background: linear-gradient(90deg, var(--border) 25%, var(--hover-bg) 50%, var(--border) 75%);
  background-size: 200% 100%; animation: shimmer 1.5s infinite;
}
.skeleton-line.w-60 { width: 60%; }
.skeleton-line.w-40 { width: 40%; }
@keyframes shimmer { 0% { background-position: 200% 0; } 100% { background-position: -200% 0; } }
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; height: 300px; color: var(--text-secondary); }
.empty-icon { font-size: 3rem; margin-bottom: 0.75rem; opacity: 0.5; }
.empty-title { font-size: 1rem; margin: 0 0 0.25rem; }
.empty-hint { font-size: 0.8125rem; margin: 0; }
</style>
