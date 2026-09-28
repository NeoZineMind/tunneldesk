<script setup lang="ts">
import { DeleteOutlined, PlusOutlined } from '@ant-design/icons-vue'
import type { TunnelConfig } from '@/shared/types'

defineProps<{
  tunnels: TunnelConfig[]
  currentTunnelId: string
  runningTunnelIds: string[]
}>()

const emit = defineEmits<{
  select: [tunnelId: string]
  add: []
  remove: [tunnelId: string]
}>()
</script>

<template>
  <a-card :bordered="false" class="surface-card h-fit">
    <template #title>
      <div class="card-title">
        <span class="card-title-main">隧道</span>
        <span class="card-title-meta">{{ tunnels.length }}</span>
      </div>
    </template>
    <template #extra>
      <div class="flex items-center gap-1">
        <a-tooltip title="添加隧道">
          <a-button type="text" shape="circle" @click="emit('add')">
            <template #icon><PlusOutlined /></template>
          </a-button>
        </a-tooltip>
        <a-tooltip title="删除当前隧道">
          <a-button type="text" danger shape="circle" @click="emit('remove', currentTunnelId)">
            <template #icon><DeleteOutlined /></template>
          </a-button>
        </a-tooltip>
      </div>
    </template>

    <div class="grid gap-1.5">
      <button
        v-for="tunnel in tunnels"
        :key="tunnel.id"
        type="button"
        class="w-full rounded-md border px-3 py-2.5 text-left transition"
        :class="
          tunnel.id === currentTunnelId
            ? 'border-[var(--line)] bg-[var(--panel-subtle)] shadow-[inset_0_0_0_1px_var(--nav-active-border)]'
            : 'border-transparent hover:bg-[var(--panel-subtle)]'
        "
        @click="emit('select', tunnel.id)"
      >
        <div class="flex items-start justify-between gap-3">
          <div class="min-w-0">
            <div class="truncate text-sm font-medium text-[var(--text-primary)]">{{ tunnel.name }}</div>
            <div class="mt-1 truncate text-xs text-[var(--text-muted)]">
              {{ tunnel.ssh.host || '未配置 Host' }}:{{ tunnel.ssh.port }}
            </div>
          </div>
          <a-tag class="m-0 shrink-0" :color="runningTunnelIds.includes(tunnel.id) ? 'success' : tunnel.enabled ? 'default' : 'warning'">
            {{ runningTunnelIds.includes(tunnel.id) ? '运行' : tunnel.enabled ? '停止' : '禁用' }}
          </a-tag>
        </div>
      </button>
    </div>
  </a-card>
</template>
