import { invoke } from '@tauri-apps/api/core'
import { defaultProfiles, defaultSettings, defaultStatus } from '@/shared/domain/defaults'
import type {
  AppLogEntry,
  AppSettings,
  AppStatus,
  ConfigBackupInfo,
  ConfigRestoreResult,
  ProfilesFile,
  ProfilesImportApplyResult,
  ProfilesImportPreview,
  ServiceStatus,
  TunnelMapping,
} from '@/shared/types'

type InvokeArgs = Record<string, unknown>

function hasTauriInvoke() {
  const tauriWindow = window as typeof window & { __TAURI_INTERNALS__?: { invoke?: unknown } }
  return typeof tauriWindow.__TAURI_INTERNALS__?.invoke === 'function'
}

async function invokeCommand<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  if (hasTauriInvoke()) {
    return invoke<T>(cmd, args)
  }
  if (import.meta.env.DEV) {
    return devInvoke<T>(cmd, args)
  }
  throw new Error('Tauri API is unavailable')
}

async function devInvoke<T>(cmd: string, args?: InvokeArgs): Promise<T> {
  switch (cmd) {
    case 'load_settings':
      return defaultSettings() as T
    case 'save_settings':
      return (args?.settings || defaultSettings()) as T
    case 'load_profiles':
      return defaultProfiles() as T
    case 'save_profiles':
      return (args?.profiles || defaultProfiles()) as T
    case 'list_config_backups':
      return [] as T
    case 'restore_config_backup':
      return { settings: defaultSettings(), profiles: defaultProfiles() } as T
    case 'delete_config_backup':
      return undefined as T
    case 'export_profiles':
      return undefined as T
    case 'preview_profiles_import':
      return emptyImportPreview() as T
    case 'apply_profiles_import':
      return {
        settings: defaultSettings(),
        profiles: defaultProfiles(),
        preview: emptyImportPreview(),
        backupPath: '',
      } as T
    case 'get_status':
    case 'start_profile':
      return defaultStatus() as T
    case 'stop_profile':
      return { ...defaultStatus(), running: false } as T
    case 'read_logs':
      return sampleLogEntries() as T
    case 'launch_at_login_enabled':
    case 'set_launch_at_login':
    case 'has_tunnel_password':
      return false as T
    case 'test_service':
      return { serviceId: String(args?.serviceId || ''), state: 'stopped', message: 'Preview mode' } as T
    default:
      return undefined as T
  }
}

function emptyImportPreview(): ProfilesImportPreview {
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

export const api = {
  loadSettings: () => invokeCommand<AppSettings>('load_settings'),
  launchAtLoginEnabled: () => invokeCommand<boolean>('launch_at_login_enabled'),
  saveSettings: (settings: AppSettings) => invokeCommand<AppSettings>('save_settings', { settings }),
  setLaunchAtLogin: (enabled: boolean) => invokeCommand<boolean>('set_launch_at_login', { enabled }),
  loadProfiles: () => invokeCommand<ProfilesFile>('load_profiles'),
  saveProfiles: (profiles: ProfilesFile) => invokeCommand<ProfilesFile>('save_profiles', { profiles }),
  listConfigBackups: () => invokeCommand<ConfigBackupInfo[]>('list_config_backups'),
  restoreConfigBackup: (backupId: string) =>
    invokeCommand<ConfigRestoreResult>('restore_config_backup', { backupId }),
  deleteConfigBackup: (backupId: string) => invokeCommand<void>('delete_config_backup', { backupId }),
  exportProfiles: (path: string, profileIds: string[]) =>
    invokeCommand<void>('export_profiles', { path, profileIds }),
  previewProfilesImport: (path: string, tunnelMappings: TunnelMapping[]) =>
    invokeCommand<ProfilesImportPreview>('preview_profiles_import', { path, tunnelMappings }),
  applyProfilesImport: (path: string, tunnelMappings: TunnelMapping[]) =>
    invokeCommand<ProfilesImportApplyResult>('apply_profiles_import', { path, tunnelMappings }),
  saveTunnelPassword: (tunnelId: string, value: string) =>
    invokeCommand<void>('save_tunnel_password', { tunnelId, value }),
  deleteTunnelPassword: (tunnelId: string) => invokeCommand<void>('delete_tunnel_password', { tunnelId }),
  hasTunnelPassword: (tunnelId: string) => invokeCommand<boolean>('has_tunnel_password', { tunnelId }),
  testSsh: (tunnelId: string) => invokeCommand<void>('test_ssh', { tunnelId }),
  startProfile: () => invokeCommand<AppStatus>('start_profile'),
  stopProfile: () => invokeCommand<AppStatus>('stop_profile'),
  getStatus: () => invokeCommand<AppStatus>('get_status'),
  testService: (serviceId: string) => invokeCommand<ServiceStatus>('test_service', { serviceId }),
  repairHosts: () => invokeCommand<void>('repair_hosts'),
  readLogs: (maxLines = 600) => invokeCommand<AppLogEntry[]>('read_logs', { maxLines }),
  openLogDir: () => invokeCommand<void>('open_log_dir'),
}

function sampleLogEntries(): AppLogEntry[] {
  return [
    {
      id: 'preview:1',
      timestamp: '2026-07-03T13:56:23+08:00',
      level: 'INFO',
      target: 'tunneldesk_lib::commands',
      message: 'Start profile config loaded service_count=3 elapsed_ms=4',
      raw: '2026-07-03T13:56:23+08:00 INFO tunneldesk_lib::commands: Start profile config loaded service_count=3 elapsed_ms=4',
    },
    {
      id: 'preview:2',
      timestamp: '2026-07-03T13:56:24+08:00',
      level: 'INFO',
      target: 'tunneldesk_lib::commands',
      message: 'Tunnel runtime started tunnel_id=default elapsed_ms=219 total_elapsed_ms=243',
      raw: '2026-07-03T13:56:24+08:00 INFO tunneldesk_lib::commands: Tunnel runtime started tunnel_id=default elapsed_ms=219 total_elapsed_ms=243',
    },
    {
      id: 'preview:3',
      timestamp: '2026-07-03T13:56:41+08:00',
      level: 'DEBUG',
      target: 'tunneldesk_lib::health',
      message: 'Checked local service mysql.internal:3306 state=healthy',
      raw: '2026-07-03T13:56:41+08:00 DEBUG tunneldesk_lib::health: Checked local service mysql.internal:3306 state=healthy',
    },
    {
      id: 'preview:4',
      timestamp: '2026-07-03T13:56:54+08:00',
      level: 'WARN',
      target: 'tunneldesk_lib::hosts',
      message: 'Skipped hosts cleanup on exit; direct hosts access is unavailable',
      raw: '2026-07-03T13:56:54+08:00 WARN tunneldesk_lib::hosts: Skipped hosts cleanup on exit; direct hosts access is unavailable',
    },
    {
      id: 'preview:5',
      timestamp: '2026-07-03T13:57:11+08:00',
      level: 'ERROR',
      target: 'tunneldesk_lib::commands',
      message: 'Failed to start profile: SSH authentication failed',
      raw: '2026-07-03T13:57:11+08:00 ERROR tunneldesk_lib::commands: Failed to start profile: SSH authentication failed',
    },
  ]
}
