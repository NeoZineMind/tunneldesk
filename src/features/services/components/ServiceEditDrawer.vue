<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import ServiceForm from './ServiceForm.vue'
import { defaultServiceDraft } from '@/shared/domain/defaults'
import { useAppStore } from '@/stores/appStore'
import type { ServiceConfig } from '@/shared/types'
import { useServiceDrawer } from '../useServiceDrawer'

const props = defineProps<{
  open: boolean
  serviceId?: string | null
  mode?: 'view' | 'edit'
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const store = useAppStore()
const serviceFormRef = ref()
const form = reactive<ServiceConfig>(defaultServiceDraft())
const drawerTitle = computed(() => props.mode === 'view' ? '查看服务' : '编辑服务')
const readonly = computed(() => props.mode === 'view')

const baseline = ref('')
const { drawerOpen, requestClose } = useServiceDrawer(
  () => props.open,
  () => !readonly.value && JSON.stringify(form) !== baseline.value,
  () => emit('update:open', false),
)

const currentService = computed(() => {
  if (!props.serviceId) return undefined
  return store.currentProfile.services.find((service) => service.id === props.serviceId)
})

function resetForm() {
  const service = currentService.value
  Object.assign(form, service ? { ...service } : defaultServiceDraft(store.currentTunnel.id, store.nextServiceLocalIp()))
  baseline.value = JSON.stringify(form)
}

watch(
  [() => props.open, () => props.serviceId],
  ([open]) => {
    if (open) resetForm()
  },
)

async function submit() {
  if (!props.serviceId || !store.canEditServices) return
  try {
    await serviceFormRef.value?.validate()
  } catch {
    return
  }
  if (store.updateService(props.serviceId, { ...form })) {
    emit('update:open', false)
  }
}
</script>

<template>
  <a-drawer v-model:open="drawerOpen" :title="drawerTitle" width="460">
    <ServiceForm
      ref="serviceFormRef"
      :model="form"
      :service-group-options="store.serviceGroupOptions"
      :tunnels="store.settings.tunnels"
      :readonly="readonly || !store.canEditServices"
    />
    <template #footer>
      <p v-if="!readonly" class="mt-0 text-xs text-[var(--text-muted)]">完成编辑后，还需点击“保存配置”写入更改。</p>
      <div class="flex justify-end gap-2">
        <a-button :disabled="store.loading" @click="requestClose">{{ readonly ? '关闭' : '取消' }}</a-button>
        <a-button v-if="!readonly" type="primary" :disabled="!store.canEditServices" @click="submit">完成编辑</a-button>
      </div>
    </template>
  </a-drawer>
</template>
