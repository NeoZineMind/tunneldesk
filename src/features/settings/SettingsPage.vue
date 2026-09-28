<script setup lang="ts">
import PageHeader from '@/shared/ui/PageHeader.vue'
import BehaviorSettingsForm from './components/BehaviorSettingsForm.vue'
</script>

<template>
  <PageHeader title="设置" description="个性化应用行为、主题与更新选项" />
  <div class="w-full max-w-[1120px]">
    <BehaviorSettingsForm />
  </div>
</template>
