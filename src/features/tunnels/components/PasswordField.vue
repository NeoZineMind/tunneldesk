<script setup lang="ts">
import { DeleteOutlined } from '@ant-design/icons-vue'

defineProps<{
  modelValue: string
  hasSavedPassword: boolean
  loading: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
  clear: []
}>()
</script>

<template>
  <div class="grid gap-2">
    <a-input-password
      :value="modelValue"
      autocomplete="new-password"
      placeholder="留空则保留已保存密码"
      @update:value="emit('update:modelValue', String($event))"
    />
    <div class="flex flex-wrap items-center justify-between gap-2">
      <a-tag :color="hasSavedPassword ? 'success' : 'default'">
        {{ hasSavedPassword ? '已保存到系统凭据库' : '未保存密码' }}
      </a-tag>
      <a-button v-if="hasSavedPassword" danger size="small" :loading="loading" @click="emit('clear')">
        <template #icon><DeleteOutlined /></template>
        清除密码
      </a-button>
    </div>
  </div>
</template>
