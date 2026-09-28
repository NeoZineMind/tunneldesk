import { flushPromises, shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import SidebarNav from '@/layouts/SidebarNav.vue'
import BehaviorSettingsForm from '@/features/settings/components/BehaviorSettingsForm.vue'

const launchAtLoginEnabled = vi.hoisted(() => vi.fn())
vi.mock('@/shared/api/tauri', () => ({ api: { launchAtLoginEnabled } }))

async function mountSettings() {
  const router = createRouter({ history: createMemoryHistory(), routes: [
    { path: '/settings', name: 'settings', component: BehaviorSettingsForm },
    { path: '/overview', name: 'overview', component: { template: '<h1>总览内容</h1>' } },
  ] })
  await router.push('/settings')
  const wrapper = shallowMount({ components: { SidebarNav }, template: '<SidebarNav :collapsed="false" /><RouterView />' }, {
    global: { plugins: [router], renderStubDefaultSlot: true, stubs: { SidebarNav: false, RouterView: false, BehaviorSettingsForm: false } },
  })
  return { router, wrapper }
}

describe('leaving settings', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    vi.useFakeTimers({ toFake: ['setTimeout', 'clearTimeout', 'requestAnimationFrame', 'cancelAnimationFrame'] })
  })
  afterEach(() => vi.useRealTimers())

  it('navigates immediately and cancels a startup query that has not begun', async () => {
    const { router, wrapper } = await mountSettings()
    const push = vi.spyOn(router, 'push')
    await wrapper.findAll('button').find((button) => button.text() === '总览')!.trigger('click')
    expect(push).toHaveBeenCalledWith({ name: 'overview' })
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('overview')
    await vi.advanceTimersByTimeAsync(120)
    expect(launchAtLoginEnabled).not.toHaveBeenCalled()
    wrapper.unmount()
  })

  it('can leave settings while the startup query is still pending', async () => {
    let finishQuery!: (enabled: boolean) => void
    launchAtLoginEnabled.mockReturnValue(new Promise<boolean>((resolve) => { finishQuery = resolve }))
    const { router, wrapper } = await mountSettings()
    await vi.advanceTimersByTimeAsync(120)
    expect(launchAtLoginEnabled).toHaveBeenCalledTimes(1)
    await wrapper.findAll('button').find((button) => button.text() === '总览')!.trigger('click')
    await flushPromises()
    expect(router.currentRoute.value.name).toBe('overview')
    finishQuery(false)
    await flushPromises()
    wrapper.unmount()
  })
})
