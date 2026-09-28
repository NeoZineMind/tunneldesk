import type { AppSettings, AppStatus, ServiceProfile } from '@/shared/types'
import { findDuplicateListener, isLoopbackIp, isValidDomain, isValidPort } from './validators'

export interface StartupValidationIssue {
  id: string
  message: string
}

export function validateProfileStart(
  profile: ServiceProfile,
  settings: AppSettings,
  status: AppStatus,
): StartupValidationIssue[] {
  const issues: StartupValidationIssue[] = []
  const enabledServices = profile.services.filter((service) => service.enabled)
  const tunnelsById = new Map(settings.tunnels.map((tunnel) => [tunnel.id, tunnel]))

  for (const service of enabledServices) {
    const label = service.name || service.id || '未命名服务'
    if (!service.name.trim()) {
      issues.push({ id: `${service.id}:name`, message: `${label} 缺少服务名` })
    }
    if (!service.domain.trim()) {
      issues.push({ id: `${service.id}:domain`, message: `${label} 缺少域名` })
    } else if (!isValidDomain(service.domain)) {
      issues.push({ id: `${service.id}:domain`, message: `${label} 的域名格式无效` })
    }
    if (!service.localIp.trim()) {
      issues.push({ id: `${service.id}:localIp:empty`, message: `${label} 缺少本地 IP` })
    } else if (!isLoopbackIp(service.localIp)) {
      issues.push({ id: `${service.id}:localIp`, message: `${label} 的本地 IP 必须是 127.x.x.x` })
    }
    if (!isValidPort(Number(service.port))) {
      issues.push({ id: `${service.id}:port`, message: `${label} 的端口必须在 1-65535 之间` })
    }
    if (!tunnelsById.has(service.tunnelId)) {
      issues.push({ id: `${service.id}:tunnel`, message: `${label} 引用了不存在的隧道：${service.tunnelId || '未选择'}` })
    }

    const duplicate = findDuplicateListener(enabledServices, service)
    if (duplicate) {
      issues.push({
        id: `${service.id}:duplicate-listener`,
        message: `${label} 的监听地址 ${service.localIp}:${service.port} 已被 ${duplicate.name || duplicate.id} 使用`,
      })
    }
  }

  const usedTunnelIds = new Set(enabledServices.map((service) => service.tunnelId).filter(Boolean))
  for (const tunnelId of usedTunnelIds) {
    const tunnel = tunnelsById.get(tunnelId)
    if (!tunnel) continue
    const label = tunnel.name || tunnel.id
    if (!tunnel.ssh.host.trim()) {
      issues.push({ id: `${tunnel.id}:ssh-host`, message: `${label} 缺少 SSH Host` })
    }
    if (!tunnel.ssh.username.trim()) {
      issues.push({ id: `${tunnel.id}:ssh-username`, message: `${label} 缺少 Username` })
    }
  }

  if (!status.privilege.canModifyHosts) {
    issues.push({ id: 'hosts-access', message: `hosts 权限不可用：${status.privilege.message}` })
  }

  return dedupeIssues(issues)
}

function dedupeIssues(issues: StartupValidationIssue[]) {
  const seen = new Set<string>()
  return issues.filter((issue) => {
    if (seen.has(issue.id)) return false
    seen.add(issue.id)
    return true
  })
}
