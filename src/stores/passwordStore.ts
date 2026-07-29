import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface EmailInfo {
  email: string
  is_primary: boolean
}

export interface PasswordEntry {
  id: string
  site_url: string
  username: string
  password: string
  emails_raw: string | null
  phone: string | null
  twofa_secret: string | null
  note: string
  autofill_mode: string
  created_at: number
  updated_at: number
}

export interface NewEntry {
  site_url: string
  username: string
  password: string
  emails_raw: string | null
  phone: string | null
  twofa_secret: string | null
  note: string
  autofill_mode: string
}

export function parseEmails(raw: string | null): EmailInfo[] {
  if (!raw) return []
  try {
    return JSON.parse(raw)
  } catch {
    return raw ? [{ email: raw, is_primary: true }] : []
  }
}

export function serializeEmails(list: EmailInfo[]): string {
  return JSON.stringify(list)
}

export const usePasswordStore = defineStore('password', () => {
  const entries = ref<PasswordEntry[]>([])
  const searchQuery = ref('')
  const loading = ref(false)

  async function fetchEntries(search?: string) {
    loading.value = true
    try {
      entries.value = await invoke('list_entries', { search: search || null })
    } finally {
      loading.value = false
    }
  }

  async function getEntry(id: string, password: string): Promise<PasswordEntry> {
    return await invoke('get_entry', { id, password })
  }

  async function addEntry(entry: NewEntry, password: string) {
    await invoke('add_entry', { entry, password })
    await fetchEntries(searchQuery.value)
  }

  async function editEntry(id: string, entry: NewEntry, password: string) {
    await invoke('edit_entry', { id, entry, password })
    await fetchEntries(searchQuery.value)
  }

  async function deleteEntry(id: string) {
    await invoke('delete_entry', { id })
    entries.value = entries.value.filter((e) => e.id !== id)
  }

  async function changeMasterPassword(old: string, new_: string) {
    await invoke('change_master_password', { old, new: new_ })
  }

  async function generateTotp(secret: string, stepOffset?: number): Promise<string> {
    return await invoke('generate_totp', { secret, stepOffset })
  }

  async function generatePassword(
    length: number,
    useUpper: boolean,
    useLower: boolean,
    useDigits: boolean,
    useSymbols: boolean,
    excludeConfusing: boolean
  ): Promise<string> {
    return await invoke('generate_password', {
      length, useUpper, useLower, useDigits, useSymbols, excludeConfusing,
    })
  }

  return {
    entries,
    searchQuery,
    loading,
    fetchEntries,
    getEntry,
    addEntry,
    editEntry,
    deleteEntry,
    changeMasterPassword,
    generateTotp,
    generatePassword,
  }
})
