import type { AppSettings } from './app'
import type { ProfilesFile } from './profile'

export interface TunnelMapping {
  sourceTunnelId: string
  targetTunnelId: string
}

export interface MissingTunnelImport {
  sourceTunnelId: string
  serviceCount: number
}

export interface ServiceOverwritePreview {
  profileId: string
  profileName: string
  serviceId: string
  oldName: string
  oldGroup: string
  oldDomain: string
  oldPort: number
  oldLocalIp: string
  oldTunnelId: string
  oldSortOrder: number
  newName: string
  newGroup: string
  newDomain: string
  newPort: number
  newLocalIp: string
  newTunnelId: string
  newSortOrder: number
}

export interface ServiceImportConflict {
  profileId: string
  profileName: string
  serviceId: string
  serviceName: string
  localIp: string
  port: number
  existingServiceId: string
  existingServiceName: string
  reason: string
}

export interface ProfilesImportPreview {
  profileCount: number
  serviceCount: number
  addedProfileCount: number
  addedServiceCount: number
  updatedServiceCount: number
  skippedServiceCount: number
  importedProfileIds: string[]
  missingTunnels: MissingTunnelImport[]
  overwrites: ServiceOverwritePreview[]
  conflicts: ServiceImportConflict[]
  canApply: boolean
}

export interface ProfilesImportApplyResult {
  settings: AppSettings
  profiles: ProfilesFile
  preview: ProfilesImportPreview
  backupPath: string
}

export interface ProfilesImportSession {
  path: string
  tunnelMappings: TunnelMapping[]
  preview: ProfilesImportPreview
}
