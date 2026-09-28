import { mount } from '@vue/test-utils'
import { reactive } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import ServiceForm from '@/features/services/components/ServiceForm.vue'
import { defaultServiceDraft, defaultSettings } from '@/shared/domain/defaults'

describe('ServiceForm', () => {
  beforeEach(() => {
    vi.stubGlobal('matchMedia', vi.fn(() => ({ matches: false, addListener: vi.fn(), removeListener: vi.fn(), addEventListener: vi.fn(), removeEventListener: vi.fn() })))
  })
  it('keeps labels associated with their own fields when both drawers are mounted', () => {
    const props = { model: defaultServiceDraft(), tunnels: defaultSettings().tunnels, serviceGroupOptions: [] }
    const wrapper = mount({ components: { ServiceForm }, setup: () => ({ props }), template: '<ServiceForm v-bind="props" /><ServiceForm v-bind="props" />' })
    const forms = wrapper.findAll('form')
    const firstId = forms[0].get('input').attributes('id')
    const secondId = forms[1].get('input').attributes('id')
    expect(firstId).not.toBe(secondId)
    expect(forms[0].get('label').attributes('for')).toBe(firstId)
    expect(forms[1].get('label').attributes('for')).toBe(secondId)
    wrapper.unmount()
  })
  it('updates the connection preview and expands advanced settings for an invalid local IP', async () => {
    const model = reactive({ ...defaultServiceDraft(), name: 'MySQL', domain: 'mysql.internal', localIp: 'not-an-ip' })
    const wrapper = mount(ServiceForm, { props: { model, tunnels: defaultSettings().tunnels, serviceGroupOptions: [] }, attachTo: document.body })
    expect(wrapper.get('[aria-label="连接预览"]').text()).toContain('mysql.internal:3306')
    model.domain = 'db.internal'
    model.port = 5432
    await wrapper.vm.$nextTick()
    expect(wrapper.get('[aria-label="连接预览"]').text()).toContain('db.internal:5432')
    vi.stubGlobal('scrollTo', vi.fn())
    await expect(wrapper.vm.validate()).rejects.toBeDefined()
    expect(wrapper.find('.ant-collapse-header').attributes('aria-expanded')).toBe('true')
    wrapper.unmount()
  })
})
