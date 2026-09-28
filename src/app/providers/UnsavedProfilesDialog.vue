<script setup lang="ts">
import { useAppStore } from '@/stores/appStore'

const store = useAppStore()
</script>

<template>
  <a-modal
    :open="store.unsavedProfilesConfirmOpen"
    title="有未保存的服务配置"
    :closable="!store.loading && !store.continuingProfilesAction"
    :mask-closable="false"
    @cancel="store.cancelUnsavedProfilesAction"
  >
    <p class="m-0 text-sm leading-6 text-[var(--text-secondary)]">
      当前 Profile 有未保存更改。请选择保存配置后继续，或丢弃更改并继续。
    </p>
    <template #footer>
      <a-button :disabled="store.loading || store.continuingProfilesAction" @click="store.cancelUnsavedProfilesAction">取消</a-button>
      <a-button danger :disabled="store.loading || store.continuingProfilesAction" @click="store.confirmUnsavedProfilesAction(false)">丢弃并继续</a-button>
      <a-button type="primary" :loading="store.loading || store.continuingProfilesAction" @click="store.confirmUnsavedProfilesAction(true)">保存并继续</a-button>
    </template>
  </a-modal>
</template>
