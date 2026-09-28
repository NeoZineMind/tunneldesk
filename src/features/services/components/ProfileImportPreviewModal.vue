<script setup lang="ts">
import { computed, reactive, watch } from 'vue'
import type { TableColumnsType } from 'ant-design-vue'
import { useAppStore } from '@/stores/appStore'
import type { ProfilesImportSession, TunnelMapping } from '@/shared/types'

const props = defineProps<{
  open: boolean
  session: ProfilesImportSession | null
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
  'mapping-change': [value: TunnelMapping[]]
  apply: []
}>()

const store = useAppStore()
const mappingValues = reactive<Record<string, string>>({})

const modalOpen = computed({
  get: () => props.open,
  set: (value: boolean) => emit('update:open', value),
})

const overwriteColumns: TableColumnsType = [
  { title: '服务', dataIndex: 'service', key: 'service', width: 170 },
  { title: '原配置', dataIndex: 'oldConfig', key: 'oldConfig' },
  { title: '新配置', dataIndex: 'newConfig', key: 'newConfig' },
]

const conflictColumns: TableColumnsType = [
  { title: '服务', dataIndex: 'service', key: 'service', width: 170 },
  { title: '监听地址', dataIndex: 'listener', key: 'listener', width: 160 },
  { title: '已占用服务', dataIndex: 'existing', key: 'existing' },
]

const overwriteRows = computed(() =>
  (props.session?.preview.overwrites || []).map((item) => ({
    key: `${item.profileId}:${item.serviceId}`,
    service: `${item.profileName} / ${item.newName}`,
    oldConfig: formatConfig(item.oldGroup, item.oldDomain, item.oldPort, item.oldLocalIp, item.oldTunnelId, item.oldSortOrder),
    newConfig: formatConfig(item.newGroup, item.newDomain, item.newPort, item.newLocalIp, item.newTunnelId, item.newSortOrder),
  })),
)

const conflictRows = computed(() =>
  (props.session?.preview.conflicts || []).map((item) => ({
    key: `${item.profileId}:${item.serviceId}`,
    service: `${item.profileName} / ${item.serviceName}`,
    listener: `${item.localIp}:${item.port}`,
    existing: `${item.existingServiceName} (${item.existingServiceId})`,
  })),
)

watch(
  () => props.session,
  (session) => {
    for (const key of Object.keys(mappingValues)) {
      delete mappingValues[key]
    }
    for (const mapping of session?.tunnelMappings || []) {
      mappingValues[mapping.sourceTunnelId] = mapping.targetTunnelId
    }
    for (const missing of session?.preview.missingTunnels || []) {
      mappingValues[missing.sourceTunnelId] ||= ''
    }
  },
  { immediate: true },
)

function formatGroup(group: string) {
  return group?.trim() || '未分组'
}

function formatConfig(group: string, domain: string, port: number, localIp: string, tunnelId: string, sortOrder: number) {
  return `${formatGroup(group)} #${sortOrder || '-'} / ${domain}:${port} -> ${localIp}:${port} / ${store.tunnelName(tunnelId)}`
}

function mappingList(): TunnelMapping[] {
  return Object.entries(mappingValues)
    .filter(([, targetTunnelId]) => Boolean(targetTunnelId))
    .map(([sourceTunnelId, targetTunnelId]) => ({ sourceTunnelId, targetTunnelId }))
}

function changeMapping(sourceTunnelId: string, targetTunnelId: string) {
  mappingValues[sourceTunnelId] = targetTunnelId
  emit('mapping-change', mappingList())
}
</script>

<template>
  <a-modal
    v-model:open="modalOpen"
    title="导入预览"
    width="780px"
    ok-text="导入"
    cancel-text="取消"
    :confirm-loading="store.loading"
    :ok-button-props="{ disabled: !session?.preview.canApply || !store.canEditServices }"
    @ok="emit('apply')"
  >
    <div v-if="session" class="grid gap-4">
      <div class="grid grid-cols-2 gap-3 md:grid-cols-5">
        <div class="rounded-md border border-[var(--line-soft)] p-3">
          <div class="text-xs text-[var(--text-muted)]">Profile</div>
          <div class="mt-1 text-lg font-semibold text-[var(--text-primary)]">{{ session.preview.profileCount }}</div>
        </div>
        <div class="rounded-md border border-[var(--line-soft)] p-3">
          <div class="text-xs text-[var(--text-muted)]">服务</div>
          <div class="mt-1 text-lg font-semibold text-[var(--text-primary)]">{{ session.preview.serviceCount }}</div>
        </div>
        <div class="rounded-md border border-[var(--line-soft)] p-3">
          <div class="text-xs text-[var(--text-muted)]">新增</div>
          <div class="mt-1 text-lg font-semibold text-[var(--text-primary)]">
            {{ session.preview.addedProfileCount }} / {{ session.preview.addedServiceCount }}
          </div>
        </div>
        <div class="rounded-md border border-[var(--line-soft)] p-3">
          <div class="text-xs text-[var(--text-muted)]">覆盖</div>
          <div class="mt-1 text-lg font-semibold text-[var(--text-primary)]">{{ session.preview.updatedServiceCount }}</div>
        </div>
        <div class="rounded-md border border-[var(--line-soft)] p-3">
          <div class="text-xs text-[var(--text-muted)]">跳过</div>
          <div class="mt-1 text-lg font-semibold text-[var(--text-primary)]">{{ session.preview.skippedServiceCount }}</div>
        </div>
      </div>

      <section v-if="session.preview.missingTunnels.length" class="rounded-md border border-amber-200 bg-amber-50 p-3">
        <div class="mb-3 text-sm font-medium text-amber-900">映射缺失隧道</div>
        <div class="grid gap-2">
          <div
            v-for="item in session.preview.missingTunnels"
            :key="item.sourceTunnelId"
            class="grid gap-2 md:grid-cols-[minmax(0,1fr)_260px] md:items-center"
          >
            <div class="min-w-0 text-sm text-amber-950">
              <span class="font-medium">{{ item.sourceTunnelId }}</span>
              <span class="ml-2 text-amber-800">{{ item.serviceCount }} 个服务</span>
            </div>
            <a-select
              :value="mappingValues[item.sourceTunnelId]"
              placeholder="选择本机隧道"
              class="w-full"
              @change="(value) => changeMapping(item.sourceTunnelId, String(value))"
            >
              <a-select-option v-for="tunnel in store.settings.tunnels" :key="tunnel.id" :value="tunnel.id">
                {{ tunnel.name }}
              </a-select-option>
            </a-select>
          </div>
        </div>
      </section>

      <section v-if="overwriteRows.length" class="grid gap-2">
        <div class="text-sm font-medium text-[var(--text-primary)]">将覆盖的服务</div>
        <a-table size="small" row-key="key" :columns="overwriteColumns" :data-source="overwriteRows" :pagination="false" />
      </section>

      <section v-if="conflictRows.length" class="grid gap-2">
        <div class="text-sm font-medium text-[var(--text-primary)]">将跳过的冲突服务</div>
        <a-table size="small" row-key="key" :columns="conflictColumns" :data-source="conflictRows" :pagination="false" />
      </section>

      <a-empty
        v-if="!session.preview.missingTunnels.length && !overwriteRows.length && !conflictRows.length"
        description="没有冲突，可以导入"
      />
    </div>
  </a-modal>
</template>
