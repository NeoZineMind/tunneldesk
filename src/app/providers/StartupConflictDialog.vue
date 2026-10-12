<script setup lang="ts">
import { useAppStore } from '@/stores/appStore'
import StartupIssueList from '@/shared/ui/StartupIssueList.vue'
const store = useAppStore()
</script>

<template>
  <a-modal v-model:open="store.startupConflictOpen" title="发现启动冲突" :footer="null" :width="700" :mask-closable="false" :closable="!store.loading" :keyboard="!store.loading">
    <a-alert class="mb-3" type="warning" show-icon message="可以跳过冲突服务，继续启动其他服务" description="跳过只影响本次启动；同域名服务会一起跳过。实际启动会重新检查监听地址。" />
    <div class="max-h-[50vh] overflow-auto"><StartupIssueList :issues="store.startupConflictIssues" /></div>
    <div class="mt-4 flex flex-wrap justify-end gap-2">
      <a-button :disabled="store.loading" @click="store.startupConflictOpen = false">取消</a-button>
      <a-button :disabled="store.loading" @click="store.recheckStartup">重新检查</a-button>
      <a-button type="primary" :loading="store.loading" @click="store.continuePartialStart">跳过冲突并继续</a-button>
    </div>
  </a-modal>
</template>
