import { describe, expect, it } from 'vitest'
import { buildOverviewSummary, overviewServices } from '@/features/overview/overviewState'
import { defaultProfiles, defaultServiceDraft, defaultSettings, defaultStatus } from '@/shared/domain/defaults'

function readyFixture() {
  const settings = defaultSettings()
  settings.tunnels[0].ssh.host = 'ssh.example.com'
  settings.tunnels[0].ssh.username = 'tunnel'
  const profile = defaultProfiles().profiles[0]
  profile.services.push({
    ...defaultServiceDraft(),
    id: 'mysql',
    name: 'MySQL',
    domain: 'mysql.local',
    port: 3306,
  })
  const status = defaultStatus()
  status.privilege = {
    ...status.privilege,
    process: 'root',
    hostsAccess: 'direct',
    canModifyHosts: true,
    message: 'Hosts can be modified directly',
  }
  return { settings, profile, status }
}

describe('overview state', () => {
  it('shows stopped counts without reporting stale reachability', () => {
    const { profile, settings, status } = readyFixture()
    status.services = [{ serviceId: 'mysql', state: 'healthy', message: 'old result' }]
    expect(buildOverviewSummary(profile, settings, status)).toMatchObject({ title: '已停止', description: '已启用 1 项服务', healthyServices: 0, abnormalServices: 0, tone: 'neutral' })
  })

  it('handles an empty profile without stale metrics', () => {
    const summary = buildOverviewSummary(defaultProfiles().profiles[0], defaultSettings(), defaultStatus())
    expect(summary.title).toBe('已停止')
    expect(summary.enabledServices).toBe(0)
  })

  it('counts only enabled services in the running summary', () => {
    const { profile, settings, status } = readyFixture()
    status.running = true
    status.runningTunnelIds = ['default']
    status.hostsBlockPresent = true
    profile.services.push({ ...profile.services[0], id: 'disabled', enabled: false })
    status.services = [{ serviceId: 'mysql', state: 'healthy', message: 'reachable' }, { serviceId: 'disabled', state: 'error', message: 'ignored' }]
    expect(buildOverviewSummary(profile, settings, status)).toMatchObject({ title: '运行中', description: '1/1 服务可达', healthyServices: 1, abnormalServices: 0, tone: 'success', issues: [] })
  })

  it.each([
    ['healthy', 1, 0, 0, 0, 'success'],
    ['error', 0, 1, 0, 0, 'danger'],
    ['stopped', 0, 1, 0, 0, 'danger'],
    ['checking', 0, 0, 1, 0, 'warning'],
    ['disabled', 0, 0, 0, 1, 'warning'],
    ['missing', 0, 0, 0, 1, 'warning'],
  ] as const)('distinguishes %s from explicit errors', (state, healthyServices, abnormalServices, checkingServices, unknownServices, tone) => {
    const { profile, settings, status } = readyFixture()
    status.running = true
    status.runningTunnelIds = ['default']
    status.hostsBlockPresent = true
    status.services = state === 'missing' ? [] : [{ serviceId: 'mysql', state, message: state }]
    expect(buildOverviewSummary(profile, settings, status)).toMatchObject({ healthyServices, abnormalServices, checkingServices, unknownServices, tone })
  })

  it('reports hosts and missing tunnel problems with concrete names', () => {
    const { profile, settings, status } = readyFixture()
    status.running = true
    const summary = buildOverviewSummary(profile, settings, status)
    expect(summary.issues).toEqual(['hosts 映射缺失', '隧道未运行：Default Tunnel'])
    expect(summary.tone).toBe('danger')
  })

  it('puts failed services first without reordering the rest or mutating configuration', () => {
    const { profile, status } = readyFixture()
    const services = ['a', 'b', 'c', 'd'].map((id) => ({ ...profile.services[0], id }))
    status.running = true
    status.services = [{ serviceId: 'c', state: 'error', message: 'refused' }]
    expect(overviewServices(services, status).map((item) => item.id)).toEqual(['c', 'a', 'b', 'd'])
    expect(services.map((item) => item.id)).toEqual(['a', 'b', 'c', 'd'])
    status.running = false
    expect(overviewServices(services, status)).toEqual(services)
  })
})
