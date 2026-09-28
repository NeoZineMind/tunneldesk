<script setup lang="ts">
import { computed, h, ref } from 'vue'
import { DeleteOutlined, EditOutlined, ExclamationCircleOutlined, DownOutlined, RightOutlined, HolderOutlined, MoreOutlined } from '@ant-design/icons-vue'
import { App, type TableColumnsType } from 'ant-design-vue'
import type { ServiceConfig } from '@/shared/types'
import CopyableDomain from '@/shared/ui/CopyableDomain.vue'
import StatusTag from '@/shared/ui/StatusTag.vue'
import { displayServiceStatus } from '@/shared/domain/serviceStatus'
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
const { modal } = App.useApp()
const draggingServiceId = ref<string | null>(null)
const dragOverServiceId = ref<string | null>(null)

interface ServicePointerEvent {
  button: number
  clientX: number
  clientY: number
  currentTarget: HTMLElement | null
  pointerId: number
  preventDefault: () => void
}

interface ServiceKeyboardEvent {
  key: string
  preventDefault: () => void
}

const draggingPointerId = ref<number | null>(null)

const props = defineProps<{
  visibleServiceIds?: string[]
  filterActive?: boolean
}>()

const visibleIdSet = computed(() => new Set(props.visibleServiceIds || store.orderedCurrentServices.map((service) => service.id)))
const visibleGroups = computed(() =>
  store.serviceGroups
    .map((group) => ({ ...group, services: group.services.filter((service) => visibleIdSet.value.has(service.id)) }))
    .filter((group) => group.services.length),
)

defineEmits<{
  view: [serviceId: string]
  edit: [serviceId: string]
}>()

const columns: TableColumnsType<ServiceConfig> = [
  { title: '服务名称', key: 'name', width: '29%' },
  { title: '连接地址', key: 'address', width: '29%' },
  { title: '隧道', key: 'tunnelId' },
  { title: '运行状态', key: 'state', width: 90 },
  { title: '操作', key: 'actions', width: 80, fixed: 'right' },
]

function isExpanded(key: string) {
  return props.filterActive || !(store.collapsedServiceGroups[store.currentProfile.id] || []).includes(key)
}

function statusFor(service: ServiceConfig) {
  return displayServiceStatus(service, store.status.running, store.status.services)
}

function beginPointerDrag(record: ServiceConfig, event: unknown) {
  if (!store.canEditServices) return
  const pointerEvent = event as ServicePointerEvent
  if (pointerEvent.button !== 0 || !pointerEvent.currentTarget) return
  pointerEvent.preventDefault()
  pointerEvent.currentTarget.setPointerCapture(pointerEvent.pointerId)
  draggingServiceId.value = record.id
  dragOverServiceId.value = null
  draggingPointerId.value = pointerEvent.pointerId
}

function updatePointerDrag(event: unknown) {
  const pointerEvent = event as ServicePointerEvent
  if (draggingPointerId.value !== pointerEvent.pointerId || !draggingServiceId.value) return
  pointerEvent.preventDefault()
  const targetId = serviceIdAtPoint(pointerEvent)
  dragOverServiceId.value = targetId && store.canReorderService(draggingServiceId.value, targetId) ? targetId : null
}

function endPointerDrag(event: unknown) {
  const pointerEvent = event as ServicePointerEvent
  if (draggingPointerId.value !== pointerEvent.pointerId) return
  const sourceId = draggingServiceId.value
  const targetId = dragOverServiceId.value || serviceIdAtPoint(pointerEvent)
  if (sourceId && targetId && store.canReorderService(sourceId, targetId)) {
    store.reorderService(sourceId, targetId, dropPlacement(targetId, pointerEvent.clientY))
  }
  if (pointerEvent.currentTarget?.hasPointerCapture(pointerEvent.pointerId)) {
    pointerEvent.currentTarget.releasePointerCapture(pointerEvent.pointerId)
  }
  clearPointerDrag()
}

function cancelPointerDrag(event: unknown) {
  const pointerEvent = event as ServicePointerEvent
  if (draggingPointerId.value !== pointerEvent.pointerId) return
  clearPointerDrag()
}

function clearPointerDrag() {
  draggingServiceId.value = null
  dragOverServiceId.value = null
  draggingPointerId.value = null
}

function serviceIdAtPoint(event: ServicePointerEvent) {
  const element = globalThis.document.elementFromPoint(event.clientX, event.clientY)
  const row = element?.closest<HTMLElement>('tr[data-row-key]')
  return row?.dataset.rowKey || null
}

function moveWithKeyboard(record: ServiceConfig, event: unknown) {
  const keyboardEvent = event as ServiceKeyboardEvent
  if (keyboardEvent.key !== 'ArrowUp' && keyboardEvent.key !== 'ArrowDown') return
  keyboardEvent.preventDefault()
  store.moveService(record.id, keyboardEvent.key === 'ArrowUp' ? -1 : 1)
}

function setEnabled(record: ServiceConfig, checked: boolean) {
  store.updateService(record.id, { ...record, enabled: checked })
}

function handleRowAction(action: string, record: ServiceConfig) {
  if (!store.canEditServices) return
  if (action === 'delete') {
    modal.confirm({
      title: '删除服务？',
      content: `确定删除「${record.name}」？保存服务配置后生效。`,
      icon: h(ExclamationCircleOutlined),
      okText: '删除',
      cancelText: '取消',
      okButtonProps: { danger: true },
      onOk: () => store.removeService(record.id),
    })
  }
}

function rowClassName(record: ServiceConfig) {
  return [
    'service-row',
    draggingServiceId.value === record.id ? 'service-row-dragging' : '',
    dragOverServiceId.value === record.id ? 'service-row-drop-target' : '',
  ].filter(Boolean).join(' ')
}

function dropPlacement(targetId: string, clientY: number) {
  const row = Array.from(globalThis.document.querySelectorAll<HTMLElement>('tr[data-row-key]'))
    .find((element) => element.dataset.rowKey === targetId)
  if (!row) return 'before'
  const rect = row.getBoundingClientRect()
  return clientY > rect.top + rect.height / 2 ? 'after' : 'before'
}
</script>

<template>
  <div class="grid min-w-0 gap-4">
    <section v-for="group in visibleGroups" :key="group.key" class="min-w-0">
      <button type="button" class="group-heading" :aria-expanded="Boolean(isExpanded(group.key))" :disabled="filterActive" @click="store.toggleServiceGroup(group.key)">
        <DownOutlined v-if="isExpanded(group.key)" /><RightOutlined v-else />
        <span class="min-w-0 flex-1 truncate text-left font-medium">{{ group.label }}</span>
        <span class="text-xs text-[var(--text-muted)]">{{ group.services.length }} / {{ store.serviceGroups.find((item) => item.key === group.key)?.services.length }} 个服务</span>
      </button>
      <a-table
        v-if="isExpanded(group.key)" size="small" row-key="id" table-layout="fixed"
        :columns="columns" :data-source="group.services" :pagination="false" :scroll="{ x: 780 }" :row-class-name="rowClassName"
      >
        <template #bodyCell="{ column, record }">
          <template v-if="column.key === 'name'">
            <div class="flex min-w-0 items-center gap-2">
              <a-tooltip :title="store.serviceEditDisabledReason || '拖动排序，方向键调整顺序'">
                <span
                  class="service-drag-handle" role="button" :tabindex="store.canEditServices ? 0 : -1" :aria-disabled="!store.canEditServices"
                  :aria-label="`拖动 ${(record as ServiceConfig).name} 排序`"
                  @pointerdown="beginPointerDrag(record as ServiceConfig, $event)" @pointermove="updatePointerDrag" @pointerup="endPointerDrag" @pointercancel="cancelPointerDrag"
                  @keydown="moveWithKeyboard(record as ServiceConfig, $event)"
                ><HolderOutlined /></span>
              </a-tooltip>
              <a-tooltip :title="store.serviceEditDisabledReason || '配置启用状态，保存配置后生效'">
                <a-switch size="small" :checked="record.enabled" :disabled="!store.canEditServices" :aria-label="`${record.name}配置启用状态`" @change="(checked) => setEnabled(record as ServiceConfig, Boolean(checked))" />
              </a-tooltip>
              <div class="min-w-0">
                <button type="button" class="service-name" :title="record.name" @click="$emit('view', record.id)">{{ record.name }}</button>
                <span class="mono block truncate text-xs text-[var(--text-muted)]" :title="record.localIp">{{ record.localIp }}</span>
              </div>
            </div>
          </template>
          <template v-else-if="column.key === 'address'">
            <CopyableDomain :value="`${record.domain}:${record.port}`" label="连接地址" mono />
          </template>
          <template v-else-if="column.key === 'tunnelId'">
            <span class="block truncate" :title="store.tunnelName(record.tunnelId)">{{ store.tunnelName(record.tunnelId) }}</span>
          </template>
          <template v-else-if="column.key === 'state'">
            <a-tooltip :title="statusFor(record as ServiceConfig).message"><StatusTag :state="statusFor(record as ServiceConfig).state" /></a-tooltip>
          </template>
          <template v-else-if="column.key === 'actions'">
            <div class="flex">
              <a-tooltip :title="store.serviceEditDisabledReason || '编辑'">
                <a-button type="text" size="small" aria-label="编辑服务" :disabled="!store.canEditServices" @click="$emit('edit', record.id)"><template #icon><EditOutlined /></template></a-button>
              </a-tooltip>
              <a-dropdown trigger="click" :disabled="!store.canEditServices">
                <a-button type="text" size="small" aria-label="更多操作" :disabled="!store.canEditServices"><template #icon><MoreOutlined /></template></a-button>
                <template #overlay>
                  <a-menu @click="(event) => handleRowAction(String(event.key), record as ServiceConfig)">
                    <a-menu-item key="delete" danger :disabled="!store.canEditServices"><template #icon><DeleteOutlined /></template>删除</a-menu-item>
                  </a-menu>
                </template>
              </a-dropdown>
            </div>
          </template>
        </template>
      </a-table>
    </section>
  </div>
</template>

<style scoped>
.group-heading { display: flex; width: 100%; align-items: center; gap: 8px; padding: 10px 4px; border: 0; border-bottom: 1px solid var(--line-soft); color: var(--text-primary); background: transparent; cursor: pointer; }
.service-name { display: block; max-width: 100%; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; border: 0; padding: 0; background: transparent; color: var(--text-primary); cursor: pointer; text-align: left; }
.service-name:hover { color: #1677ff; }
.service-drag-handle[aria-disabled="true"] { opacity: .4; cursor: default; }

.service-drag-handle {
  display: inline-flex;
  width: 24px;
  height: 28px;
  flex: 0 0 auto;
  color: var(--text-muted);
  cursor: grab;
  align-items: center;
  justify-content: center;
  border-radius: 5px;
  font-size: 15px;
  touch-action: none;
  user-select: none;
}

.service-drag-handle:hover,
.service-drag-handle:focus-visible {
  color: #2563eb;
  background: rgba(37, 99, 235, 0.08);
  outline: none;
}

.service-drag-handle:active {
  cursor: grabbing;
}

:deep(.service-row) {
  cursor: default;
}

:deep(.service-row-dragging) {
  opacity: 0.45;
}

:deep(.service-row-drop-target td) {
  background: rgba(22, 119, 255, 0.08) !important;
}
</style>
