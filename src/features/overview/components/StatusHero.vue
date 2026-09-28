<script setup lang="ts">
import { computed } from 'vue'
import { CheckCircleOutlined, PauseCircleOutlined, WarningOutlined } from '@ant-design/icons-vue'
import { useAppStore } from '@/stores/appStore'
import { buildOverviewSummary } from '../overviewState'

const store = useAppStore()
const summary = computed(() => buildOverviewSummary(store.currentProfile, store.settings, store.status))
</script>

<template>
  <section class="status-summary" aria-label="当前环境连接摘要">
    <div class="flex min-w-0 flex-wrap items-center gap-x-3 gap-y-2" role="status">
      <span class="inline-flex items-center gap-2 font-semibold" :style="{ color: summary.tone === 'success' ? 'var(--success)' : summary.tone === 'danger' ? 'var(--danger)' : summary.tone === 'warning' ? 'var(--warning)' : 'var(--text-muted)' }">
        <PauseCircleOutlined v-if="!store.status.running" /><CheckCircleOutlined v-else-if="summary.tone === 'success'" /><WarningOutlined v-else />
        {{ !store.statusReady ? '读取状态中' : summary.title }}
      </span>
      <span v-if="store.statusReady" class="text-sm text-[var(--text-secondary)]">{{ summary.description }}</span>
    </div>
    <div v-if="store.statusReady && summary.issues.length && store.currentProfile.services.length" class="mt-3 flex items-start gap-2 border-t border-[var(--line-soft)] pt-3 text-sm">
      <WarningOutlined class="mt-1 text-[var(--warning)]" />
      <span class="min-w-0 flex-1 break-words text-[var(--text-secondary)]">{{ summary.issues.join('；') }}</span>
      <RouterLink :to="{ name: 'diagnostics' }" class="shrink-0">查看诊断</RouterLink>
    </div>
  </section>
</template>

<style scoped>
.status-summary { padding: 16px 20px; border-radius: 8px; background: var(--panel-bg); border: 1px solid var(--line-soft); }
</style>
