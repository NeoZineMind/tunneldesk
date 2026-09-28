<script setup lang="ts">
import { reactive, ref, watch } from 'vue'
import ServiceForm from './ServiceForm.vue'
import { defaultServiceDraft } from '@/shared/domain/defaults'
import { useAppStore } from '@/stores/appStore'
import type { ServiceConfig } from '@/shared/types'
import { useServiceDrawer } from '../useServiceDrawer'

const props = defineProps<{
  open: boolean
}>()

const emit = defineEmits<{
  'update:open': [value: boolean]
}>()

const store = useAppStore()
const serviceFormRef = ref()
const form = reactive<ServiceConfig>(defaultServiceDraft())

const baseline = ref('')
const { drawerOpen, requestClose } = useServiceDrawer(
  () => props.open, () => JSON.stringify(form) !== baseline.value,
  () => emit('update:open', false),
)

function resetForm() {
  Object.assign(form, defaultServiceDraft(store.currentTunnel.id, store.nextServiceLocalIp()))
  baseline.value = JSON.stringify(form)
}

watch(
  () => props.open,
  (value) => {
    if (value) resetForm()
  },
)

async function submit() {
  if (!store.canEditServices) return
  try {
    await serviceFormRef.value?.validate()
  } catch {
    return
  }
  if (store.addService({ ...form })) {
    emit('update:open', false)
  }
}
</script>

<template>
  <a-drawer v-model:open="drawerOpen" title="添加服务" width="460">
    <ServiceForm ref="serviceFormRef" :model="form" :service-group-options="store.serviceGroupOptions" :tunnels="store.settings.tunnels" :readonly="!store.canEditServices" />
    <template #footer>
      <p class="mt-0 text-xs text-[var(--text-muted)]">添加后，还需点击“保存配置”写入更改。</p>
      <div class="flex justify-end gap-2">
        <a-button :disabled="store.loading" @click="requestClose">取消</a-button>
        <a-button type="primary" :disabled="!store.canEditServices" @click="submit">添加到配置</a-button>
      </div>
    </template>
  </a-drawer>
</template>
