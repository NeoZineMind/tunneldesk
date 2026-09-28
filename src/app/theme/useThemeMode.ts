import { computed, onBeforeUnmount, onMounted, ref, type MaybeRefOrGetter, toValue } from 'vue'
import type { ThemeMode } from '@/shared/types'

export function useThemeMode(mode: MaybeRefOrGetter<ThemeMode>) {
  const prefersDark = ref(false)
  let mediaQuery: MediaQueryList | null = null
  let listener: ((event: MediaQueryListEvent) => void) | null = null

  onMounted(() => {
    mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
    prefersDark.value = mediaQuery.matches
    listener = (event: MediaQueryListEvent) => {
      prefersDark.value = event.matches
    }
    mediaQuery.addEventListener('change', listener)
  })

  onBeforeUnmount(() => {
    if (mediaQuery && listener) {
      mediaQuery.removeEventListener('change', listener)
    }
  })

  const effectiveTheme = computed(() => {
    const value = toValue(mode)
    if (value === 'system') return prefersDark.value ? 'dark' : 'light'
    return value
  })

  return { effectiveTheme }
}
