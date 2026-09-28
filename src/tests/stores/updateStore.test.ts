import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { useAppStore } from '@/stores/appStore'
import { useUpdateStore } from '@/stores/updateStore'

const updaterMock = vi.hoisted(() => ({
  check: vi.fn(),
}))

const processMock = vi.hoisted(() => ({
  relaunch: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-updater', () => updaterMock)
vi.mock('@tauri-apps/plugin-process', () => processMock)

describe('updateStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    updaterMock.check.mockResolvedValue(null)
    processMock.relaunch.mockResolvedValue(undefined)
  })

  it('keeps silent startup checks quiet when no update is available', async () => {
    const appStore = useAppStore()
    const updateStore = useUpdateStore()

    const update = await updateStore.checkForUpdates({ silent: true })

    expect(update).toBeNull()
    expect(updateStore.hasAvailableUpdate).toBe(false)
    expect(appStore.message).toBe('')
  })

  it('stores an available update and reports it on manual checks', async () => {
    const update = {
      version: '0.2.0',
      currentVersion: '0.1.1',
      downloadAndInstall: vi.fn(),
    }
    updaterMock.check.mockResolvedValue(update)
    const appStore = useAppStore()
    const updateStore = useUpdateStore()

    await updateStore.checkForUpdates()

    expect(updateStore.availableVersion).toBe('0.2.0')
    expect(updateStore.hasAvailableUpdate).toBe(true)
    expect(appStore.message).toBe('发现新版本 0.2.0')
  })

  it('downloads, tracks progress, and relaunches after install', async () => {
    const downloadAndInstall = vi.fn(async (onEvent: (event: unknown) => void) => {
      onEvent({ event: 'Started', data: { contentLength: 100 } })
      onEvent({ event: 'Progress', data: { chunkLength: 35 } })
      onEvent({ event: 'Progress', data: { chunkLength: 65 } })
      onEvent({ event: 'Finished' })
    })
    updaterMock.check.mockResolvedValue({
      version: '0.2.0',
      currentVersion: '0.1.1',
      downloadAndInstall,
    })
    const appStore = useAppStore()
    const updateStore = useUpdateStore()

    await updateStore.installAvailableUpdate()

    expect(downloadAndInstall).toHaveBeenCalled()
    expect(updateStore.progressPercent).toBe(100)
    expect(processMock.relaunch).toHaveBeenCalled()
    expect(appStore.message).toBe('更新已安装，正在重启')
  })

  it('explains updater binary format mismatches', async () => {
    updaterMock.check.mockResolvedValue({
      version: '0.2.0',
      currentVersion: '0.1.1',
      downloadAndInstall: vi.fn().mockRejectedValue('invalid updater binary format'),
    })
    const appStore = useAppStore()
    const updateStore = useUpdateStore()

    await updateStore.installAvailableUpdate()

    expect(appStore.message).toBe('更新包格式与当前安装方式不匹配，请下载对应的 deb/AppImage 安装包后重试')
  })
})
