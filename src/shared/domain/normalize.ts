import type { AppSettings, AppStatus, ProfilesFile } from '@/shared/types'
import { DEFAULT_PROFILE_ID, DEFAULT_TUNNEL_ID, defaultProfiles, defaultSettings, defaultStatus, defaultSsh, defaultTunnel } from './defaults'

export function normalizeSettings(value?: Partial<AppSettings> | null): AppSettings {
  const fallback = defaultSettings()
  const source = value ?? fallback
  const tunnels = source.tunnels?.length ? source.tunnels : [defaultTunnel()]
  const currentTunnelId = tunnels.some((tunnel) => tunnel.id === source.currentTunnelId)
    ? source.currentTunnelId || DEFAULT_TUNNEL_ID
    : tunnels[0].id

  return {
    ...fallback,
    ...source,
    schemaVersion: 2,
    currentProfileId: source.currentProfileId || DEFAULT_PROFILE_ID,
    currentTunnelId,
    tunnels: tunnels.map((tunnel) => ({
      ...defaultTunnel(),
      ...tunnel,
      enabled: tunnel.enabled ?? true,
      ssh: {
        ...defaultSsh(),
        ...tunnel.ssh,
      },
    })),
    behavior: {
      ...fallback.behavior,
      ...source.behavior,
      themeMode: source.behavior?.themeMode || 'system',
    },
  }
}

export function normalizeProfiles(value?: Partial<ProfilesFile> | null, fallbackTunnelId = DEFAULT_TUNNEL_ID): ProfilesFile {
  const fallback = defaultProfiles()
  const profiles = value?.profiles?.length ? value.profiles : fallback.profiles

  return {
    schemaVersion: 2,
    profiles: profiles.map((profile) => ({
      ...profile,
      id: profile.id || DEFAULT_PROFILE_ID,
      name: profile.name || 'Default Profile',
      enabled: profile.enabled ?? true,
      services: (profile.services || []).map((service, index) => {
        const sortOrder = Number(service.sortOrder)
        return {
          ...service,
          group: service.group?.trim() || '',
          remark: service.remark || '',
          port: Number(service.port),
          tunnelId: service.tunnelId || fallbackTunnelId,
          sortOrder: Number.isFinite(sortOrder) && sortOrder > 0 ? sortOrder : (index + 1) * 10,
          enabled: service.enabled ?? true,
        }
      }),
    })),
  }
}

export function normalizeStatus(value?: Partial<AppStatus> | null): AppStatus {
  const fallback = defaultStatus()

  return {
    ...fallback,
    ...value,
    runningTunnelIds: value?.runningTunnelIds || [],
    tunnels: value?.tunnels || [],
    privilege: {
      ...fallback.privilege,
      ...value?.privilege,
    },
    services: value?.services || [],
  }
}
