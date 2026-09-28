import type { AppSettings } from './app'
import type { ProfilesFile } from './profile'

export interface ConfigBackupInfo {
  id: string
  fileName: string
  createdAt: string
}

export interface ConfigRestoreResult {
  settings: AppSettings
  profiles: ProfilesFile
}
