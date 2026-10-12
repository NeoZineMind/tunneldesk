import type { ServiceConfig, ServiceState, ServiceStatus } from '@/shared/types'

export function displayServiceStatus(service: ServiceConfig, running: boolean, statuses: ServiceStatus[]): ServiceStatus {
  if (!service.enabled) return { serviceId: service.id, state: 'disabled', message: '配置未启用' }
  const status = serviceStatusFor(service.id, statuses)
  if (!running) return status?.state === 'error' || status?.state === 'skipped' ? status : { serviceId: service.id, state: 'stopped', message: '当前 Profile 已停止' }
  // An enabled service reported as disabled is inconsistent with the current configuration.
  if (!status || status.state === 'disabled') return { serviceId: service.id, state: 'unknown', message: '暂无服务状态，请刷新或查看诊断' }
  return status
}

export function runningLabel(running: boolean, outcome?: 'complete' | 'partial' | 'failed'): string {
  return running ? outcome === 'partial' ? '部分运行' : '运行中' : outcome === 'failed' ? '启动失败' : '已停止'
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
  if (state === 'skipped') return '已跳过'
  return '未知'
}

export function serviceStateColor(state?: ServiceState): string {
  if (state === 'healthy') return 'success'
  if (state === 'checking') return 'warning'
  if (state === 'error') return 'error'
  if (state === 'skipped') return 'warning'
  if (state === 'disabled' || state === 'stopped') return 'default'
  return 'warning'
}
