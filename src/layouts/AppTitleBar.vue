<script setup lang="ts">
import { BorderOutlined, CloseOutlined, MinusOutlined } from '@ant-design/icons-vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import AppMark from '@/shared/ui/AppMark.vue'

function hasTauriWindow() {
  const tauriWindow = window as typeof window & { __TAURI_INTERNALS__?: { metadata?: unknown } }
  return Boolean(tauriWindow.__TAURI_INTERNALS__?.metadata)
}

const appWindow = hasTauriWindow() ? getCurrentWindow() : null

function minimize() {
  void appWindow?.hide()
}

function toggleMaximize() {
  void appWindow?.toggleMaximize()
}

function closeWindow() {
  void appWindow?.close()
}
</script>

<template>
  <div
    data-tauri-drag-region
    class="app-titlebar grid h-[34px] select-none grid-cols-[1fr_auto] border-b border-[var(--line-soft)] bg-[var(--titlebar-bg)] text-[var(--text-primary)]"
  >
    <div data-tauri-drag-region class="flex min-w-0 items-center gap-2 px-3">
      <AppMark :size="26" :framed="false" />
      <span class="truncate text-[14px] font-semibold text-[var(--text-primary)]">TunnelDesk</span>
    </div>
    <div class="flex h-full items-stretch">
      <button class="titlebar-button" type="button" aria-label="最小化" @click.stop="minimize" @dblclick.stop>
        <MinusOutlined />
      </button>
      <button class="titlebar-button" type="button" aria-label="最大化" @click.stop="toggleMaximize" @dblclick.stop>
        <BorderOutlined />
      </button>
      <button class="titlebar-button close" type="button" aria-label="关闭" @click.stop="closeWindow" @dblclick.stop>
        <CloseOutlined />
      </button>
    </div>
  </div>
</template>
