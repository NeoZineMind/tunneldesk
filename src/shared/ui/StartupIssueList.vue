<script setup lang="ts">
import type { StartupIssue } from '@/shared/types'
defineProps<{ issues: StartupIssue[] }>()
</script>

<template>
  <ul class="m-0 grid list-none gap-3 p-0" aria-label="启动问题详情">
    <li v-for="issue in issues" :key="issue.serviceId" class="rounded border border-[var(--line-soft)] p-3 text-sm">
      <div class="font-medium">{{ issue.serviceName }} · <span class="mono">{{ issue.address }}</span></div>
      <div class="mt-1 break-words text-[var(--text-secondary)]">{{ issue.message }}</div>
      <div v-for="owner in issue.owners" :key="`${owner.pid}:${owner.address}`" class="mt-1 break-words">占用进程：{{ owner.processName || '名称未知' }}（PID {{ owner.pid }}），监听 {{ owner.address }}</div>
      <div v-if="!issue.owners.length && ['listenerConflict', 'permissionDenied', 'listenerError'].includes(issue.kind)" class="mt-1 text-[var(--text-muted)]">无法获取占用进程；可能是权限限制、系统保留端口或占用已解除。</div>
    </li>
  </ul>
</template>
