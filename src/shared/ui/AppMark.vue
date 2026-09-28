<script setup lang="ts">
withDefaults(
  defineProps<{
    size?: number
    framed?: boolean
  }>(),
  {
    size: 32,
    framed: false,
  },
)
</script>

<template>
  <span
    class="app-mark"
    :class="{ 'app-mark-framed': framed }"
    :style="{ width: `${size}px`, height: `${size}px` }"
    aria-hidden="true"
  >
    <svg viewBox="0 0 64 64" fill="none" xmlns="http://www.w3.org/2000/svg">
      <path d="M15 11h31" class="app-mark-primary" stroke-width="8" stroke-linecap="square" />
      <path d="M30.5 12v27" class="app-mark-primary" stroke-width="8" stroke-linecap="square" />
      <path d="M9 47h40" class="app-mark-line" stroke-width="3.5" stroke-linecap="round" />
      <circle cx="8" cy="47" r="4" class="app-mark-node" stroke-width="3" />
      <circle cx="50" cy="47" r="4" class="app-mark-node" stroke-width="3" />
      <path d="M31 45c4-2 5-13 10-13 3 0 5 5 9 8" class="app-mark-accent" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
      <path d="m47 35 4 5-6 1" class="app-mark-accent" stroke-width="4" stroke-linecap="round" stroke-linejoin="round" />
    </svg>
  </span>
</template>

<style scoped>
.app-mark {
  display: inline-flex;
  flex: 0 0 auto;
  align-items: center;
  justify-content: center;
  color: #0f3b82;
}

.app-mark-framed {
  border-radius: 9px;
  background: linear-gradient(145deg, #1d4ed8, #2563eb);
  box-shadow: 0 4px 12px rgba(37, 99, 235, 0.22);
  padding: 5px;
  color: #fff;
}

.app-mark svg {
  width: 100%;
  height: 100%;
}

.app-mark-primary {
  stroke: currentColor;
}

.app-mark-accent {
  stroke: #20b7e5;
}

.app-mark-line,
.app-mark-node {
  stroke: currentColor;
}

.app-mark-node {
  fill: var(--sidebar-bg);
}

:global(:root[data-theme="dark"]) .app-mark:not(.app-mark-framed) {
  color: #93c5fd;
}
</style>
