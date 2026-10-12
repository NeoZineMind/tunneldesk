import type { ThemeMode, TunnelConfig } from './tunnel'

export type CloseAction = 'ask' | 'minimizeToTray' | 'exit'
export type StartMode = 'strict' | 'partial'

export interface BehaviorSettings {
  startMode: StartMode
  startMinimized: boolean
  autoStartProfile: boolean
  launchAtLogin: boolean
  autoRepairOnStart: boolean
  cleanupOnExit: boolean
  themeMode: ThemeMode
  closeAction: CloseAction
}

export interface AppSettings {
  schemaVersion: number
  currentProfileId: string
  currentTunnelId: string
  tunnels: TunnelConfig[]
  behavior: BehaviorSettings
}
