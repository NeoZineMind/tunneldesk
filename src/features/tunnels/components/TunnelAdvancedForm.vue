<script setup lang="ts">
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
</script>

<template>
  <a-collapse class="surface-card" ghost>
    <a-collapse-panel key="advanced" header="高级设置">
      <p class="mt-0 text-sm text-[var(--text-muted)]">调整 SSH 保活策略。通常无需修改默认值。</p>
      <a-form :model="store.currentTunnel" layout="vertical">
        <div class="grid grid-cols-1 gap-x-4 md:grid-cols-2">
          <a-form-item label="保活间隔（秒）" tooltip="ServerAliveInterval：发送 SSH 保活请求的间隔">
            <a-input-number v-model:value="store.currentTunnel.ssh.serverAliveInterval" class="w-full" :min="5" />
          </a-form-item>
          <a-form-item label="最大未响应次数" tooltip="ServerAliveCountMax：连续未收到保活响应的最大次数">
            <a-input-number v-model:value="store.currentTunnel.ssh.serverAliveCountMax" class="w-full" :min="1" />
          </a-form-item>
        </div>
      </a-form>
    </a-collapse-panel>
  </a-collapse>
</template>
