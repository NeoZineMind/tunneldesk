<script setup lang="ts">
defineProps<{
  description: string
}>()
</script>

<template>
  <div class="py-10 text-center">
    <a-empty :description="description" />
    <div v-if="$slots.actions" class="mt-4 flex flex-wrap items-center justify-center gap-2">
      <slot name="actions" />
    </div>
  </div>
</template>
