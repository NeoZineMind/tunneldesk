export type AuthMethod = 'password' | 'privateKey' | 'agent'
export type ThemeMode = 'system' | 'light' | 'dark'

export interface SshSettings {
  host: string
  port: number
  username: string
  authMethod: AuthMethod
  identityFile: string
  passwordCredentialKey: string
  keyPassphraseCredentialKey: string
  serverAliveInterval: number
  serverAliveCountMax: number
}

export interface TunnelConfig {
  id: string
  name: string
  enabled: boolean
  ssh: SshSettings
}
