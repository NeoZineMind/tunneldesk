<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, watch } from 'vue'
import { setTheme } from '@tauri-apps/api/app'
import zhCN from 'ant-design-vue/es/locale/zh_CN'
import CloseRequestDialog from './CloseRequestDialog.vue'
import StartupValidationDialog from './StartupValidationDialog.vue'
import ToastBridge from './ToastBridge.vue'
import UnsavedProfilesDialog from './UnsavedProfilesDialog.vue'
import { useAppStore } from '@/stores/appStore'
import { useUpdateStore } from '@/stores/updateStore'
import { useAntdTheme } from '@/app/theme/antdTheme'
import { useThemeMode } from '@/app/theme/useThemeMode'

const store = useAppStore()
const updateStore = useUpdateStore()
const themeMode = computed(() => store.settings.behavior.themeMode)
const { effectiveTheme } = useThemeMode(themeMode)
const themeConfig = useAntdTheme(effectiveTheme)
let updateCheckTimer: ReturnType<typeof window.setTimeout> | undefined

function preventContextMenu(event: { preventDefault: () => void }) {
  event.preventDefault()
}

watch(
  effectiveTheme,
  (value) => {
    document.documentElement.dataset.theme = value
    document.documentElement.classList.toggle('dark', value === 'dark')
    void setTheme(value).catch(() => {})
  },
  { immediate: true },
)

onMounted(() => {
  document.addEventListener('contextmenu', preventContextMenu, true)
  void store.bootstrap().then(() => {
    updateCheckTimer = window.setTimeout(() => {
      void updateStore.checkForUpdates({ silent: true })
    }, 8000)
  })
})

onBeforeUnmount(() => {
  document.removeEventListener('contextmenu', preventContextMenu, true)
  if (updateCheckTimer) {
    window.clearTimeout(updateCheckTimer)
  }
})
</script>

<template>
  <a-config-provider :locale="zhCN" :theme="themeConfig" component-size="middle">
    <a-app class="h-full">
      <slot />
      <CloseRequestDialog />
      <StartupValidationDialog />
      <UnsavedProfilesDialog />
      <ToastBridge />
    </a-app>
  </a-config-provider>
</template>
