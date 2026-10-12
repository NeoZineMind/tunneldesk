import { mount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import StartupConflictDialog from '@/app/providers/StartupConflictDialog.vue'
import StartupReportPanel from '@/shared/ui/StartupReportPanel.vue'
import StartupIssueList from '@/shared/ui/StartupIssueList.vue'
import { useAppStore } from '@/stores/appStore'
import type { StartupIssue } from '@/shared/types'

const issue: StartupIssue = { serviceId: 'mysql', serviceName: 'MySQL', domain: 'mysql.example', address: '127.77.0.10:3306', kind: 'listenerConflict', osError: 10048, message: 'Address occupied', owners: [{ pid: 42, processName: 'mysqld', address: '0.0.0.0:3306' }] }
const stubs = {
  AModal: { template: '<div><slot /></div>' },
  ACard: { template: '<section><slot /></section>' },
  AAlert: { props: ['message'], template: '<p>{{ message }}</p>' },
  AButton: { props: ['disabled'], emits: ['click'], template: '<button :disabled="disabled" @click="$emit(\'click\')"><slot /></button>' },
}

describe('startup diagnostics UI', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    const store = useAppStore()
    store.initialized = true
    store.statusReady = true
  })

  it('shows the conflict address, process and PID and offers skip/recheck/cancel', async () => {
    const store = useAppStore()
    store.startupConflictIssues = [issue]
    store.startupConflictOpen = true
    const skip = vi.spyOn(store, 'continuePartialStart').mockResolvedValue()
    const wrapper = mount(StartupConflictDialog, { global: { stubs } })
    expect(wrapper.text()).toContain('127.77.0.10:3306')
    expect(wrapper.text()).toContain('mysqld（PID 42）')
    expect(wrapper.text()).toContain('0.0.0.0:3306')
    const buttons = wrapper.findAll('button')
    expect(buttons.map(button => button.text())).toEqual(['取消', '重新检查', '跳过冲突并继续'])
    await buttons[2].trigger('click')
    expect(skip).toHaveBeenCalledOnce()
    await buttons[0].trigger('click')
    expect(store.startupConflictOpen).toBe(false)
  })

  it('keeps all-failed diagnostics visible and supports incremental retry while partially running', async () => {
    const store = useAppStore()
    store.status.startupReport = { profileId: 'default', mode: 'strict', outcome: 'failed', startedServiceIds: [], issues: [issue], message: '启动失败，未启动任何服务' }
    const retry = vi.spyOn(store, 'retrySkippedServices').mockResolvedValue()
    const wrapper = mount(StartupReportPanel, { global: { stubs } })
    expect(wrapper.text()).toContain('启动失败，未启动任何服务')
    expect(wrapper.text()).toContain('以容错模式重试')
    store.status.running = true
    store.status.startupReport.outcome = 'partial'
    await wrapper.vm.$nextTick()
    const button = wrapper.findAll('button').find(button => button.text() === '重试跳过的服务')!
    await button.trigger('click')
    expect(retry).toHaveBeenCalledOnce()
  })

  it('explains unavailable owner information without hiding the original error', () => {
    const wrapper = mount(StartupIssueList, { props: { issues: [{ ...issue, owners: [] }] } })
    expect(wrapper.text()).toContain('Address occupied')
    expect(wrapper.text()).toContain('无法获取占用进程')
  })
})
