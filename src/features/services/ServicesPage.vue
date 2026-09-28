<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import {
  DeleteOutlined,
  EditOutlined,
  ExportOutlined,
  ImportOutlined,
  MoreOutlined,
  PlusOutlined,
  RollbackOutlined,
  ReloadOutlined,
  SearchOutlined,
} from '@ant-design/icons-vue'
import PageHeader from '@/shared/ui/PageHeader.vue'
import EmptyState from '@/shared/ui/EmptyState.vue'
import { useAppStore } from '@/stores/appStore'
import type { ProfilesImportSession, TunnelMapping } from '@/shared/types'
import ConfigBackupsModal from './components/ConfigBackupsModal.vue'
import ProfileImportPreviewModal from './components/ProfileImportPreviewModal.vue'
import ServiceCreateDrawer from './components/ServiceCreateDrawer.vue'
import ServiceEditDrawer from './components/ServiceEditDrawer.vue'
import ServicesTable from './components/ServicesTable.vue'
import { normalizeRouteAction, omitRouteAction } from '@/shared/domain/routeAction'
import { displayServiceStatus } from '@/shared/domain/serviceStatus'

const store = useAppStore()
const route = useRoute()
const router = useRouter()
const createOpen = ref(false)
const editOpen = ref(false)
const serviceDrawerMode = ref<'view' | 'edit'>('edit')
const editingServiceId = ref<string | null>(null)
const profileCreateOpen = ref(false)
const profileName = ref('')
const profileRenameOpen = ref(false)
const profileRenameName = ref('')
const profileDeleteOpen = ref(false)
const importOpen = ref(false)
const backupsOpen = ref(false)
const reloadOpen = ref(false)
const importSession = ref<ProfilesImportSession | null>(null)
const searchText = ref('')
const groupFilter = ref('all')
const statusFilter = ref<'all' | 'enabled' | 'disabled' | 'abnormal'>('all')

const groupOptions = computed(() => [
  { value: 'all', label: '全部分组' },
  ...store.serviceGroupOptions.map((option) => ({ value: option.value, label: option.value })),
  { value: '__ungrouped', label: '未分组' },
])
const filteredServices = computed(() => {
  const keyword = searchText.value.trim().toLocaleLowerCase()
  return store.orderedCurrentServices.filter((service) => {
    const groupMatches =
      groupFilter.value === 'all' ||
      (groupFilter.value === '__ungrouped' ? !service.group.trim() : service.group === groupFilter.value)
    const status = displayServiceStatus(service, store.status.running, store.status.services)
    const statusMatches =
      statusFilter.value === 'all' ||
      (statusFilter.value === 'enabled' && service.enabled) ||
      (statusFilter.value === 'disabled' && !service.enabled) ||
      (statusFilter.value === 'abnormal' && store.status.running && service.enabled && (status.state === 'error' || status.state === 'stopped'))
    const searchMatches =
      !keyword ||
      `${service.name} ${service.domain} ${service.localIp} ${service.remark}`.toLocaleLowerCase().includes(keyword)
    return groupMatches && statusMatches && searchMatches
  })
})

onMounted(() => {
  void consumeRouteAction()
})

async function consumeRouteAction() {
  const action = normalizeRouteAction(route.query.action, ['import', 'create'] as const)
  if (route.query.action !== undefined) {
    await router.replace({ query: omitRouteAction(route.query) })
  }
  if (action === 'create' && store.canEditServices) createOpen.value = true
  if (action === 'import') {
    await openImportPreview()
  }
}

type ProfileMenuClick = {
  key: string | number
}

function openCreateProfile() {
  profileName.value = `Profile ${store.profiles.profiles.length + 1}`
  profileCreateOpen.value = true
}

async function createProfile() {
  await store.runAfterUnsavedProfilesConfirm(async () => {
    if (await store.createProfile(profileName.value)) profileCreateOpen.value = false
  })
}

function openRenameProfile() {
  profileRenameName.value = store.currentProfile.name
  profileRenameOpen.value = true
}

async function renameProfile() {
  await store.runAfterUnsavedProfilesConfirm(async () => {
    if (await store.renameProfile(store.currentProfile.id, profileRenameName.value)) profileRenameOpen.value = false
  })
}

async function confirmDeleteProfile() {
  await store.runAfterUnsavedProfilesConfirm(async () => {
    if (await store.deleteProfile(store.currentProfile.id)) profileDeleteOpen.value = false
  })
}

function handleProfileAction(action: string) {
  if (action === 'reload') {
    if (store.canEditServices) reloadOpen.value = true
    return
  }
  if (action === 'create') {
    openCreateProfile()
    return
  }
  if (action === 'rename') {
    openRenameProfile()
    return
  }
  if (action === 'delete') {
    profileDeleteOpen.value = true
    return
  }
  if (action === 'import') {
    void openImportPreview()
    return
  }
  if (action === 'backups') {
    backupsOpen.value = true
    void store.refreshConfigBackups()
    return
  }
  if (action === 'export-current') {
    void store.exportCurrentProfile()
    return
  }
  if (action === 'export-all') {
    void store.exportAllProfiles()
  }
}

async function reloadConfig() {
  reloadOpen.value = false
  await store.reload()
}

function handleProfileMenuClick(event: ProfileMenuClick) {
  handleProfileAction(String(event.key))
}

function openViewService(serviceId: string) {
  editingServiceId.value = serviceId
  serviceDrawerMode.value = 'view'
  editOpen.value = true
}

function openEditService(serviceId: string) {
  if (!store.canEditServices) return
  editingServiceId.value = serviceId
  serviceDrawerMode.value = 'edit'
  editOpen.value = true
}

async function openImportPreview() {
  const session = await store.previewProfilesImport()
  if (!session) return
  importSession.value = session
  importOpen.value = true
}

async function updateImportMappings(tunnelMappings: TunnelMapping[]) {
  if (!importSession.value) return
  const session = await store.previewProfilesImport(importSession.value.path, tunnelMappings, true)
  if (session) {
    importSession.value = session
  }
}

async function applyProfilesImport() {
  if (!importSession.value) return
  const session = importSession.value
  await store.runAfterUnsavedProfilesConfirm(async () => {
    const result = await store.applyProfilesImport(session.path, session.tunnelMappings)
    if (!result) return
    importOpen.value = false
    importSession.value = null
  })
}
</script>

<template>
  <PageHeader title="服务" :description="`当前 Profile：${store.currentProfile.name}`">
    <template #actions>
      <div class="flex items-center gap-1">
        <a-dropdown trigger="click">
          <a-button class="w-9 px-0" :loading="store.loading" title="Profile 操作" aria-label="Profile 操作">
            <template #icon><MoreOutlined /></template>
          </a-button>
          <template #overlay>
            <a-menu @click="handleProfileMenuClick">
              <a-menu-item key="create" :disabled="!store.canEditServices">
                <template #icon><PlusOutlined /></template>
                新建
              </a-menu-item>
              <a-menu-item key="rename" :disabled="!store.canEditServices">
                <template #icon><EditOutlined /></template>
                重命名
              </a-menu-item>
              <a-menu-item key="delete" danger :disabled="!store.canEditServices || store.profiles.profiles.length <= 1">
                <template #icon><DeleteOutlined /></template>
                删除
              </a-menu-item>
              <a-menu-divider />
              <a-menu-item key="import" :disabled="!store.canEditServices">
                <template #icon><ImportOutlined /></template>
                导入
              </a-menu-item>
              <a-menu-item key="backups" :disabled="store.loading">
                <template #icon><RollbackOutlined /></template>
                配置备份
              </a-menu-item>
              <a-menu-item key="reload" :disabled="!store.canEditServices">
                <template #icon><ReloadOutlined /></template>
                重新加载配置
              </a-menu-item>
              <a-menu-item key="export-current" :disabled="store.loading">
                <template #icon><ExportOutlined /></template>
                导出当前
              </a-menu-item>
              <a-menu-item key="export-all" :disabled="store.loading">
                <template #icon><ExportOutlined /></template>
                导出全部
              </a-menu-item>
            </a-menu>
          </template>
        </a-dropdown>
      </div>
      <a-button :disabled="!store.canEditServices" @click="createOpen = true">
        <template #icon><PlusOutlined /></template>
        添加服务
      </a-button>
    </template>
  </PageHeader>

  <a-alert v-if="store.serviceEditDisabledReason" class="mb-4" type="info" show-icon :message="store.serviceEditDisabledReason" />
  <a-card :bordered="false" class="surface-card">
    <template #title>
      <div class="card-title">
        <span class="card-title-main">服务映射</span>
        <span class="card-title-meta">{{ store.currentProfile.services.length }} 个服务</span>
      </div>
    </template>
    <div v-if="store.currentProfile.services.length" class="grid gap-4">
      <div class="flex flex-wrap items-center gap-2 rounded-lg bg-[var(--panel-subtle)] p-3">
        <a-input v-model:value="searchText" allow-clear class="flex-1" style="min-width: 220px" placeholder="搜索名称、域名、IP 或备注">
          <template #prefix><SearchOutlined /></template>
        </a-input>
        <a-select v-model:value="groupFilter" class="w-[150px]" :options="groupOptions" />
        <a-select v-model:value="statusFilter" class="w-[140px]">
          <a-select-option value="all">全部状态</a-select-option>
          <a-select-option value="enabled">已启用</a-select-option>
          <a-select-option value="disabled">未启用</a-select-option>
          <a-select-option value="abnormal">仅看异常</a-select-option>
        </a-select>
        <span class="text-xs text-[var(--text-muted)]">{{ filteredServices.length }} / {{ store.currentProfile.services.length }}</span>
      </div>
      <ServicesTable
        v-if="filteredServices.length"
        :visible-service-ids="filteredServices.map((service) => service.id)"
        :filter-active="Boolean(searchText.trim()) || groupFilter !== 'all' || statusFilter !== 'all'"
        @view="openViewService"
        @edit="openEditService"
      />
      <EmptyState v-else description="没有符合当前筛选条件的服务" />
    </div>
    <EmptyState v-else description="暂无服务配置">
      <template #actions>
        <a-button type="primary" :disabled="!store.canEditServices" @click="createOpen = true">
          <template #icon><PlusOutlined /></template>
          添加服务
        </a-button>
        <a-button :disabled="!store.canEditServices" @click="openImportPreview">
          <template #icon><ImportOutlined /></template>
          导入配置
        </a-button>
      </template>
    </EmptyState>
  </a-card>

  <ServiceCreateDrawer v-model:open="createOpen" />
  <ServiceEditDrawer v-model:open="editOpen" :service-id="editingServiceId" :mode="serviceDrawerMode" />
  <ConfigBackupsModal v-model:open="backupsOpen" />
  <a-modal v-model:open="reloadOpen" title="重新加载配置" ok-text="继续" cancel-text="取消" @ok="reloadConfig">
    <p>从磁盘重新读取所有配置，未保存的隧道修改将被替换。服务配置有未保存更改时，会继续询问保存或丢弃。</p>
    <p>如果只想更新运行信息，请使用总览中的“刷新状态”。</p>
  </a-modal>
  <a-modal
    v-model:open="profileCreateOpen"
    title="新建 Profile"
    ok-text="创建"
    cancel-text="取消"
    :confirm-loading="store.loading"
    @ok="createProfile"
  >
    <a-input v-model:value="profileName" placeholder="Profile 名称" @press-enter="createProfile" />
  </a-modal>
  <a-modal
    v-model:open="profileRenameOpen"
    title="重命名 Profile"
    ok-text="保存"
    cancel-text="取消"
    :confirm-loading="store.loading"
    @ok="renameProfile"
  >
    <a-input v-model:value="profileRenameName" placeholder="Profile 名称" @press-enter="renameProfile" />
  </a-modal>
  <a-modal
    v-model:open="profileDeleteOpen"
    title="删除 Profile"
    ok-text="删除"
    cancel-text="取消"
    :ok-button-props="{ danger: true }"
    :confirm-loading="store.loading"
    @ok="confirmDeleteProfile"
  >
    <p class="m-0">
      确定删除当前 Profile「{{ store.currentProfile.name }}」？
    </p>
  </a-modal>
  <ProfileImportPreviewModal
    v-model:open="importOpen"
    :session="importSession"
    @mapping-change="updateImportMappings"
    @apply="applyProfilesImport"
  />
</template>
