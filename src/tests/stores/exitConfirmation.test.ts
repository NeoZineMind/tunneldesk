import { flushPromises, shallowMount } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import CloseRequestDialog from '@/app/providers/CloseRequestDialog.vue'
import { useAppStore } from '@/stores/appStore'
import { defaultProfiles, defaultServiceDraft, defaultSettings, defaultStatus } from '@/shared/domain/defaults'

const mocks = vi.hoisted(() => ({
  handlers: new Map<string, () => void>(), emit: vi.fn(), exit: vi.fn(), dispose: vi.fn(),
  saveProfiles: vi.fn(), saveSettings: vi.fn(), loadSettings: vi.fn(), loadProfiles: vi.fn(), getStatus: vi.fn(),
}))
vi.mock('@tauri-apps/api/event', () => ({ emit: mocks.emit, listen: vi.fn(async (name: string, callback: () => void) => { mocks.handlers.set(name, callback); return mocks.dispose }) }))
vi.mock('@tauri-apps/api/window', () => ({ getCurrentWindow: () => ({ onCloseRequested: async () => mocks.dispose }) }))
vi.mock('@tauri-apps/plugin-process', () => ({ exit: mocks.exit }))
vi.mock('@/shared/api/tauri', () => ({ api: mocks }))

describe('tray exit confirmation', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mocks.handlers.clear()
    vi.stubGlobal('__TAURI_INTERNALS__', { metadata: {} })
    setActivePinia(createPinia())
    mocks.loadSettings.mockResolvedValue(defaultSettings())
    mocks.loadProfiles.mockResolvedValue(defaultProfiles())
    mocks.getStatus.mockResolvedValue(defaultStatus())
    mocks.saveProfiles.mockImplementation(async (profiles) => profiles)
    mocks.saveSettings.mockImplementation(async (settings) => settings)
  })
  afterEach(() => vi.unstubAllGlobals())

  it('retains drafts on cancel and only confirms native exit after a successful save', async () => {
    const store = useAppStore()
    await store.bootstrap()
    await flushPromises()
    store.addService({ ...defaultServiceDraft(), name: 'MySQL', domain: 'mysql.internal' })
    const wrapper = shallowMount(CloseRequestDialog)
    await flushPromises()
    mocks.handlers.get('app-exit-requested')?.()
    await flushPromises()
    expect(store.unsavedProfilesConfirmOpen).toBe(true)
    expect(mocks.emit).not.toHaveBeenCalled()
    store.cancelUnsavedProfilesAction()
    expect(store.profilesDirty).toBe(true)
    mocks.handlers.get('app-exit-requested')?.()
    await flushPromises()
    mocks.saveProfiles.mockRejectedValueOnce(new Error('disk full'))
    await store.confirmUnsavedProfilesAction(true)
    expect(mocks.emit).not.toHaveBeenCalled()
    expect(store.profilesDirty).toBe(true)
    await store.confirmUnsavedProfilesAction(true)
    expect(mocks.emit).toHaveBeenCalledWith('app-exit-confirmed')
    expect(store.profilesDirty).toBe(false)
    wrapper.unmount()
    expect(mocks.dispose).toHaveBeenCalledTimes(2)
  })

  it('retains failed settings on exit and only exits once retry succeeds', async () => {
    const store = useAppStore()
    await store.bootstrap()
    await flushPromises()
    store.settings.behavior.themeMode = 'dark'
    mocks.saveSettings.mockRejectedValueOnce(new Error('disk full')).mockRejectedValueOnce(new Error('still full'))
    expect(await store.saveSettingsOnly()).toBe(false)
    const wrapper = shallowMount(CloseRequestDialog)
    await flushPromises()
    mocks.handlers.get('app-exit-requested')?.()
    await flushPromises()
    expect(mocks.emit).not.toHaveBeenCalled()
    expect(store.settings.behavior.themeMode).toBe('dark')
    mocks.handlers.get('app-exit-requested')?.()
    await flushPromises()
    expect(mocks.emit).toHaveBeenCalledWith('app-exit-confirmed')
    wrapper.unmount()
  })

  it('blocks tray exit while local form edits are open', async () => {
    const store = useAppStore()
    store.serviceEditorOpen = true
    const wrapper = shallowMount(CloseRequestDialog)
    await flushPromises()
    mocks.handlers.get('app-exit-requested')?.()
    await flushPromises()
    expect(mocks.emit).not.toHaveBeenCalled()
    expect(mocks.exit).not.toHaveBeenCalled()
    expect(store.message).toContain('请先完成或取消服务编辑')
    wrapper.unmount()
  })
})
