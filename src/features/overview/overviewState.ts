import type { AppSettings, AppStatus, ServiceConfig, ServiceProfile } from '@/shared/types'
import { validateProfileStart } from '@/shared/domain/startupValidation'
import { displayServiceStatus } from '@/shared/domain/serviceStatus'

export type OverviewTone = 'success' | 'warning' | 'danger' | 'neutral'

export interface OverviewSummary {
  title: string
  description: string
  tone: OverviewTone
  enabledServices: number
  runningTunnels: number
  healthyServices: number
  abnormalServices: number
  checkingServices: number
  unknownServices: number
  issues: string[]
}

export function buildOverviewSummary(profile: ServiceProfile, settings: AppSettings, status: AppStatus): OverviewSummary {
  const enabled = profile.services.filter((service) => service.enabled)
  const states = enabled.map((service) => displayServiceStatus(service, status.running, status.services).state)
  const count = (state: string) => status.running ? states.filter((value) => value === state).length : 0
  const healthyServices = count('healthy')
  const abnormalServices = count('error') + count('stopped')
  const checkingServices = count('checking')
  const unknownServices = count('unknown')
  const issues: string[] = []
  if (status.running) {
    if (!status.hostsBlockPresent) issues.push('hosts 映射缺失')
    const missing = [...new Set(enabled.map((service) => service.tunnelId))]
      .filter((id) => !status.runningTunnelIds.includes(id))
      .map((id) => settings.tunnels.find((tunnel) => tunnel.id === id)?.name || id)
    if (missing.length) issues.push(`隧道未运行：${missing.join('、')}`)
  } else {
    issues.push(...validateProfileStart(profile, settings, status).map((issue) => issue.message))
  }
  const parts = [`${healthyServices}/${enabled.length} 服务可达`]
  if (abnormalServices) parts.push(`${abnormalServices} 项异常`)
  if (checkingServices) parts.push(`${checkingServices} 项检查中`)
  if (unknownServices) parts.push(`${unknownServices} 项未知`)
  const tone: OverviewTone = !status.running ? 'neutral'
    : abnormalServices || issues.length ? 'danger'
      : checkingServices || unknownServices ? 'warning' : 'success'
  return {
    title: status.running ? '运行中' : '已停止',
    description: status.running ? parts.join(' · ') : `已启用 ${enabled.length} 项服务`,
    tone, enabledServices: enabled.length, healthyServices, abnormalServices, checkingServices, unknownServices,
    runningTunnels: status.runningTunnelIds.length, issues,
  }
}

export function overviewServices(services: ServiceConfig[], status: AppStatus): ServiceConfig[] {
  if (!status.running) return services
  const abnormal = (service: ServiceConfig) => {
    const state = displayServiceStatus(service, true, status.services).state
    return service.enabled && (state === 'error' || state === 'stopped') ? 1 : 0
  }
  return [...services].sort((left, right) => abnormal(right) - abnormal(left))
}
