import { describe, expect, it } from 'vitest'
import { defaultProfiles, defaultServiceDraft, defaultSettings, defaultStatus } from '@/shared/domain/defaults'
import { validateProfileStart } from '@/shared/domain/startupValidation'

describe('startup validation', () => {
  it('requires at least one enabled service before checking ports or starting', () => {
    const issues = validateProfileStart(defaultProfiles().profiles[0], defaultSettings(), defaultStatus())
    expect(issues).toContainEqual({ id: 'no-enabled-services', message: '请至少启用一个服务后再启动' })
  })
  it('rejects a service domain with a trailing quote', () => {
    const settings = defaultSettings()
    settings.tunnels[0].ssh.host = 'ssh.example.com'
    settings.tunnels[0].ssh.username = 'tunnel'

    const profile = defaultProfiles().profiles[0]
    profile.services.push({
      ...defaultServiceDraft(),
      id: 'mysql',
      name: 'MySQL',
      domain: 'mysql.example.internal"',
      port: 3306,
    })

    const status = defaultStatus()
    status.privilege.canModifyHosts = true

    const issues = validateProfileStart(profile, settings, status)

    expect(issues).toContainEqual({ id: 'mysql:domain', message: 'MySQL 的域名格式无效' })
  })
})
