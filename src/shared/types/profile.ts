export interface ServiceConfig {
  id: string
  name: string
  group: string
  domain: string
  remark: string
  port: number
  localIp: string
  tunnelId: string
  sortOrder: number
  enabled: boolean
}

export interface ServiceProfile {
  id: string
  name: string
  enabled: boolean
  services: ServiceConfig[]
}

export interface ProfilesFile {
  schemaVersion: number
  profiles: ServiceProfile[]
}
