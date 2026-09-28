import { describe, expect, it } from 'vitest'
import { defaultSettings } from '@/shared/domain/defaults'
import { normalizeProfiles, normalizeSettings, normalizeStatus } from '@/shared/domain/normalize'
import { serviceStateColor, serviceStateText } from '@/shared/domain/serviceStatus'
import { nextLocalIp, nextTunnelId, slugify } from '@/shared/domain/tunnelFactory'
import { findDuplicateListener, isLoopbackIp, isValidDomain, isValidPort } from '@/shared/domain/validators'

describe('domain defaults and normalization', () => {
  it('normalizes missing settings fields', () => {
    const settings = normalizeSettings({
      schemaVersion: 1,
      currentProfileId: '',
      currentTunnelId: 'missing',
      tunnels: [
        {
          id: 'jump',
          name: 'Jump',
          enabled: true,
          ssh: {
            host: 'example.com',
            port: 22,
            username: 'root',
            authMethod: 'password',
            identityFile: '',
            passwordCredentialKey: '',
            keyPassphraseCredentialKey: '',
            serverAliveInterval: 30,
            serverAliveCountMax: 3,
          },
        },
      ],
    })

    expect(settings.schemaVersion).toBe(2)
    expect(settings.currentProfileId).toBe('default')
    expect(settings.currentTunnelId).toBe('jump')
    expect(settings.behavior.cleanupOnExit).toBe(true)
  })

  it('normalizes profiles with fallback tunnel ids', () => {
    const profiles = normalizeProfiles(
      {
        schemaVersion: 1,
        profiles: [
          {
            id: 'default',
            name: 'Default',
            enabled: true,
            services: [
              {
                id: 'mysql',
                name: 'MySQL',
                group: '',
                domain: 'mysql.internal',
                remark: '',
                port: 3306,
                localIp: '127.77.0.10',
                tunnelId: '',
                sortOrder: 0,
                enabled: true,
              },
            ],
          },
        ],
      },
      'jump',
    )

    expect(profiles.schemaVersion).toBe(2)
    expect(profiles.profiles[0].services[0].tunnelId).toBe('jump')
    expect(profiles.profiles[0].services[0].sortOrder).toBe(10)
  })

  it('normalizes status arrays', () => {
    expect(normalizeStatus({ running: true }).runningTunnelIds).toEqual([])
  })
})

describe('domain factories and validators', () => {
  it('creates stable ids and loopback addresses', () => {
    expect(slugify('My SQL Service')).toBe('my-sql-service')
    expect(nextTunnelId([{ id: 'default' }, { id: 'tunnel-1' }])).toBe('tunnel-2')
    expect(nextLocalIp([{ localIp: '127.77.0.10' }])).toBe('127.77.0.11')
  })

  it('validates ports and loopback ips', () => {
    expect(isValidPort(1)).toBe(true)
    expect(isValidPort(65535)).toBe(true)
    expect(isValidPort(0)).toBe(false)
    expect(isLoopbackIp('127.77.0.10')).toBe(true)
    expect(isLoopbackIp('10.0.0.1')).toBe(false)
  })

  it('rejects malformed service domains', () => {
    expect(isValidDomain('mysql.example.internal')).toBe(true)
    expect(isValidDomain('mysql_service.example.internal')).toBe(true)
    expect(isValidDomain('mysql.example.internal"')).toBe(false)
    expect(isValidDomain(' mysql.example.internal')).toBe(false)
    expect(isValidDomain('mysql example.internal')).toBe(false)
    expect(isValidDomain('')).toBe(false)
  })

  it('detects duplicate listener pairs', () => {
    const duplicate = findDuplicateListener(
      [
        {
          id: 'mysql',
          name: 'MySQL',
          group: '',
          domain: 'mysql.internal',
          remark: '',
          port: 3306,
          localIp: '127.77.0.10',
          tunnelId: 'default',
          sortOrder: 10,
          enabled: true,
        },
      ],
      {
        id: 'redis',
        port: 3306,
        localIp: '127.77.0.10',
      },
    )

    expect(duplicate?.id).toBe('mysql')
  })

  it('maps service states to labels and colors', () => {
    expect(serviceStateText('healthy')).toBe('可达')
    expect(serviceStateColor('error')).toBe('error')
    expect(serviceStateText()).toBe('未知')
  })

  it('keeps default settings valid', () => {
    expect(defaultSettings().currentProfileId).toBe('default')
  })
})
