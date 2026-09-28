<script setup lang="ts">
import { BulbFilled, BulbOutlined, DesktopOutlined } from '@ant-design/icons-vue'
import { h } from 'vue'
import { useAppStore } from '@/stores/appStore'
import type { ThemeMode } from '@/shared/types'

const store = useAppStore()

const options = [
  { label: '系统', value: 'system', icon: h(DesktopOutlined) },
  { label: '浅色', value: 'light', icon: h(BulbOutlined) },
  { label: '深色', value: 'dark', icon: h(BulbFilled) },
]

function changeTheme(value: string | number) {
  const mode = String(value) as ThemeMode
  if (store.settingsControlsDisabled || store.settings.behavior.themeMode === mode) return
  store.settings.behavior.themeMode = mode
  void store.saveSettingsOnly('')
}
</script>

<template>
  <a-segmented
    :value="store.settings.behavior.themeMode"
    :options="options"
    :disabled="store.settingsControlsDisabled"
    aria-label="主题"
    @change="changeTheme"
  />
</template>
