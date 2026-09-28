<script setup lang="ts">
defineProps<{
  title?: string
  description?: string
}>()
</script>

<template>
  <div class="page-header mb-5 flex flex-col gap-3 sm:flex-row sm:flex-wrap sm:items-center sm:justify-between">
    <div class="min-w-0">
      <h1 v-if="title" class="m-0 truncate text-[26px] font-semibold leading-9 text-[var(--text-primary)]">{{ title }}</h1>
      <p v-if="description" class="m-0 mt-1 max-w-2xl text-sm text-[var(--text-muted)]">{{ description }}</p>
    </div>
    <div v-if="$slots.actions" class="flex min-w-0 flex-wrap items-center justify-start gap-2 sm:justify-end">
      <slot name="actions" />
    </div>
  </div>
</template>
