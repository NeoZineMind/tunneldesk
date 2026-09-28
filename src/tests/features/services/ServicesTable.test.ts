import { nextTick } from 'vue'
import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { App as AntApp } from 'ant-design-vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ServicesTable from '@/features/services/components/ServicesTable.vue'
import { defaultServiceDraft } from '@/shared/domain/defaults'
import { useAppStore } from '@/stores/appStore'

vi.mock('@/shared/api/tauri', () => ({
  api: {},
}))

vi.mock('@tauri-apps/plugin-dialog', () => ({
  open: vi.fn(),
  save: vi.fn(),
}))

describe('ServicesTable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.stubGlobal('matchMedia', vi.fn(() => ({
      matches: false,
      media: '',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })))
  })

  it('expands filtered groups and restores collapse state when the filter clears', async () => {
    const store = useAppStore()
    store.initialized = true
    store.statusReady = true
    store.currentProfile.services = [{ ...defaultServiceDraft(), id: 'mysql', name: 'MySQL', group: '数据库' }]
    const wrapper = mount({ props: { filterActive: Boolean }, components: { AntApp, ServicesTable }, template: '<AntApp><ServicesTable :filter-active="filterActive" /></AntApp>' }, { attachTo: document.body })
    await nextTick()
    await wrapper.find('.group-heading').trigger('click')
    expect(wrapper.find('tr[data-row-key="mysql"]').exists()).toBe(false)
    await wrapper.setProps({ filterActive: true })
    expect(wrapper.find('tr[data-row-key="mysql"]').exists()).toBe(true)
    await wrapper.setProps({ filterActive: false })
    expect(wrapper.find('tr[data-row-key="mysql"]').exists()).toBe(false)
    wrapper.unmount()
  })

  it('allows keyboard sorting only while stopped and keeps viewing available when running', async () => {
    const store = useAppStore()
    store.initialized = true
    store.statusReady = true
    store.currentProfile.services = ['a', 'b'].map((id, index) => ({ ...defaultServiceDraft(), id, name: id, sortOrder: (index + 1) * 10 }))
    const wrapper = mount({ components: { AntApp, ServicesTable }, template: '<AntApp><ServicesTable /></AntApp>' }, { attachTo: document.body })
    await nextTick()
    await wrapper.findAll('.service-drag-handle')[1].trigger('keydown', { key: 'ArrowUp' })
    expect(store.orderedCurrentServices.map((item) => item.id)).toEqual(['b', 'a'])
    store.status.running = true
    await nextTick()
    await wrapper.findAll('.service-drag-handle')[0].trigger('keydown', { key: 'ArrowDown' })
    expect(store.orderedCurrentServices.map((item) => item.id)).toEqual(['b', 'a'])
    expect(wrapper.find('[role="switch"]').attributes('disabled')).toBeDefined()
    await wrapper.find('.service-name').trigger('click')
    expect(wrapper.findComponent(ServicesTable).emitted('view')?.[0]).toEqual(['b'])
    wrapper.unmount()
  })

  it('reorders a service after dragging its handle onto another row', async () => {
    const store = useAppStore()
    store.initialized = true
    store.statusReady = true
    const reorderSpy = vi.spyOn(store, 'reorderService')
    store.currentProfile.services = [
      { ...defaultServiceDraft(), id: 'service-a', name: 'Service A', group: '开发', sortOrder: 10 },
      { ...defaultServiceDraft(), id: 'service-b', name: 'Service B', group: '开发', sortOrder: 20 },
    ]

    const wrapper = mount({
      components: { AntApp, ServicesTable },
      template: '<AntApp><ServicesTable /></AntApp>',
    }, { attachTo: globalThis.document.body })
    await nextTick()

    const handles = wrapper.findAll('.service-drag-handle')
    const targetRow = wrapper.find('tr[data-row-key="service-b"]')
    expect(handles).toHaveLength(2)
    expect(targetRow.exists()).toBe(true)

    Object.defineProperties(handles[0].element, {
      setPointerCapture: { configurable: true, value: vi.fn() },
      hasPointerCapture: { configurable: true, value: vi.fn(() => true) },
      releasePointerCapture: { configurable: true, value: vi.fn() },
    })
    Object.defineProperty(globalThis.document, 'elementFromPoint', {
      configurable: true,
      value: vi.fn(() => targetRow.element),
    })

    dispatchPointerEvent(handles[0].element, 'pointerdown', { button: 0, pointerId: 7, clientX: 160, clientY: 10 })
    await nextTick()
    expect(wrapper.find('tr[data-row-key="service-a"]').classes()).toContain('service-row-dragging')

    dispatchPointerEvent(handles[0].element, 'pointermove', { button: 0, pointerId: 7, clientX: 160, clientY: 40 })
    await nextTick()
    expect(targetRow.classes()).toContain('service-row-drop-target')

    dispatchPointerEvent(handles[0].element, 'pointerup', { button: 0, pointerId: 7, clientX: 160, clientY: 40 })
    await nextTick()

    expect(reorderSpy).toHaveBeenCalledWith('service-a', 'service-b', 'after')
    expect(store.orderedCurrentServices.map((service) => service.id)).toEqual(['service-b', 'service-a'])
    wrapper.unmount()
  })
})

function dispatchPointerEvent(element: Element, type: string, values: Record<string, number>) {
  const event = new globalThis.Event(type, { bubbles: true, cancelable: true })
  for (const [key, value] of Object.entries(values)) {
    Object.defineProperty(event, key, { value })
  }
  element.dispatchEvent(event)
}
