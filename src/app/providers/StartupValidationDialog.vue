<script setup lang="ts">
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
</script>

<template>
  <a-modal
    v-model:open="store.startupValidationOpen"
    title="启动前检查失败"
    :footer="null"
  >
    <a-alert class="mb-3" type="error" show-icon message="请修复以下配置问题后再启动" />
    <ul class="m-0 grid gap-2 pl-5 text-sm leading-6 text-[var(--text-secondary)]">
      <li v-for="issue in store.startupValidationIssues" :key="issue.id">
        {{ issue.message }}
      </li>
    </ul>
    <div class="mt-4 flex justify-end">
      <a-button type="primary" @click="store.closeStartupValidation">知道了</a-button>
    </div>
  </a-modal>
</template>
