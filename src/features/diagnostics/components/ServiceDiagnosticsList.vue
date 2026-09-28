<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useRoute } from 'vue-router'
import { displayServiceStatus } from '@/shared/domain/serviceStatus'
import { useAppStore } from '@/stores/appStore'
import StatusTag from '@/shared/ui/StatusTag.vue'

const store = useAppStore()
const route = useRoute()
const listRef = ref<HTMLElement>()
const serviceRows = computed(() => store.orderedCurrentServices.map((service) => ({
  service, status: displayServiceStatus(service, store.status.running, store.status.services),
})))
const selectedId = computed(() => typeof route.query.serviceId === 'string' && serviceRows.value.some((item) => item.service.id === route.query.serviceId) ? route.query.serviceId : '')
watch(selectedId, async (id) => {
  if (!id) return
  await nextTick()
  const target = Array.from(listRef.value?.querySelectorAll<HTMLElement>('[data-service-id]') || []).find((element) => element.dataset.serviceId === id)
  target?.scrollIntoView({ block: 'center' })
  target?.focus({ preventScroll: true })
}, { immediate: true })
</script>

<template>
  <section ref="listRef" class="min-w-0">
    <a-card :bordered="false" class="surface-card">
      <template #title><div class="card-title"><span class="card-title-main">服务状态</span><span class="card-title-meta">{{ serviceRows.length }}</span></div></template>
      <a-empty v-if="!serviceRows.length" description="当前 Profile 暂无服务" />
      <a-list v-else :data-source="serviceRows">
        <template #renderItem="{ item }">
          <a-list-item class="diagnostic-service" :class="{ 'diagnostic-service-selected': selectedId === item.service.id }" :data-service-id="item.service.id" tabindex="-1">
            <a-list-item-meta :title="item.service.name" :description="item.status.message" />
            <StatusTag :state="item.status.state" />
          </a-list-item>
        </template>
      </a-list>
    </a-card>
  </section>
</template>

<style scoped>
.diagnostic-service { gap: 12px; padding-inline: 12px; }
.diagnostic-service-selected { background: var(--nav-active-bg); box-shadow: inset 3px 0 #2563eb; }
:deep(.ant-list-item-meta-description) { overflow-wrap: anywhere; }
</style>
