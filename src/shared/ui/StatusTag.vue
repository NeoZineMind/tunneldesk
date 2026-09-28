<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircleOutlined, CloseCircleOutlined, LoadingOutlined, PauseCircleOutlined, QuestionCircleOutlined } from '@ant-design/icons-vue'
import type { ServiceState } from '@/shared/types'
import { serviceStateColor, serviceStateText } from '@/shared/domain/serviceStatus'

const props = defineProps<{
  state?: ServiceState
  text?: string
}>()

const color = computed(() => serviceStateColor(props.state))
const label = computed(() => props.text || serviceStateText(props.state))
</script>

<template>
  <a-tag :color="color" class="whitespace-nowrap">
    <template #icon><CheckCircleOutlined v-if="state === 'healthy'" /><CloseCircleOutlined v-else-if="state === 'error'" /><LoadingOutlined v-else-if="state === 'checking'" /><PauseCircleOutlined v-else-if="state === 'stopped' || state === 'disabled'" /><QuestionCircleOutlined v-else /></template>
    {{ label }}
  </a-tag>
</template>
