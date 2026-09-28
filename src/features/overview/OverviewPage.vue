<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import {
  FundOutlined,
  PauseCircleOutlined,
  PlayCircleOutlined,
  ReloadOutlined,
  VerticalAlignTopOutlined,
} from '@ant-design/icons-vue'
import PageHeader from '@/shared/ui/PageHeader.vue'
import { useAppStore } from '@/stores/appStore'
import { useUpdateStore } from '@/stores/updateStore'
import ServiceStatusTable from './components/ServiceStatusTable.vue'
import StatusHero from './components/StatusHero.vue'

const store = useAppStore()
const updateStore = useUpdateStore()
const router = useRouter()
const affectedTunnelCount = computed(() => store.status.runningTunnelIds.length)
</script>

<template>
  <PageHeader title="总览" :description="`当前 Profile：${store.currentProfile.name}`">
    <template #actions>
      <a-button
        v-if="updateStore.hasAvailableUpdate"
        size="large"
        type="primary"
        ghost
        :loading="updateStore.installing"
        :disabled="store.loading"
        @click="updateStore.installAvailableUpdate"
      >
        <template #icon><VerticalAlignTopOutlined /></template>
        {{ updateStore.installButtonText }}
      </a-button>
      <a-button :disabled="store.loading" @click="store.refreshRuntimeStatus">
        <template #icon><ReloadOutlined /></template>
        刷新状态
      </a-button>
      <a-button :disabled="store.loading" @click="router.push({ name: 'diagnostics' })">
        <template #icon><FundOutlined /></template>
        诊断
      </a-button>
      <a-button v-if="!store.status.running" type="primary" :disabled="!store.canEditServices || store.serviceEditorOpen" :loading="store.loading" @click="store.start">
        <template #icon><PlayCircleOutlined /></template>
        启动
      </a-button>
      <a-popconfirm
        v-else
        title="停止当前 Profile？"
        :description="`将中断 ${affectedTunnelCount} 条运行隧道。`"
        ok-text="停止"
        cancel-text="取消"
        :ok-button-props="{ danger: true }"
        @confirm="store.stop"
      >
        <a-button danger :loading="store.loading">
          <template #icon><PauseCircleOutlined /></template>
          停止
        </a-button>
      </a-popconfirm>
    </template>
  </PageHeader>
  <div class="grid gap-4">
    <StatusHero />
    <ServiceStatusTable />
  </div>
</template>
