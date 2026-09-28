<script setup lang="ts">
import { onBeforeUnmount, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import PageHeader from '@/shared/ui/PageHeader.vue'
import { useAppStore } from '@/stores/appStore'
import TunnelAdvancedForm from './components/TunnelAdvancedForm.vue'
import TunnelForm from './components/TunnelForm.vue'
import TunnelList from './components/TunnelList.vue'
import { normalizeRouteAction, omitRouteAction } from '@/shared/domain/routeAction'

const store = useAppStore()
const route = useRoute()
const router = useRouter()
let passwordStateTimer: ReturnType<typeof window.setTimeout> | undefined

onMounted(() => {
  void consumeRouteAction()
  passwordStateTimer = window.setTimeout(() => {
    void store.refreshPasswordState()
  }, 120)
})

async function consumeRouteAction() {
  const action = normalizeRouteAction(route.query.action, ['create'] as const)
  if (route.query.action !== undefined) {
    await router.replace({ query: omitRouteAction(route.query) })
  }
  if (action === 'create') {
    store.addTunnel()
  }
}

onBeforeUnmount(() => {
  if (passwordStateTimer) {
    window.clearTimeout(passwordStateTimer)
  }
})
</script>

<template>
  <PageHeader title="隧道" description="管理 SSH 连接与认证信息" />
  <div class="grid grid-cols-1 gap-4 lg:grid-cols-[280px_minmax(0,1fr)]">
    <TunnelList
      :tunnels="store.settings.tunnels"
      :current-tunnel-id="store.currentTunnel.id"
      :running-tunnel-ids="store.status.runningTunnelIds"
      @select="store.selectTunnel"
      @add="store.addTunnel"
      @remove="store.removeTunnel"
    />
    <div class="grid min-w-0 gap-4">
      <TunnelForm />
      <TunnelAdvancedForm />
    </div>
  </div>
</template>
