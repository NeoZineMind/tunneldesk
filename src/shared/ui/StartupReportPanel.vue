<script setup lang="ts">
import { computed } from 'vue'
import { useAppStore } from '@/stores/appStore'
import { copyTextToClipboard } from '@/shared/domain/clipboard'
import StartupIssueList from './StartupIssueList.vue'
const store = useAppStore()
const report = computed(() => store.status.startupReport?.profileId === store.currentProfile.id ? store.status.startupReport : null)
async function copyReport() {
  if (!report.value) return
  const copied = await copyTextToClipboard(JSON.stringify(report.value, null, 2))
  store.setMessage(copied ? 'success' : 'error', copied ? '启动诊断已复制' : '复制失败')
}
</script>

<template>
  <a-card v-if="report && report.outcome !== 'complete'" :bordered="false" class="surface-card" title="启动报告">
    <a-alert class="mb-3" :type="report.outcome === 'failed' ? 'error' : 'warning'" show-icon :message="report.message" />
    <div class="mb-3 flex flex-wrap gap-2">
      <a-button :disabled="store.loading" @click="copyReport">复制诊断</a-button>
      <a-button v-if="store.status.running" type="primary" :loading="store.loading" @click="store.retrySkippedServices">重试跳过的服务</a-button>
      <a-button v-else-if="report.issues.length && !report.issues.some(issue => issue.kind === 'hostsError')" type="primary" :disabled="!store.canEditServices" :loading="store.loading" @click="store.continuePartialStart">以容错模式重试</a-button>
      <a-button v-else :disabled="!store.canEditServices" :loading="store.loading" @click="store.start">重新启动</a-button>
    </div>
    <StartupIssueList :issues="report.issues" />
  </a-card>
</template>
