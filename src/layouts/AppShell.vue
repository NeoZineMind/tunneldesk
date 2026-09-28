<script setup lang="ts">
import { ref } from 'vue'
import AppTitleBar from './AppTitleBar.vue'
import SidebarNav from './SidebarNav.vue'
import ProfilesSaveBar from './ProfilesSaveBar.vue'
import { useAppStore } from '@/stores/appStore'

const collapsed = ref(false)
const store = useAppStore()
</script>

<template>
  <div class="grid h-full grid-rows-[34px_minmax(0,1fr)] overflow-hidden bg-[var(--app-bg)] text-[var(--text-primary)]">
    <AppTitleBar />
    <a-layout class="min-h-0 min-w-0 bg-transparent">
      <a-layout-sider
        v-model:collapsed="collapsed"
        breakpoint="lg"
        collapsible
        theme="light"
        width="232"
        :collapsed-width="64"
        class="app-sider"
      >
        <SidebarNav :collapsed="collapsed" />
      </a-layout-sider>
      <a-layout-content class="flex min-h-0 min-w-0 flex-col overflow-hidden bg-transparent">
        <a-alert v-if="store.saveRefreshWarning" type="warning" show-icon closable :message="store.saveRefreshWarning" @close="store.saveRefreshWarning = ''" />
        <div class="app-scrollbar min-h-0 flex-1 overflow-auto px-4 py-5 md:px-6 md:py-6 xl:px-7">
          <RouterView />
        </div>
        <ProfilesSaveBar />
      </a-layout-content>
    </a-layout>
  </div>
</template>
