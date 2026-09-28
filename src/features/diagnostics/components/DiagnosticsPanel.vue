<script setup lang="ts">
import { computed } from 'vue'
import { ReloadOutlined } from '@ant-design/icons-vue'
import { runningLabel } from '@/shared/domain/serviceStatus'
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
const processLabel = computed(() => {
  if (store.status.privilege.process === 'root') {
    return '已提权'
  }
  if (store.status.privilege.process === 'user') {
    return '普通用户'
  }
  return '未知'
})
const hostsAccessLabel = computed(() => {
  if (store.status.privilege.hostsAccess === 'direct') {
    return '可直接写入'
  }
  if (store.status.privilege.hostsAccess === 'polkit-helper') {
    return '可系统授权'
  }
  return '不可用'
})
const hostsAccessColor = computed(() => (store.status.privilege.canModifyHosts ? 'success' : 'warning'))
</script>

<template>
  <a-card :bordered="false" class="surface-card">
    <template #title>
      <div class="card-title">
        <span class="card-title-main">hosts 与权限</span>
      </div>
    </template>
    <a-alert
      v-if="!store.status.privilege.canModifyHosts"
      class="mb-4"
      type="warning"
      show-icon
      message="当前无法修改 hosts"
      :description="store.status.privilege.message"
    />
    <a-descriptions size="small" :column="1">
      <a-descriptions-item label="当前进程">
        <a-tag :color="store.status.privilege.process === 'root' ? 'success' : 'default'">
          {{ processLabel }}
        </a-tag>
      </a-descriptions-item>
      <a-descriptions-item label="hosts 修改能力">
        <a-tag :color="hostsAccessColor">
          {{ hostsAccessLabel }}
        </a-tag>
      </a-descriptions-item>
      <a-descriptions-item label="Linux 权限组件">
        <a-tag :color="store.status.privilege.helperInstalled ? 'success' : 'default'">
          {{ store.status.privilege.helperInstalled ? '已安装' : '未安装' }}
        </a-tag>
      </a-descriptions-item>
      <a-descriptions-item label="权限说明">
        {{ store.status.privilege.message }}
      </a-descriptions-item>
      <a-descriptions-item label="hosts 标记块">
        <a-tag :color="store.status.hostsBlockPresent ? 'processing' : 'success'">
          {{ store.status.hostsBlockPresent ? '存在' : '不存在' }}
        </a-tag>
      </a-descriptions-item>
      <a-descriptions-item label="运行状态">
        <a-tag :color="store.status.running ? 'success' : 'default'">{{ runningLabel(store.status.running) }}</a-tag>
      </a-descriptions-item>
      <a-descriptions-item label="运行隧道">
        {{ store.status.runningTunnelIds.map(store.tunnelName).join(', ') || '无' }}
      </a-descriptions-item>
    </a-descriptions>
    <div class="mt-4 flex justify-end">
      <a-button :loading="store.loading" @click="store.repairHosts">
        <template #icon><ReloadOutlined /></template>
        修复 hosts
      </a-button>
    </div>
  </a-card>
</template>
