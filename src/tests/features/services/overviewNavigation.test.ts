import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { createMemoryHistory, createRouter } from 'vue-router'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ServiceStatusTable from '@/features/overview/components/ServiceStatusTable.vue'
import ServiceDiagnosticsList from '@/features/diagnostics/components/ServiceDiagnosticsList.vue'
import { defaultServiceDraft } from '@/shared/domain/defaults'
import { useAppStore } from '@/stores/appStore'

function makeRouter() {
  return createRouter({ history: createMemoryHistory(), routes: ['overview', 'diagnostics', 'services', 'tunnels'].map((name) => ({ name, path: `/${name}`, component: { template: '<div />' } })) })
}

describe('overview service navigation', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.stubGlobal('matchMedia', vi.fn(() => ({ matches: false, addListener: vi.fn(), removeListener: vi.fn(), addEventListener: vi.fn(), removeEventListener: vi.fn() })))
    useAppStore().currentProfile.services = Array.from({ length: 12 }, (_, index) => ({ ...defaultServiceDraft(), id: `service-${index}`, name: `Service ${index}`, domain: `db-${index}.internal`, sortOrder: index + 1 }))
  })

  it('paginates all services and links diagnostics to the correct service', async () => {
    const router = makeRouter()
    await router.push('/overview')
    const wrapper = mount(ServiceStatusTable, { global: { plugins: [router] } })
    expect(wrapper.findAll('tr[data-row-key]')).toHaveLength(10)
    await wrapper.get('.ant-pagination-item-2').trigger('click')
    expect(wrapper.findAll('tr[data-row-key]')).toHaveLength(2)
    expect(wrapper.get('tr[data-row-key="service-10"] a').attributes('href')).toBe('/diagnostics?serviceId=service-10')
    wrapper.unmount()
  })

  it('focuses a valid diagnostic target and ignores a missing service ID', async () => {
    const router = makeRouter()
    await router.push('/diagnostics?serviceId=service-10')
    const scroll = vi.fn()
    Object.defineProperty(HTMLElement.prototype, 'scrollIntoView', { configurable: true, value: scroll })
    const wrapper = mount(ServiceDiagnosticsList, { attachTo: document.body, global: { plugins: [router] } })
    await flushPromises()
    const target = wrapper.get('[data-service-id="service-10"]')
    expect(target.classes()).toContain('diagnostic-service-selected')
    expect(document.activeElement).toBe(target.element)
    expect(scroll).toHaveBeenCalled()
    await router.push('/diagnostics?serviceId=missing')
    await flushPromises()
    expect(wrapper.find('.diagnostic-service-selected').exists()).toBe(false)
    expect(wrapper.findAll('[data-service-id]')).toHaveLength(12)
    wrapper.unmount()
  })
})
