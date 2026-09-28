import type { AppSettings, AppStatus, ProfilesFile, ServiceConfig, SshSettings, TunnelConfig } from '@/shared/types'

export const DEFAULT_PROFILE_ID = 'default'
export const DEFAULT_TUNNEL_ID = 'default'

export function defaultSsh(): SshSettings {
  return {
    host: '',
    port: 22,
    username: '',
    authMethod: 'password',
    identityFile: '',
    passwordCredentialKey: '',
    keyPassphraseCredentialKey: '',
    serverAliveInterval: 30,
    serverAliveCountMax: 3,
  }
}

export function defaultTunnel(): TunnelConfig {
  return {
    id: DEFAULT_TUNNEL_ID,
    name: 'Default Tunnel',
    enabled: true,
    ssh: defaultSsh(),
  }
}

export function defaultSettings(): AppSettings {
  return {
    schemaVersion: 2,
    currentProfileId: DEFAULT_PROFILE_ID,
    currentTunnelId: DEFAULT_TUNNEL_ID,
    tunnels: [defaultTunnel()],
    behavior: {
      startMinimized: false,
      autoStartProfile: false,
      launchAtLogin: false,
      autoRepairOnStart: false,
      cleanupOnExit: true,
      themeMode: 'system',
      closeAction: 'ask',
    },
  }
}

export function defaultProfiles(): ProfilesFile {
  return {
    schemaVersion: 2,
    profiles: [{ id: DEFAULT_PROFILE_ID, name: 'Default Profile', enabled: true, services: [] }],
  }
}

export function defaultStatus(): AppStatus {
  return {
    running: false,
    currentProfileId: DEFAULT_PROFILE_ID,
    runningTunnelIds: [],
    tunnels: [],
    isAdmin: false,
    privilege: {
      process: 'unknown',
      hostsAccess: 'unavailable',
      helperInstalled: false,
      canModifyHosts: false,
      message: 'Hosts access is unavailable',
    },
    hostsBlockPresent: false,
    message: 'Stopped',
    services: [],
  }
}

export function defaultServiceDraft(tunnelId = DEFAULT_TUNNEL_ID, localIp = '127.77.0.10'): ServiceConfig {
  return {
    id: '',
    name: '',
    group: '',
    domain: '',
    remark: '',
    port: 3306,
    localIp,
    tunnelId,
    sortOrder: 0,
    enabled: true,
  }
}
