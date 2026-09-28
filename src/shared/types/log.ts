export type AppLogLevel = 'TRACE' | 'DEBUG' | 'INFO' | 'WARN' | 'ERROR' | 'UNKNOWN'

export interface AppLogEntry {
  id: string
  timestamp: string
  level: AppLogLevel | string
  target: string
  message: string
  raw: string
}
