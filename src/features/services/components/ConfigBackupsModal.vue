<script setup lang="ts">
import { computed } from 'vue'
import { DeleteOutlined, ReloadOutlined, RollbackOutlined } from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/appStore'
import type { ConfigBackupInfo } from '@/shared/types'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const store = useAppStore()
const modalOpen = computed({
  get: () => props.open,
  set: (value: boolean) => emit('update:open', value),
})

function formatBackupTime(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return date.toLocaleString()
}

async function restoreBackup(backup: ConfigBackupInfo) {
  await store.runAfterUnsavedProfilesConfirm(async () => {
    if (await store.restoreConfigBackup(backup.id)) modalOpen.value = false
  })
}
</script>

<template>
  <a-modal v-model:open="modalOpen" title="配置备份" :footer="null" width="680px" @after-open-change="(open) => open && store.refreshConfigBackups()">
    <div class="grid gap-3">
      <div class="flex justify-end">
        <a-button :loading="store.loading" @click="store.refreshConfigBackups">
          <template #icon><ReloadOutlined /></template>
          刷新
        </a-button>
      </div>
      <a-empty v-if="!store.configBackups.length" description="暂无配置备份" />
      <a-list v-else item-layout="horizontal" :data-source="store.configBackups">
        <template #renderItem="{ item }">
          <a-list-item>
            <template #actions>
              <a-popconfirm :disabled="!store.canEditServices" title="恢复这个备份？当前配置会被替换。" ok-text="恢复" cancel-text="取消" @confirm="restoreBackup(item as ConfigBackupInfo)">
                <a-button type="text" :disabled="!store.canEditServices" :title="store.serviceEditDisabledReason">
                  <template #icon><RollbackOutlined /></template>
                  恢复
                </a-button>
              </a-popconfirm>
              <a-popconfirm title="删除这个备份？" ok-text="删除" cancel-text="取消" @confirm="store.deleteConfigBackup((item as ConfigBackupInfo).id)">
                <a-button danger type="text">
                  <template #icon><DeleteOutlined /></template>
                  删除
                </a-button>
              </a-popconfirm>
            </template>
            <a-list-item-meta :title="formatBackupTime((item as ConfigBackupInfo).createdAt)" :description="(item as ConfigBackupInfo).fileName" />
          </a-list-item>
        </template>
      </a-list>
    </div>
  </a-modal>
</template>
