import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'
import { defaultProfiles, defaultSettings, defaultStatus, defaultServiceDraft } from '@/shared/domain/defaults'
import { useAppStore } from '@/stores/appStore'
import { flushPromises } from '@vue/test-utils'
import type { AppSettings } from '@/shared/types'

function deferred<T>() {
  let resolve!: (value: T) => void
  let reject!: (error: Error) => void
  const promise = new Promise<T>((done, fail) => { resolve = done; reject = fail })
  return { promise, resolve, reject }
}

const mockApi = vi.hoisted(() => ({
  loadSettings: vi.fn(),
  saveSettings: vi.fn(),
  launchAtLoginEnabled: vi.fn(),
  setLaunchAtLogin: vi.fn(),
  loadProfiles: vi.fn(),
  saveProfiles: vi.fn(),
  listConfigBackups: vi.fn(),
  restoreConfigBackup: vi.fn(),
  deleteConfigBackup: vi.fn(),
  exportProfiles: vi.fn(),
  previewProfilesImport: vi.fn(),
  applyProfilesImport: vi.fn(),
  saveTunnelPassword: vi.fn(),
  deleteTunnelPassword: vi.fn(),
  hasTunnelPassword: vi.fn(),
  testSsh: vi.fn(),
  startProfile: vi.fn(),
  stopProfile: vi.fn(),
  getStatus: vi.fn(),
  testService: vi.fn(),
  repairHosts: vi.fn(),
  openLogDir: vi.fn(),
}))

const mockDialog = vi.hoisted(() => ({
  open: vi.fn(),
  save: vi.fn(),
}))

vi.mock('@/shared/api/tauri', () => ({
  api: mockApi,
}))

vi.mock('@tauri-apps/plugin-dialog', () => mockDialog)

describe('appStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
    mockApi.loadSettings.mockResolvedValue(defaultSettings())
    mockApi.saveSettings.mockImplementation(async (settings) => settings)
    mockApi.launchAtLoginEnabled.mockResolvedValue(false)
    mockApi.setLaunchAtLogin.mockResolvedValue(true)
    mockApi.loadProfiles.mockResolvedValue(defaultProfiles())
    mockApi.saveProfiles.mockImplementation(async (profiles) => profiles)
    mockApi.listConfigBackups.mockResolvedValue([])
    mockApi.restoreConfigBackup.mockResolvedValue({ settings: defaultSettings(), profiles: defaultProfiles() })
    mockApi.deleteConfigBackup.mockResolvedValue(undefined)
    mockApi.exportProfiles.mockResolvedValue(undefined)
    mockApi.previewProfilesImport.mockResolvedValue(emptyImportPreview())
    mockApi.applyProfilesImport.mockResolvedValue({
      settings: defaultSettings(),
      profiles: defaultProfiles(),
      preview: emptyImportPreview(),
      backupPath: '',
    })
    mockApi.hasTunnelPassword.mockResolvedValue(false)
    mockApi.saveTunnelPassword.mockResolvedValue(undefined)
    mockApi.deleteTunnelPassword.mockResolvedValue(undefined)
    mockApi.testSsh.mockResolvedValue(undefined)
    mockApi.startProfile.mockResolvedValue({ ...defaultStatus(), running: true, runningTunnelIds: ['default'] })
    mockApi.stopProfile.mockResolvedValue({ ...defaultStatus(), running: false })
    mockApi.getStatus.mockResolvedValue(defaultStatus())
    mockApi.repairHosts.mockResolvedValue(undefined)
    mockApi.openLogDir.mockResolvedValue(undefined)
    mockDialog.open.mockResolvedValue('profiles.json')
    mockDialog.save.mockResolvedValue('profiles-export.json')
    const store = useAppStore()
    store.initialized = true
    store.statusReady = true
  })

  it('serializes settings writes and coalesces later edits without overwriting them', async () => {
    const store = useAppStore()
    await store.refresh()
    const first = deferred<AppSettings>()
    const second = deferred<AppSettings>()
    mockApi.saveSettings.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise)
    store.settings.behavior.themeMode = 'dark'
    const save1 = store.saveSettingsOnly()
    await flushPromises()
    store.settings.behavior.themeMode = 'light'
    const save2 = store.saveSettingsOnly()
    store.settings.behavior.startMinimized = true
    const save3 = store.saveSettingsOnly()
    expect(store.settingsSaving).toBe(true)
    expect(store.settingsControlsDisabled).toBe(false)
    expect(mockApi.saveSettings).toHaveBeenCalledTimes(1)
    first.resolve(mockApi.saveSettings.mock.calls[0][0])
    await flushPromises()
    expect(store.settings.behavior.themeMode).toBe('light')
    expect(store.settings.behavior.startMinimized).toBe(true)
    expect(mockApi.saveSettings).toHaveBeenCalledTimes(2)
    expect(mockApi.saveSettings.mock.calls[1][0].behavior).toMatchObject({ themeMode: 'light', startMinimized: true })
    expect(store.settingsSaveState).toBe('saving')
    second.resolve(mockApi.saveSettings.mock.calls[1][0])
    expect(await Promise.all([save1, save2, save3])).toEqual([true, true, true])
    expect(store.settingsSaveState).toBe('saved')
    expect(store.loading).toBe(false)
  })

  it('continues with the newest preferences when an older write fails', async () => {
    const store = useAppStore()
    const first = deferred<AppSettings>()
    mockApi.saveSettings.mockReturnValueOnce(first.promise)
    store.settings.behavior.themeMode = 'dark'
    const saving = store.saveSettingsOnly()
    await flushPromises()
    store.settings.behavior.themeMode = 'light'
    const latest = store.saveSettingsOnly()
    first.reject(new Error('temporary failure'))
    expect(await Promise.all([saving, latest])).toEqual([true, true])
    expect(store.settings.behavior.themeMode).toBe('light')
    expect(store.settingsSaveState).toBe('saved')
  })

  it('coalesces preference changes before a write starts', async () => {
    const store = useAppStore()
    store.settings.behavior.themeMode = 'dark'
    const first = store.saveSettingsOnly()
    store.settings.behavior.themeMode = 'light'
    const second = store.saveSettingsOnly()
    expect(await Promise.all([first, second])).toEqual([true, true])
    expect(mockApi.saveSettings).toHaveBeenCalledTimes(1)
    expect(mockApi.saveSettings.mock.calls[0][0].behavior.themeMode).toBe('light')
  })

  it('keeps failed preferences for retry without committing tunnel drafts', async () => {
    const store = useAppStore()
    await store.refresh()
    store.currentTunnel.name = 'Unfinished tunnel'
    store.settings.behavior.themeMode = 'dark'
    mockApi.saveSettings.mockRejectedValueOnce(new Error('disk full'))
    expect(await store.saveSettingsOnly()).toBe(false)
    expect(store.settingsSaveState).toBe('error')
    expect(store.settingsSaveError).toContain('disk full')
    expect(store.settings.behavior.themeMode).toBe('dark')
    expect(await store.saveSettingsOnly()).toBe(true)
    expect(mockApi.saveSettings.mock.calls[1][0].tunnels[0].name).toBe(defaultSettings().tunnels[0].name)
    expect(store.currentTunnel.name).toBe('Unfinished tunnel')
    expect(store.currentTunnelDirty).toBe(true)
    expect(store.settingsSaveError).toBe('')
  })

  it('queues preference saves behind another operation and preserves edits made during that save', async () => {
    const store = useAppStore()
    await store.refresh()
    const tunnelSave = deferred<AppSettings>()
    mockApi.saveSettings.mockReturnValueOnce(tunnelSave.promise)
    store.currentTunnel.name = 'Saved tunnel'
    const savingTunnel = store.saveTunnel()
    await flushPromises()
    store.settings.behavior.themeMode = 'dark'
    const savingBehavior = store.saveSettingsOnly()
    await flushPromises()
    expect(mockApi.saveSettings).toHaveBeenCalledTimes(1)
    tunnelSave.resolve(mockApi.saveSettings.mock.calls[0][0])
    await savingTunnel
    expect(await savingBehavior).toBe(true)
    expect(store.settings.behavior.themeMode).toBe('dark')
    expect(mockApi.saveSettings.mock.calls[1][0]).toMatchObject({ tunnels: [{ ...store.currentTunnel }], behavior: { themeMode: 'dark' } })
  })

  it('ignores a startup query after a user changes the preference and serializes rapid toggles', async () => {
    const store = useAppStore()
    await store.refresh()
    const query = deferred<boolean>()
    const registry = deferred<boolean>()
    mockApi.launchAtLoginEnabled.mockReturnValueOnce(query.promise)
    mockApi.setLaunchAtLogin.mockReturnValueOnce(registry.promise).mockResolvedValueOnce(false)
    const querying = store.refreshLaunchAtLoginState()
    store.settings.behavior.launchAtLogin = true
    const enabling = store.updateLaunchAtLogin()
    await flushPromises()
    query.resolve(false)
    await querying
    expect(store.settings.behavior.launchAtLogin).toBe(true)
    store.settings.behavior.launchAtLogin = false
    const disabling = store.updateLaunchAtLogin()
    expect(mockApi.setLaunchAtLogin).toHaveBeenCalledTimes(1)
    registry.resolve(true)
    expect(await Promise.all([enabling, disabling])).toEqual([true, true])
    expect(mockApi.setLaunchAtLogin.mock.calls).toEqual([[true], [false]])
    expect(store.settings.behavior.launchAtLogin).toBe(false)
    expect(mockApi.saveSettings.mock.lastCall?.[0].behavior.launchAtLogin).toBe(false)
  })

  it('retries the registry operation after failure and does not let a query erase the pending preference', async () => {
    const store = useAppStore()
    await store.refresh()
    mockApi.setLaunchAtLogin.mockRejectedValueOnce(new Error('access denied'))
    store.settings.behavior.launchAtLogin = true
    expect(await store.updateLaunchAtLogin()).toBe(false)
    expect(mockApi.saveSettings).not.toHaveBeenCalled()
    await store.refreshLaunchAtLoginState()
    expect(mockApi.launchAtLoginEnabled).not.toHaveBeenCalled()
    expect(store.settings.behavior.launchAtLogin).toBe(true)
    expect(await store.saveSettingsOnly()).toBe(true)
    expect(mockApi.setLaunchAtLogin).toHaveBeenCalledTimes(2)
  })

  it('refreshes runtime status without loading configuration or disturbing drafts', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    store.currentTunnel.name = 'Draft'
    const loads = mockApi.loadSettings.mock.calls.length
    const profileLoads = mockApi.loadProfiles.mock.calls.length
    const passwords = mockApi.hasTunnelPassword.mock.calls.length
    await store.refreshRuntimeStatus()
    expect(mockApi.loadSettings).toHaveBeenCalledTimes(loads)
    expect(mockApi.loadProfiles).toHaveBeenCalledTimes(profileLoads)
    expect(mockApi.hasTunnelPassword).toHaveBeenCalledTimes(passwords)
    expect(store.profilesDirty).toBe(true)
    expect(store.currentTunnel.name).toBe('Draft')
    expect(store.unsavedProfilesConfirmOpen).toBe(false)
    expect(store.message).toBe('状态已刷新')
  })

  it('ignores stale status responses and failures after a newer request or stop', async () => {
    const store = useAppStore()
    const older = deferred<ReturnType<typeof defaultStatus>>()
    mockApi.getStatus.mockReturnValueOnce(older.promise)
    const refreshing = store.refreshStatus()
    await store.refreshStatus()
    older.resolve({ ...defaultStatus(), running: true })
    await refreshing
    expect(store.status.running).toBe(false)
    const failed = deferred<ReturnType<typeof defaultStatus>>()
    mockApi.getStatus.mockReturnValueOnce(failed.promise)
    const failing = store.refreshStatus()
    await store.stop()
    failed.reject(new Error('outdated error'))
    await expect(failing).resolves.toBeUndefined()
    expect(store.message).toBe('隧道已停止')
  })

  it('confirms service drafts before reloading and cancels on save failure', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    const loads = mockApi.loadSettings.mock.calls.length
    await store.reload()
    expect(store.unsavedProfilesConfirmOpen).toBe(true)
    store.cancelUnsavedProfilesAction()
    expect(store.profilesDirty).toBe(true)
    expect(mockApi.loadSettings).toHaveBeenCalledTimes(loads)
    await store.reload()
    mockApi.saveProfiles.mockRejectedValueOnce(new Error('disk full'))
    await store.confirmUnsavedProfilesAction(true)
    expect(mockApi.loadSettings).toHaveBeenCalledTimes(loads)
    expect(store.profilesDirty).toBe(true)
    await store.confirmUnsavedProfilesAction(false)
    expect(mockApi.loadSettings).toHaveBeenCalledTimes(loads + 1)
    expect(store.profilesDirty).toBe(false)
    expect(store.message).toBe('配置已重新加载')
  })

  it('blocks mutations until initialization and status have completed', () => {
    const store = useAppStore()
    store.initialized = false
    store.statusReady = false
    expect(store.addService(serviceDraft())).toBe(false)
    store.initialized = true
    expect(store.canEditServices).toBe(false)
    store.statusReady = true
    expect(store.canEditServices).toBe(true)
  })

  it('keeps drafts when saving fails and does not continue a pending action', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    const next = vi.fn(async () => {})
    await store.runAfterUnsavedProfilesConfirm(next)
    mockApi.saveProfiles.mockRejectedValueOnce(new Error('disk full'))
    await store.confirmUnsavedProfilesAction(true)
    expect(store.profilesDirty).toBe(true)
    expect(store.currentProfile.services).toHaveLength(1)
    expect(store.unsavedProfilesConfirmOpen).toBe(true)
    expect(next).not.toHaveBeenCalled()
    expect(store.message).toContain('disk full')
  })

  it('treats a status refresh failure after persistence as a successful save', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    mockApi.getStatus.mockRejectedValueOnce(new Error('offline'))
    expect(await store.saveProfiles()).toBe(true)
    expect(store.profilesDirty).toBe(false)
    expect(store.currentProfile.services).toHaveLength(1)
    expect(store.saveRefreshWarning).toContain('已保存，但状态刷新失败')
  })

  it.each(['save', 'discard', 'cancel'] as const)('handles %s before switching profiles', async (choice) => {
    const store = useAppStore()
    const profiles = defaultProfiles()
    profiles.profiles.push({ id: 'team', name: 'Team', enabled: true, services: [] })
    mockApi.loadProfiles.mockResolvedValue(profiles)
    await store.refresh()
    store.addService(serviceDraft())
    await store.selectProfile('team')
    expect(store.settings.currentProfileId).toBe('default')
    expect(store.unsavedProfilesConfirmOpen).toBe(true)
    if (choice === 'cancel') store.cancelUnsavedProfilesAction()
    else await store.confirmUnsavedProfilesAction(choice === 'save')
    expect(store.settings.currentProfileId).toBe(choice === 'cancel' ? 'default' : 'team')
    expect(store.profiles.profiles[0].services).toHaveLength(choice === 'discard' ? 0 : 1)
    expect(mockApi.saveProfiles).toHaveBeenCalledTimes(choice === 'save' ? 1 : 0)
  })

  it('restores the selected profile when persistence fails', async () => {
    const store = useAppStore()
    await store.refresh()
    await store.createProfile('Team')
    mockApi.saveSettings.mockRejectedValueOnce(new Error('cannot save selection'))
    await store.selectProfile('default')
    expect(store.settings.currentProfileId).toBe('team')
  })

  it('guards against duplicate saves and editing during a save', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    let finish!: (value: ReturnType<typeof defaultProfiles>) => void
    mockApi.saveProfiles.mockImplementationOnce((value) => new Promise((resolve) => { finish = () => resolve(value) }))
    const saving = store.saveProfiles()
    expect(await store.saveProfiles()).toBe(false)
    expect(store.updateService('mysql', { ...serviceDraft(), name: 'changed' })).toBe(false)
    finish(defaultProfiles())
    expect(await saving).toBe(true)
    expect(mockApi.saveProfiles).toHaveBeenCalledTimes(1)
  })

  it('blocks service and profile modifications while running', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    await store.saveProfiles()
    const before = JSON.stringify(store.profiles)
    store.status.running = true
    expect(store.addService(serviceDraft())).toBe(false)
    expect(store.updateService('mysql', { ...serviceDraft(), enabled: false })).toBe(false)
    store.removeService('mysql')
    store.moveService('mysql', 1)
    expect(await store.saveProfiles()).toBe(false)
    expect(await store.restoreConfigBackup('backup')).toBe(false)
    expect(await store.applyProfilesImport('profiles.json', [])).toBeUndefined()
    expect(JSON.stringify(store.profiles)).toBe(before)
    expect(mockApi.restoreConfigBackup).not.toHaveBeenCalled()
  })

  it('preserves service drafts and unrelated settings when saving a tunnel or discarding services', async () => {
    const store = useAppStore()
    await store.refresh()
    store.addService(serviceDraft())
    store.currentTunnel.name = 'Local jump host'
    await store.saveTunnel()
    expect(store.profilesDirty).toBe(true)
    store.currentTunnel.ssh.username = 'unsaved-user'
    store.discardProfilesChanges()
    expect(store.currentProfile.services).toHaveLength(0)
    expect(store.currentTunnel.ssh.username).toBe('unsaved-user')
  })

  it('asks about drafts before startup and validates the saved configuration', async () => {
    const store = useAppStore()
    await store.refresh()
    store.currentTunnel.ssh.host = 'jump.example.com'
    store.currentTunnel.ssh.username = 'dev'
    store.status.privilege.canModifyHosts = true
    mockApi.getStatus.mockResolvedValue({ ...defaultStatus(), privilege: { ...defaultStatus().privilege, canModifyHosts: true } })
    store.addService(serviceDraft())
    await store.start()
    expect(mockApi.startProfile).not.toHaveBeenCalled()
    await store.confirmUnsavedProfilesAction(true)
    expect(mockApi.saveProfiles).toHaveBeenCalledTimes(1)
    expect(mockApi.startProfile).toHaveBeenCalledTimes(1)
  })

  it('loads settings, profiles, status, and password state', async () => {
    const store = useAppStore()

    await store.refresh()

    expect(store.initialized).toBe(true)
    expect(store.currentTunnel.id).toBe('default')
    expect(mockApi.hasTunnelPassword).toHaveBeenCalledWith('default')
  })

  it('bootstraps config before background status completes', async () => {
    let resolveStatus: (value: unknown) => void = () => {}
    const statusPromise = new Promise((resolve) => {
      resolveStatus = resolve
    })
    mockApi.getStatus.mockReturnValue(statusPromise)
    const store = useAppStore()

    await store.bootstrap()

    expect(store.initialized).toBe(true)
    expect(mockApi.loadSettings).toHaveBeenCalled()
    expect(mockApi.loadProfiles).toHaveBeenCalled()
    expect(mockApi.getStatus).toHaveBeenCalled()
    expect(store.status.running).toBe(false)

    resolveStatus({ ...defaultStatus(), running: true, runningTunnelIds: ['default'] })
    await statusPromise
    await Promise.resolve()

    expect(store.status.running).toBe(true)
    expect(store.status.runningTunnelIds).toEqual(['default'])
  })

  it('reload waits for status and updates the store', async () => {
    mockApi.getStatus.mockResolvedValue({ ...defaultStatus(), running: true, runningTunnelIds: ['default'] })
    const store = useAppStore()

    await store.reload()

    expect(store.status.running).toBe(true)
    expect(store.status.runningTunnelIds).toEqual(['default'])
    expect(store.message).toBe('配置已重新加载')
  })

  it('starts and stops the active profile', async () => {
    const store = useAppStore()
    mockApi.getStatus.mockResolvedValue({
      ...defaultStatus(),
      privilege: { ...defaultStatus().privilege, canModifyHosts: true },
    })

    await store.refresh()

    await store.start()
    expect(store.status.running).toBe(true)
    expect(store.status.runningTunnelIds).toEqual(['default'])

    await store.stop()
    expect(store.status.running).toBe(false)
  })

  it('saves tunnel passwords without exposing saved values', async () => {
    const store = useAppStore()

    await store.refresh()
    await store.saveTunnel('secret')

    expect(mockApi.saveSettings).toHaveBeenCalled()
    expect(mockApi.saveTunnelPassword).toHaveBeenCalledWith('default', 'secret')
  })

  it('adds and removes services in the default profile', () => {
    const store = useAppStore()

    const added = store.addService({
      id: '',
      name: 'MySQL',
      group: '',
      domain: 'mysql.internal',
      remark: '',
      port: 3306,
      localIp: '127.77.0.10',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })

    expect(added).toBe(true)
    expect(store.currentProfile.services).toHaveLength(1)

    store.removeService('mysql')
    expect(store.currentProfile.services).toHaveLength(0)
  })

  it('updates an existing service without changing its id', () => {
    const store = useAppStore()

    store.addService({
      id: '',
      name: 'MySQL',
      group: '',
      domain: 'mysql.internal',
      remark: '',
      port: 3306,
      localIp: '127.77.0.10',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })

    const updated = store.updateService('mysql', {
      id: 'ignored',
      name: 'Primary MySQL',
      group: 'Database',
      domain: 'primary-mysql.internal',
      remark: 'Primary database',
      port: 3307,
      localIp: '127.77.0.11',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: false,
    })

    expect(updated).toBe(true)
    expect(store.currentProfile.services[0]).toEqual(expect.objectContaining({
      id: 'mysql',
      name: 'Primary MySQL',
      group: 'Database',
      domain: 'primary-mysql.internal',
      remark: 'Primary database',
      port: 3307,
      localIp: '127.77.0.11',
      enabled: false,
    }))
  })

  it('tracks unsaved profile changes and clears the flag after saving', async () => {
    const store = useAppStore()

    await store.refresh()
    expect(store.profilesDirty).toBe(false)

    store.addService({
      id: '',
      name: 'MySQL',
      group: '',
      domain: 'mysql.internal',
      remark: '',
      port: 3306,
      localIp: '127.77.0.10',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })

    expect(store.profilesDirty).toBe(true)
    await store.saveProfiles()
    expect(store.profilesDirty).toBe(false)
  })

  it('groups services and moves order within a group', async () => {
    const store = useAppStore()

    store.addService({
      id: '',
      name: 'MySQL',
      group: 'Database',
      domain: 'mysql.internal',
      remark: '',
      port: 3306,
      localIp: '127.77.0.10',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })
    store.addService({
      id: '',
      name: 'Postgres',
      group: 'Database',
      domain: 'postgres.internal',
      remark: '',
      port: 5432,
      localIp: '127.77.0.11',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })

    expect(store.serviceGroups[0].label).toBe('Database')
    expect(store.serviceGroups[0].services.map((service) => service.id)).toEqual(['mysql', 'postgres'])

    store.moveService('postgres', -1)

    expect(store.serviceGroups[0].services.map((service) => service.id)).toEqual(['postgres', 'mysql'])
    await store.saveProfiles()

    expect(mockApi.saveProfiles).toHaveBeenCalledWith(
      expect.objectContaining({
        profiles: [
          expect.objectContaining({
            services: [
              expect.objectContaining({ id: 'postgres', group: 'Database', sortOrder: 10 }),
              expect.objectContaining({ id: 'mysql', group: 'Database', sortOrder: 20 }),
            ],
          }),
        ],
      }),
    )
  })

  it('reorders services by dragging within the same group', () => {
    const store = useAppStore()

    store.addService({
      id: '',
      name: 'MySQL',
      group: 'Database',
      domain: 'mysql.internal',
      remark: '',
      port: 3306,
      localIp: '127.77.0.10',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })
    store.addService({
      id: '',
      name: 'Postgres',
      group: 'Database',
      domain: 'postgres.internal',
      remark: '',
      port: 5432,
      localIp: '127.77.0.11',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })
    store.addService({
      id: '',
      name: 'Redis',
      group: 'Cache',
      domain: 'redis.internal',
      remark: '',
      port: 6379,
      localIp: '127.77.0.12',
      tunnelId: 'default',
      sortOrder: 0,
      enabled: true,
    })

    expect(store.reorderService('postgres', 'mysql', 'before')).toBe(true)
    expect(store.serviceGroups.find((group) => group.label === 'Database')?.services.map((service) => service.id)).toEqual(['postgres', 'mysql'])
    expect(store.reorderService('mysql', 'redis', 'before')).toBe(false)
  })

  it('switches profiles by saving current profile selection', async () => {
    const store = useAppStore()
    mockApi.loadProfiles.mockResolvedValue({
      schemaVersion: 2,
      profiles: [
        { id: 'default', name: 'Default Profile', enabled: true, services: [] },
        { id: 'team', name: 'Team Profile', enabled: true, services: [] },
      ],
    })

    await store.refresh()
    await store.selectProfile('team')

    expect(store.settings.currentProfileId).toBe('team')
    expect(mockApi.saveSettings).toHaveBeenLastCalledWith(expect.objectContaining({ currentProfileId: 'team' }))
  })

  it('creates a profile and switches to it', async () => {
    const store = useAppStore()

    const created = await store.createProfile('Team Profile')

    expect(created).toBe(true)
    expect(store.settings.currentProfileId).toBe('team-profile')
    expect(store.currentProfile.name).toBe('Team Profile')
    expect(mockApi.saveProfiles).toHaveBeenCalledWith(
      expect.objectContaining({
        profiles: expect.arrayContaining([expect.objectContaining({ id: 'team-profile', name: 'Team Profile' })]),
      }),
    )
    expect(mockApi.saveSettings).toHaveBeenLastCalledWith(expect.objectContaining({ currentProfileId: 'team-profile' }))
  })

  it('renames a profile', async () => {
    const store = useAppStore()

    const renamed = await store.renameProfile('default', 'Renamed Profile')

    expect(renamed).toBe(true)
    expect(store.currentProfile.name).toBe('Renamed Profile')
    expect(mockApi.saveProfiles).toHaveBeenCalledWith(
      expect.objectContaining({
        profiles: expect.arrayContaining([expect.objectContaining({ id: 'default', name: 'Renamed Profile' })]),
      }),
    )
    expect(mockApi.saveSettings).not.toHaveBeenCalled()
  })

  it('prevents duplicate profile names when renaming', async () => {
    const store = useAppStore()
    mockApi.loadProfiles.mockResolvedValue({
      schemaVersion: 2,
      profiles: [
        { id: 'default', name: 'Default Profile', enabled: true, services: [] },
        { id: 'team', name: 'Team Profile', enabled: true, services: [] },
      ],
    })

    await store.refresh()
    mockApi.saveProfiles.mockClear()
    const renamed = await store.renameProfile('team', 'Default Profile')

    expect(renamed).toBe(false)
    expect(mockApi.saveProfiles).not.toHaveBeenCalled()
    expect(store.message).toContain('Profile 已存在')
  })

  it('deletes the current profile and switches to a remaining one', async () => {
    const store = useAppStore()
    mockApi.loadSettings.mockResolvedValue({ ...defaultSettings(), currentProfileId: 'team' })
    mockApi.loadProfiles.mockResolvedValue({
      schemaVersion: 2,
      profiles: [
        { id: 'default', name: 'Default Profile', enabled: true, services: [] },
        { id: 'team', name: 'Team Profile', enabled: true, services: [] },
      ],
    })

    await store.refresh()
    const deleted = await store.deleteProfile('team')

    expect(deleted).toBe(true)
    expect(store.settings.currentProfileId).toBe('default')
    expect(store.profiles.profiles).toHaveLength(1)
    expect(mockApi.saveProfiles).toHaveBeenCalledWith(
      expect.objectContaining({
        profiles: [expect.objectContaining({ id: 'default' })],
      }),
    )
    expect(mockApi.saveSettings).toHaveBeenLastCalledWith(expect.objectContaining({ currentProfileId: 'default' }))
  })

  it('keeps at least one profile when deleting', async () => {
    const store = useAppStore()

    await store.refresh()
    const deleted = await store.deleteProfile('default')

    expect(deleted).toBe(false)
    expect(mockApi.saveProfiles).not.toHaveBeenCalled()
    expect(store.message).toContain('至少保留一个 Profile')
  })

  it('blocks profile switching and imports while running', async () => {
    const store = useAppStore()
    mockApi.getStatus.mockResolvedValue({ ...defaultStatus(), running: true, runningTunnelIds: ['default'] })

    await store.refresh()
    await store.selectProfile('team')
    const renamed = await store.renameProfile('default', 'Renamed Profile')
    const session = await store.previewProfilesImport()

    expect(renamed).toBe(false)
    expect(session).toBeUndefined()
    expect(mockApi.saveSettings).not.toHaveBeenCalled()
    expect(mockApi.saveProfiles).not.toHaveBeenCalled()
    expect(mockApi.previewProfilesImport).not.toHaveBeenCalled()
    expect(store.message).toContain('运行中不能导入')
  })

  it('previews and applies profile imports', async () => {
    const store = useAppStore()
    const importedProfiles = {
      schemaVersion: 2,
      profiles: [{ id: 'team', name: 'Team Profile', enabled: true, services: [] }],
    }
    const importedSettings = { ...defaultSettings(), currentProfileId: 'team' }
    mockApi.previewProfilesImport.mockResolvedValue({
      ...emptyImportPreview(),
      profileCount: 1,
      importedProfileIds: ['team'],
      overwrites: [
        {
          profileId: 'default',
          profileName: 'Default Profile',
          serviceId: 'mysql',
          oldName: 'MySQL',
          oldGroup: '',
          oldDomain: 'mysql.old',
          oldPort: 3306,
          oldLocalIp: '127.77.0.10',
          oldTunnelId: 'default',
          oldSortOrder: 10,
          newName: 'MySQL',
          newGroup: 'Database',
          newDomain: 'mysql.new',
          newPort: 3306,
          newLocalIp: '127.77.0.10',
          newTunnelId: 'default',
          newSortOrder: 20,
        },
      ],
    })
    mockApi.applyProfilesImport.mockResolvedValue({
      settings: importedSettings,
      profiles: importedProfiles,
      preview: emptyImportPreview(),
      backupPath: 'profiles-backup.json',
    })

    const session = await store.previewProfilesImport()
    const result = await store.applyProfilesImport('profiles.json', [])

    expect(session?.preview.profileCount).toBe(1)
    expect(mockApi.previewProfilesImport).toHaveBeenCalledWith('profiles.json', [])
    expect(result?.backupPath).toBe('profiles-backup.json')
    expect(store.settings.currentProfileId).toBe('team')
    expect(store.profiles.profiles[0].id).toBe('team')
    expect(mockApi.listConfigBackups).toHaveBeenCalled()
  })

  it('restores and deletes config backups', async () => {
    const store = useAppStore()
    const restoredProfiles = {
      schemaVersion: 2,
      profiles: [{ id: 'restored', name: 'Restored Profile', enabled: true, services: [] }],
    }
    mockApi.listConfigBackups.mockResolvedValue([{ id: 'backup-1', fileName: 'backup-1.json', createdAt: '2026-07-04T00:00:00+08:00' }])
    mockApi.restoreConfigBackup.mockResolvedValue({
      settings: { ...defaultSettings(), currentProfileId: 'restored' },
      profiles: restoredProfiles,
    })

    await store.refreshConfigBackups()
    const restored = await store.restoreConfigBackup('backup-1')
    const deleted = await store.deleteConfigBackup('backup-1')

    expect(store.configBackups).toHaveLength(1)
    expect(restored).toBe(true)
    expect(deleted).toBe(true)
    expect(store.settings.currentProfileId).toBe('restored')
    expect(store.profiles.profiles[0].id).toBe('restored')
    expect(mockApi.deleteConfigBackup).toHaveBeenCalledWith('backup-1')
  })
})

function emptyImportPreview() {
  return {
    profileCount: 0,
    serviceCount: 0,
    addedProfileCount: 0,
    addedServiceCount: 0,
    updatedServiceCount: 0,
    skippedServiceCount: 0,
    importedProfileIds: [],
    missingTunnels: [],
    overwrites: [],
    conflicts: [],
    canApply: true,
  }
}

function serviceDraft() {
  return { ...defaultServiceDraft(), id: 'mysql', name: 'MySQL', domain: 'mysql.internal' }
}
