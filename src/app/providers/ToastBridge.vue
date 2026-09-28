<script setup lang="ts">
import { watch } from 'vue'
import { App } from 'ant-design-vue'
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
const { message } = App.useApp()

watch(
  () => store.message,
  (content) => {
    if (!content) return
    message[store.messageType](content, 3)
    store.clearMessage()
  },
)
</script>

<template>
  <span hidden />
</template>
