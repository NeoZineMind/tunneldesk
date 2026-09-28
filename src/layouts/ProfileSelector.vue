<script setup lang="ts">
import { computed, ref } from 'vue'
import { FolderOutlined } from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/appStore'

defineProps<{ collapsed: boolean }>()
const store = useAppStore()
const open = ref(false)
const stateLabel = computed(() => !store.statusReady ? '读取状态中' : store.status.running ? '运行中' : '已停止')

async function select(value: unknown) {
  open.value = false
  await store.selectProfile(String(value))
}
</script>

<template>
  <div class="shrink-0 px-3 pb-3">
    <a-popover v-if="collapsed" v-model:open="open" trigger="click" placement="rightTop">
      <template #content>
        <div class="w-[220px]">
          <div class="mb-2 text-xs text-[var(--text-muted)]">环境 / Profile · {{ stateLabel }}</div>
          <a-select aria-label="切换环境 / Profile" :value="store.settings.currentProfileId" class="w-full" :disabled="Boolean(store.profileSwitchDisabledReason)" @change="select">
            <a-select-option v-for="profile in store.profiles.profiles" :key="profile.id" :value="profile.id">{{ profile.name }}</a-select-option>
          </a-select>
          <p v-if="store.profileSwitchDisabledReason" class="mb-0 mt-2 text-xs text-[var(--text-muted)]">{{ store.profileSwitchDisabledReason }}</p>
        </div>
      </template>
      <button class="sidebar-nav-item justify-center" type="button" :title="`${store.currentProfile.name} · ${stateLabel}`" aria-label="环境 / Profile">
        <FolderOutlined />
      </button>
    </a-popover>
    <div v-else class="rounded-lg bg-[var(--panel-subtle)] p-3">
      <div class="mb-2 flex items-center gap-2 text-xs text-[var(--text-muted)]"><FolderOutlined />环境 / Profile</div>
      <a-select aria-label="切换环境 / Profile" :value="store.settings.currentProfileId" class="w-full" :disabled="Boolean(store.profileSwitchDisabledReason)" @change="select">
        <a-select-option v-for="profile in store.profiles.profiles" :key="profile.id" :value="profile.id">{{ profile.name }}</a-select-option>
      </a-select>
      <div class="mt-2 flex items-center gap-1.5 text-xs" :style="{ color: store.status.running ? 'var(--success)' : 'var(--text-muted)' }">
        <span class="h-1.5 w-1.5 rounded-full bg-current" />{{ stateLabel }}
      </div>
      <p v-if="store.profileSwitchDisabledReason" class="mb-0 mt-2 text-xs leading-5 text-[var(--text-muted)]">{{ store.profileSwitchDisabledReason }}</p>
    </div>
  </div>
</template>
