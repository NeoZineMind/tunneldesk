<script setup lang="ts">
import { computed } from 'vue'
import { CopyOutlined } from '@ant-design/icons-vue'
import { copyTextToClipboard } from '@/shared/domain/clipboard'
import { useAppStore } from '@/stores/appStore'

const props = withDefaults(defineProps<{
  value: string
  mono?: boolean
  copyValue?: string
  label?: string
}>(), {
  mono: false,
  label: '域名',
  copyValue: undefined,
})

const store = useAppStore()
const title = computed(() => (props.value ? `${props.value}（点击复制${props.label}）` : `无${props.label}`))

async function copyDomain() {
  const value = props.copyValue ?? props.value
  const copied = await copyTextToClipboard(value)
  if (copied) {
    store.setMessage('success', `已复制${props.label}：${value}`)
    return
  }
  store.setMessage('error', `复制${props.label}失败`)
}
</script>

<template>
  <button
    type="button"
    class="copyable-domain"
    :class="{ mono }"
    :title="title"
    :disabled="!value"
    @click.stop="copyDomain"
  >
    <span class="copyable-domain-text">{{ value || '-' }}</span>
    <CopyOutlined class="copyable-domain-icon" />
  </button>
</template>

<style scoped>
.copyable-domain {
  display: inline-flex;
  max-width: 100%;
  min-width: 0;
  align-items: center;
  gap: 6px;
  border: 0;
  background: transparent;
  color: var(--text-primary);
  cursor: pointer;
  padding: 0;
  text-align: left;
  vertical-align: middle;
}

.copyable-domain:hover {
  color: #1677ff;
}

.copyable-domain:disabled {
  color: var(--text-muted);
  cursor: default;
}

.copyable-domain-text {
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.copyable-domain-icon {
  flex: 0 0 auto;
  font-size: 12px;
  opacity: 0.65;
}
</style>
