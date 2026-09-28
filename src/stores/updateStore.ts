import { computed, ref, shallowRef } from 'vue'
import { defineStore } from 'pinia'
import { check, type DownloadEvent, type Update } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { commandErrorMessage } from '@/shared/api/commandError'
import { useAppStore } from './appStore'
import { version as currentVersion } from '../../package.json'

interface CheckOptions {
  silent?: boolean
}

function updateErrorMessage(error: unknown): string {
  const message = commandErrorMessage(error)
  if (message.includes('invalid updater binary format')) {
    return '更新包格式与当前安装方式不匹配，请下载对应的 deb/AppImage 安装包后重试'
  }
  return message
}

export const useUpdateStore = defineStore('updates', () => {
  const checking = ref(false)
  const installing = ref(false)
  const availableUpdate = shallowRef<Update | null>(null)
  const downloadedBytes = ref(0)
  const contentLength = ref<number | null>(null)

  const hasAvailableUpdate = computed(() => availableUpdate.value !== null)
  const availableVersion = computed(() => availableUpdate.value?.version || '')
  const progressPercent = computed(() => {
    if (!contentLength.value) return null
    return Math.min(100, Math.round((downloadedBytes.value / contentLength.value) * 100))
  })
  const installButtonText = computed(() => {
    if (installing.value) {
      return progressPercent.value === null ? '正在更新' : `更新中 ${progressPercent.value}%`
    }
    return availableVersion.value ? `更新到 ${availableVersion.value}` : '立即更新'
  })
  const updateSummary = computed(() => {
    if (installing.value) {
      return progressPercent.value === null ? '正在下载更新' : `正在下载更新 ${progressPercent.value}%`
    }
    if (checking.value) return '正在检查更新'
    if (availableVersion.value) return `发现新版本 ${availableVersion.value}`
    return `当前版本 v${currentVersion}`
  })

  function resetProgress() {
    downloadedBytes.value = 0
    contentLength.value = null
  }

  function applyDownloadEvent(event: DownloadEvent) {
    if (event.event === 'Started') {
      downloadedBytes.value = 0
      contentLength.value = event.data.contentLength ?? null
      return
    }
    if (event.event === 'Progress') {
      downloadedBytes.value += event.data.chunkLength
      return
    }
    if (event.event === 'Finished' && contentLength.value) {
      downloadedBytes.value = contentLength.value
    }
  }

  async function checkForUpdates(options: CheckOptions = {}) {
    if (checking.value || installing.value) return availableUpdate.value
    checking.value = true
    resetProgress()
    try {
      const update = await check()
      availableUpdate.value = update
      if (!options.silent) {
        useAppStore().setMessage('success', update ? `发现新版本 ${update.version}` : '已是最新版本')
      }
      return update
    } catch (error) {
      if (!options.silent) {
        useAppStore().setMessage('error', updateErrorMessage(error))
      }
      return null
    } finally {
      checking.value = false
    }
  }

  async function installAvailableUpdate() {
    if (installing.value) return
    const update = availableUpdate.value || (await checkForUpdates({ silent: false }))
    if (!update) return

    installing.value = true
    resetProgress()
    try {
      await update.downloadAndInstall(applyDownloadEvent)
      useAppStore().setMessage('success', '更新已安装，正在重启')
      await relaunch()
    } catch (error) {
      useAppStore().setMessage('error', updateErrorMessage(error))
    } finally {
      installing.value = false
    }
  }

  return {
    checking,
    installing,
    availableUpdate,
    downloadedBytes,
    contentLength,
    hasAvailableUpdate,
    availableVersion,
    progressPercent,
    installButtonText,
    updateSummary,
    checkForUpdates,
    installAvailableUpdate,
  }
})
