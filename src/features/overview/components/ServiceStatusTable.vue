<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import type { TableColumnsType } from 'ant-design-vue'
import { ImportOutlined, PlusOutlined } from '@ant-design/icons-vue'
import { useRouter } from 'vue-router'
import StatusTag from '@/shared/ui/StatusTag.vue'
import CopyableDomain from '@/shared/ui/CopyableDomain.vue'
import { displayServiceStatus } from '@/shared/domain/serviceStatus'
import { useAppStore } from '@/stores/appStore'
import type { ServiceConfig } from '@/shared/types'
import EmptyState from '@/shared/ui/EmptyState.vue'
import { overviewServices } from '../overviewState'

const store = useAppStore()
const router = useRouter()
const page = ref(1)
const services = computed(() => overviewServices(store.orderedCurrentServices, store.status))
const hasTunnel = computed(() => store.settings.tunnels.some((tunnel) => tunnel.enabled && tunnel.ssh.host.trim() && tunnel.ssh.username.trim()))
watch(() => store.currentProfile.id, () => { page.value = 1 })
watch(() => services.value.length, (length) => { page.value = Math.min(page.value, Math.max(1, Math.ceil(length / 10))) })
const columns: TableColumnsType<ServiceConfig> = [
  { title: '服务名称', key: 'name', width: '26%' },
  { title: '连接地址', key: 'address', width: '30%' },
  { title: '隧道', key: 'tunnelId' },
  { title: '运行状态', key: 'state', width: 90 },
  { title: '操作', key: 'actions', width: 94, fixed: 'right' },
]
function statusFor(service: ServiceConfig) { return displayServiceStatus(service, store.status.running, store.status.services) }
function hasError(service: ServiceConfig) { return store.status.running && service.enabled && ['error', 'stopped'].includes(statusFor(service).state) }
</script>

<template>
  <section class="min-w-0" aria-label="服务状态">
    <div class="mb-3 flex items-center justify-between gap-2">
      <div class="font-semibold">服务状态 <span class="ml-2 text-xs font-normal text-[var(--text-muted)]">{{ services.length }} 个服务</span></div>
      <a-button type="link" @click="router.push({ name: 'services' })">管理服务</a-button>
    </div>
    <a-table
      v-if="services.length" size="small" row-key="id" table-layout="fixed" :columns="columns" :data-source="services" :scroll="{ x: 780 }"
      :pagination="{ current: page, pageSize: 10, showSizeChanger: false, hideOnSinglePage: true, onChange: (value: number) => page = value }"
    >
      <template #bodyCell="{ column, record }">
        <template v-if="column.key === 'name'">
          <div class="truncate font-medium" :title="record.name">{{ record.name }}</div>
          <div class="mono truncate text-xs text-[var(--text-muted)]" :title="record.localIp">{{ record.localIp }}</div>
          <details v-if="hasError(record as ServiceConfig)" class="service-error">
            <summary :title="statusFor(record as ServiceConfig).message">{{ statusFor(record as ServiceConfig).message }}</summary>
            <p>{{ statusFor(record as ServiceConfig).message }}</p>
          </details>
        </template>
        <template v-else-if="column.key === 'address'"><CopyableDomain :value="`${record.domain}:${record.port}`" label="连接地址" mono /></template>
        <template v-else-if="column.key === 'tunnelId'"><span class="block truncate" :title="store.tunnelName(record.tunnelId)">{{ store.tunnelName(record.tunnelId) }}</span></template>
        <template v-else-if="column.key === 'state'"><StatusTag :state="statusFor(record as ServiceConfig).state" /></template>
        <template v-else-if="column.key === 'actions'"><RouterLink :to="{ name: 'diagnostics', query: { serviceId: record.id } }">查看诊断</RouterLink></template>
      </template>
    </a-table>
    <EmptyState v-else description="当前 Profile 还没有服务配置">
      <template #actions>
        <a-button v-if="!hasTunnel" type="primary" :disabled="!store.canEditServices" @click="router.push({ name: 'tunnels', query: { action: 'create' } })"><template #icon><PlusOutlined /></template>添加隧道</a-button>
        <a-button v-else type="primary" :disabled="!store.canEditServices" @click="router.push({ name: 'services', query: { action: 'create' } })"><template #icon><PlusOutlined /></template>添加服务</a-button>
        <a-button :disabled="!store.canEditServices" @click="router.push({ name: 'services', query: { action: 'import' } })"><template #icon><ImportOutlined /></template>导入配置</a-button>
      </template>
    </EmptyState>
  </section>
</template>

<style scoped>
.service-error { margin-top: 4px; color: var(--danger); font-size: 12px; }
.service-error summary { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; cursor: pointer; }
.service-error p { margin: 4px 0 0; overflow-wrap: anywhere; }
</style>
