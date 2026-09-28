import { defineStore } from 'pinia'
import { ref } from 'vue'
import { api } from '@/shared/api/tauri'

export const usePasswordStore = defineStore('passwords', () => {
  const savedPasswords = ref<Record<string, boolean>>({})

  function hasPassword(tunnelId: string): boolean {
    return Boolean(savedPasswords.value[tunnelId])
  }

  async function refresh(tunnelId: string) {
    if (!tunnelId) return
    try {
      savedPasswords.value = { ...savedPasswords.value, [tunnelId]: await api.hasTunnelPassword(tunnelId) }
    } catch {
      savedPasswords.value = { ...savedPasswords.value, [tunnelId]: false }
    }
  }

  async function save(tunnelId: string, value: string) {
    if (!value) return
    await api.saveTunnelPassword(tunnelId, value)
    savedPasswords.value = { ...savedPasswords.value, [tunnelId]: true }
  }

  async function clear(tunnelId: string) {
    await api.deleteTunnelPassword(tunnelId)
    savedPasswords.value = { ...savedPasswords.value, [tunnelId]: false }
  }

  return { savedPasswords, hasPassword, refresh, save, clear }
})
