import type { ServiceConfig, ServiceState, ServiceStatus } from '@/shared/types'

export function displayServiceStatus(service: ServiceConfig, running: boolean, statuses: ServiceStatus[]): ServiceStatus {
  if (!service.enabled) return { serviceId: service.id, state: 'disabled', message: '配置未启用' }
  if (!running) return { serviceId: service.id, state: 'stopped', message: '当前 Profile 已停止' }
  const status = serviceStatusFor(service.id, statuses)
  // An enabled service reported as disabled is inconsistent with the current configuration.
  if (!status || status.state === 'disabled') return { serviceId: service.id, state: 'unknown', message: '暂无服务状态，请刷新或查看诊断' }
  return status
}

export function runningLabel(running: boolean): string {
  return running ? '运行中' : '已停止'
}

export function serviceStatusFor(serviceId: string, statuses: ServiceStatus[]): ServiceStatus | undefined {
  return statuses.find((item) => item.serviceId === serviceId)
}

export function serviceStateText(state?: ServiceState): string {
  if (state === 'healthy') return '可达'
  if (state === 'disabled') return '未启用'
  if (state === 'stopped') return '已停止'
  if (state === 'checking') return '检查中'
  if (state === 'error') return '异常'
  return '未知'
}

export function serviceStateColor(state?: ServiceState): string {
  if (state === 'healthy') return 'success'
  if (state === 'checking') return 'warning'
  if (state === 'error') return 'error'
  if (state === 'disabled' || state === 'stopped') return 'default'
  return 'warning'
}
