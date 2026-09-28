<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref } from 'vue'
import { exit } from '@tauri-apps/plugin-process'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit, listen } from '@tauri-apps/api/event'
import { useAppStore } from '@/stores/appStore'
import { api } from '@/shared/api/tauri'
import type { CloseAction } from '@/shared/types'

const store = useAppStore()
const appWindow = hasTauriWindow() ? getCurrentWindow() : null
const open = ref(false)
const remember = ref(false)
let unlisten: (() => void) | undefined
let unlistenExit: (() => void) | undefined

function hasTauriWindow() {
  const tauriWindow = window as typeof window & { __TAURI_INTERNALS__?: { metadata?: unknown } }
  return Boolean(tauriWindow.__TAURI_INTERNALS__?.metadata)
}

async function minimizeToTray() {
  await appWindow?.hide()
}

async function exitApp() {
  if (store.settings.behavior.cleanupOnExit) {
    await api.repairHosts().catch(() => {})
  }
  await exit(0)
}

async function persistRememberedAction(action: Exclude<CloseAction, 'ask'>) {
  if (!remember.value) return true
  store.settings.behavior.closeAction = action
  return store.saveSettingsOnly('关闭行为已保存')
}

async function choose(action: Exclude<CloseAction, 'ask'>) {
  if (store.loading || !await persistRememberedAction(action)) return
  if (store.settingsSaveState === 'error' && !await store.saveSettingsOnly()) return
  open.value = false
  if (action === 'minimizeToTray') {
    await minimizeToTray()
    return
  }
  await exitApp()
}

async function handleCloseRequest() {
  if (store.loading || store.serviceEditorOpen) {
    store.setMessage('info', store.serviceEditorOpen ? '请先完成或取消服务编辑，再关闭应用' : '正在处理，请稍候再关闭应用')
    return
  }
  if (store.profilesDirty) {
    await store.runAfterUnsavedProfilesConfirm(handleCloseRequest)
    return
  }
  if (store.settingsSaveState === 'error' && !await store.saveSettingsOnly()) return
  const action = store.settings.behavior.closeAction
  if (action === 'minimizeToTray') {
    await minimizeToTray()
    return
  }
  if (action === 'exit') {
    await exitApp()
    return
  }
  remember.value = false
  open.value = true
}

async function handleExitRequest() {
  if (store.loading || store.serviceEditorOpen) {
    store.setMessage('info', store.serviceEditorOpen ? '请先完成或取消服务编辑，再退出应用' : '正在处理，请稍候再退出应用')
    return
  }
  if (store.settingsSaveState === 'error' && !await store.saveSettingsOnly()) return
  open.value = false
  await store.runAfterUnsavedProfilesConfirm(async () => { await emit('app-exit-confirmed') })
}

onMounted(() => {
  if (!appWindow) return
  void listen('app-exit-requested', () => { void handleExitRequest() }).then((dispose) => {
    unlistenExit = dispose
  })
  void appWindow.onCloseRequested((event) => {
    event.preventDefault()
    void handleCloseRequest()
  }).then((dispose) => {
    unlisten = dispose
  })
})

onBeforeUnmount(() => {
  unlisten?.()
  unlistenExit?.()
})
</script>

<template>
  <a-modal v-model:open="open" title="关闭 TunnelDesk" :footer="null" :closable="false" width="420px">
    <div class="grid gap-4">
      <p class="m-0 text-sm leading-6 text-[var(--text-secondary)]">你想将 TunnelDesk 最小化到托盘继续运行，还是直接退出应用？</p>
      <a-checkbox v-model:checked="remember">记住我的选择</a-checkbox>
      <div class="flex justify-end gap-2">
        <a-button :disabled="store.loading" @click="choose('minimizeToTray')">最小化到托盘</a-button>
        <a-button :disabled="store.loading" type="primary" danger @click="choose('exit')">退出</a-button>
      </div>
    </div>
  </a-modal>
</template>
