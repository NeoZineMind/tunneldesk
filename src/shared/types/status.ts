import type { StartMode } from './app'

export type ServiceState = 'disabled' | 'stopped' | 'checking' | 'healthy' | 'error' | 'unknown' | 'skipped'

export interface StartupIssue {
  serviceId: string
  serviceName: string
  domain: string
  address: string
  kind: 'listenerConflict' | 'permissionDenied' | 'addressUnavailable' | 'listenerError' | 'domainConflict' | 'sharedDomain' | 'sshError' | 'forwardingError' | 'hostsError'
  osError: number | null
  message: string
  owners: { pid: number; processName: string | null; address: string }[]
}

export interface StartupReport {
  profileId: string
  mode: StartMode
  outcome: 'complete' | 'partial' | 'failed'
  startedServiceIds: string[]
  issues: StartupIssue[]
  message: string
}

export interface ServiceStatus {
  serviceId: string
  state: ServiceState
  message: string
}

export interface TunnelStatus {
  tunnelId: string
  name: string
  running: boolean
  message: string
}

export type ProcessPrivilege = 'root' | 'user' | 'unknown'
export type HostsAccess = 'direct' | 'polkit-helper' | 'unavailable'

export interface PrivilegeStatus {
  process: ProcessPrivilege
  hostsAccess: HostsAccess
  helperInstalled: boolean
  canModifyHosts: boolean
  message: string
}

export interface AppStatus {
  startupReport: StartupReport | null
  running: boolean
  currentProfileId: string
  runningTunnelIds: string[]
  tunnels: TunnelStatus[]
  isAdmin: boolean
  privilege: PrivilegeStatus
  hostsBlockPresent: boolean
  message: string
  services: ServiceStatus[]
}
