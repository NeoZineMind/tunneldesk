<script setup lang="ts">
import { computed, ref, type Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  ApartmentOutlined,
  FileTextOutlined,
  FundOutlined,
  HddOutlined,
  HomeOutlined,
  SettingOutlined,
} from '@ant-design/icons-vue'
import type { RouteKey } from '@/app/router/routes'
import AppMark from '@/shared/ui/AppMark.vue'
import ProfileSelector from './ProfileSelector.vue'

const route = useRoute()
const router = useRouter()
const pendingKey = ref<RouteKey | null>(null)

defineProps<{
  collapsed: boolean
}>()

const navItems: { key: RouteKey; icon: Component; label: string }[] = [
  { key: 'overview', icon: HomeOutlined, label: '总览' },
  { key: 'tunnels', icon: ApartmentOutlined, label: '隧道' },
  { key: 'services', icon: HddOutlined, label: '服务' },
  { key: 'diagnostics', icon: FundOutlined, label: '诊断' },
  { key: 'logs', icon: FileTextOutlined, label: '日志' },
  { key: 'settings', icon: SettingOutlined, label: '设置' },
]

const currentKey = computed<RouteKey>(() => {
  const routeName = String(route.name || 'overview')
  return navItems.some((item) => item.key === routeName) ? (routeName as RouteKey) : 'overview'
})

const selectedKey = computed(() => pendingKey.value || currentKey.value)

function go(key: RouteKey) {
  if (selectedKey.value === key) return
  pendingKey.value = key
  void router.push({ name: key }).catch(() => {}).finally(() => {
    if (pendingKey.value === key) pendingKey.value = null
  })
}
</script>

<template>
  <div class="flex h-full flex-col bg-[var(--sidebar-bg)]">
    <div class="flex h-16 shrink-0 items-center" :class="collapsed ? 'justify-center' : 'px-6'">
      <AppMark :size="38" :framed="false" />
    </div>

    <ProfileSelector :collapsed="collapsed" />
    <nav class="app-scrollbar flex min-h-0 flex-1 flex-col overflow-y-auto px-3 py-3">
      <div class="space-y-1">
        <a-tooltip
          v-for="item in navItems.filter((entry) => entry.key !== 'settings')"
          :key="item.key"
          :title="collapsed ? item.label : undefined"
          placement="right"
        >
          <button
            type="button"
            class="sidebar-nav-item"
            :class="[
              selectedKey === item.key ? 'sidebar-nav-item-active' : '',
              collapsed ? 'justify-center px-0' : 'px-3',
            ]"
            @click="go(item.key)"
          >
            <component :is="item.icon" class="sidebar-nav-icon" />
            <span v-if="!collapsed" class="truncate">{{ item.label }}</span>
          </button>
        </a-tooltip>
      </div>

      <div class="mt-auto space-y-2">
        <div class="border-t border-[var(--line-soft)] pt-2">
          <a-tooltip :title="collapsed ? '设置' : undefined" placement="right">
            <button
              type="button"
              class="sidebar-nav-item"
              :class="[
                selectedKey === 'settings' ? 'sidebar-nav-item-active' : '',
                collapsed ? 'justify-center px-0' : 'px-3',
              ]"
              @click="go('settings')"
            >
              <SettingOutlined class="sidebar-nav-icon" />
              <span v-if="!collapsed">设置</span>
            </button>
          </a-tooltip>
        </div>
      </div>
    </nav>
  </div>
</template>
