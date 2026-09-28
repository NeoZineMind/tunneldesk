<script setup lang="ts">
import PageHeader from '@/shared/ui/PageHeader.vue'
import DiagnosticsPanel from './components/DiagnosticsPanel.vue'
import TunnelStatusList from './components/TunnelStatusList.vue'
import ServiceDiagnosticsList from './components/ServiceDiagnosticsList.vue'
</script>

<template>
  <PageHeader title="诊断" description="检查 hosts 权限、隧道连接和服务状态" />
  <div class="grid grid-cols-1 gap-4">
    <div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
      <DiagnosticsPanel />
      <TunnelStatusList />
    </div>
    <ServiceDiagnosticsList />
  </div>
</template>
