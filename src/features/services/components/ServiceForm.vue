<script setup lang="ts">
import { computed, nextTick, ref, useId } from 'vue'
import type { FormInstance } from 'ant-design-vue'
import { domainRule, loopbackIpRule, portRule, requiredRule } from '@/shared/domain/validators'
import type { ServiceConfig, TunnelConfig } from '@/shared/types'

const props = withDefaults(defineProps<{
  model: ServiceConfig
  serviceGroupOptions: { value: string }[]
  tunnels: TunnelConfig[]
  readonly?: boolean
}>(), { readonly: false })

const formRef = ref<FormInstance>()
const formName = `service-${useId()}`
const advancedKeys = ref<string[]>([])
const preview = computed(() => {
  const tunnel = props.tunnels.find((item) => item.id === props.model.tunnelId)
  if (!tunnel || !props.model.domain.trim() || !props.model.port) return '填写域名、端口并选择隧道后，将在此显示连接预览。'
  return `通过${tunnel.name}访问 ${props.model.domain.trim()}:${props.model.port}`
})

async function validate() {
  try {
    await formRef.value?.validate()
  } catch (error) {
    const fields = (error as { errorFields?: { name: string[] }[] }).errorFields || []
    if (fields.some((field) => field.name.includes('localIp'))) advancedKeys.value = ['local']
    await nextTick()
    const first = fields[0]?.name
    if (first) formRef.value?.scrollToField(first, { block: 'center' })
    throw error
  }
}

defineExpose({ validate })
</script>

<template>
  <a-form ref="formRef" :name="formName" :model="model" layout="vertical" :disabled="readonly">
    <a-form-item label="服务名称" name="name" :rules="[requiredRule('请填写服务名')]"><a-input v-model:value="model.name" /></a-form-item>
    <div class="service-endpoint-fields">
      <a-form-item label="域名" name="domain" :rules="[domainRule()]"><a-input v-model:value="model.domain" placeholder="mysql.internal" /></a-form-item>
      <a-form-item label="端口" name="port" :rules="[portRule()]"><a-input-number v-model:value="model.port" class="w-full" :min="1" :max="65535" /></a-form-item>
    </div>
    <a-form-item label="隧道" name="tunnelId" :rules="[requiredRule('请选择隧道')]">
      <a-select v-model:value="model.tunnelId"><a-select-option v-for="tunnel in tunnels" :key="tunnel.id" :value="tunnel.id">{{ tunnel.name }}</a-select-option></a-select>
    </a-form-item>
    <a-form-item label="分组" name="group"><a-auto-complete v-model:value="model.group" class="w-full" :options="serviceGroupOptions" placeholder="未分组" /></a-form-item>
    <a-form-item label="备注" name="remark"><a-textarea v-model:value="model.remark" :auto-size="{ minRows: 2, maxRows: 4 }" /></a-form-item>
    <a-form-item label="配置启用" name="enabled" extra="保存配置后，下次启动时是否包含此服务。"><a-switch v-model:checked="model.enabled" /></a-form-item>
    <a-collapse v-model:active-key="advancedKeys" ghost class="mb-4">
      <a-collapse-panel key="local" header="高级设置" force-render>
        <a-form-item label="本地 IP" name="localIp" :rules="[loopbackIpRule()]" extra="已自动分配回环地址，通常无需修改。"><a-input v-model:value="model.localIp" /></a-form-item>
      </a-collapse-panel>
    </a-collapse>
    <div class="rounded-lg bg-[var(--panel-subtle)] p-3" aria-label="连接预览">
      <div class="mb-1 text-xs text-[var(--text-muted)]">连接预览</div>
      <p class="m-0 break-words text-sm text-[var(--text-secondary)]">{{ preview }}</p>
    </div>
  </a-form>
</template>

<style scoped>
.service-endpoint-fields { display: grid; grid-template-columns: minmax(0, 1fr) 112px; gap: 12px; }
@media (max-width: 480px) { .service-endpoint-fields { grid-template-columns: 1fr; gap: 0; } }
</style>
