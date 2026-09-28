import { defineComponent, ref } from 'vue'
import { flushPromises, mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { App } from 'ant-design-vue'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { useServiceDrawer } from '@/features/services/useServiceDrawer'
import CopyableDomain from '@/shared/ui/CopyableDomain.vue'
import { useAppStore } from '@/stores/appStore'

describe('service editing and copying', () => {
  beforeEach(() => setActivePinia(createPinia()))
  afterEach(() => vi.restoreAllMocks())

  it('keeps local edits until confirmed, blocks duplicate confirmations, and releases the editor lock', async () => {
    const confirm = vi.fn()
    vi.spyOn(App, 'useApp').mockReturnValue({ modal: { confirm } } as unknown as ReturnType<typeof App.useApp>)
    const wrapper = mount(defineComponent({
      setup() {
        const open = ref(false)
        const dirty = ref(false)
        const drawer = useServiceDrawer(() => open.value, () => dirty.value, () => { open.value = false })
        return { open, dirty, ...drawer }
      },
      template: '<div />',
    }))
    wrapper.vm.open = true
    wrapper.vm.dirty = true
    await wrapper.vm.$nextTick()
    expect(useAppStore().serviceEditorOpen).toBe(true)
    wrapper.vm.requestClose()
    wrapper.vm.requestClose()
    expect(confirm).toHaveBeenCalledTimes(1)
    expect(wrapper.vm.open).toBe(true)
    confirm.mock.calls[0][0].onCancel()
    expect(wrapper.vm.open).toBe(true)
    wrapper.vm.requestClose()
    confirm.mock.calls[1][0].onOk()
    await wrapper.vm.$nextTick()
    expect(wrapper.vm.open).toBe(false)
    expect(useAppStore().serviceEditorOpen).toBe(false)
    wrapper.unmount()
  })

  it('copies the complete endpoint and retains the existing domain copy feedback', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    const wrapper = mount(CopyableDomain, { props: { value: 'mysql.internal:3306', label: '连接地址' } })
    await wrapper.get('button').trigger('click')
    await flushPromises()
    expect(writeText).toHaveBeenLastCalledWith('mysql.internal:3306')
    expect(useAppStore().message).toBe('已复制连接地址：mysql.internal:3306')
    await wrapper.setProps({ value: 'mysql.internal', label: '域名' })
    await wrapper.get('button').trigger('click')
    await flushPromises()
    expect(writeText).toHaveBeenLastCalledWith('mysql.internal')
    expect(useAppStore().message).toBe('已复制域名：mysql.internal')
    wrapper.unmount()
    vi.unstubAllGlobals()
  })
})
