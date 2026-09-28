export type ServiceState = 'disabled' | 'stopped' | 'checking' | 'healthy' | 'error' | 'unknown'

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
