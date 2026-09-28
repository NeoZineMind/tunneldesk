<script setup lang="ts">
import { useAppStore } from '@/stores/appStore'
import { SaveOutlined } from '@ant-design/icons-vue'

const store = useAppStore()
</script>

<template>
  <div v-if="store.profilesDirty" class="profiles-save-bar" role="region" aria-label="服务配置保存">
    <div class="min-w-0 text-sm" role="status">
      <span class="font-medium">有未保存更改</span>
      <span class="ml-2 text-[var(--text-muted)]">{{ store.currentProfile.name }}</span>
    </div>
    <div class="flex shrink-0 gap-2">
      <a-popconfirm title="放弃未保存的服务配置？" ok-text="放弃更改" cancel-text="取消" :disabled="!store.canEditServices || store.serviceEditorOpen" @confirm="store.discardProfilesChanges">
        <a-button :disabled="!store.canEditServices || store.serviceEditorOpen">放弃更改</a-button>
      </a-popconfirm>
      <a-button type="primary" :loading="store.loading" :disabled="!store.canEditServices || store.serviceEditorOpen" @click="store.saveProfiles">
        <template #icon><SaveOutlined /></template>
        保存配置
      </a-button>
    </div>
  </div>
</template>

<style scoped>
.profiles-save-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  flex-wrap: wrap;
  gap: 12px;
  padding: 12px 24px;
  border-top: 1px solid var(--line);
  background: var(--panel-bg);
}
</style>
