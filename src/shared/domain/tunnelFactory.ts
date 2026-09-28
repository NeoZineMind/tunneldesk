import type { ServiceConfig, TunnelConfig } from '@/shared/types'
import { defaultSsh } from './defaults'

export function slugify(value: string): string {
  return value
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-|-$/g, '')
}

export function nextTunnelId(tunnels: Pick<TunnelConfig, 'id'>[]): string {
  const used = new Set(tunnels.map((tunnel) => tunnel.id))
  for (let index = 1; index < 100; index += 1) {
    const candidate = `tunnel-${index}`
    if (!used.has(candidate)) return candidate
  }
  return `tunnel-${Date.now()}`
}

export function createTunnel(tunnels: TunnelConfig[]): TunnelConfig {
  const id = nextTunnelId(tunnels)
  return {
    id,
    name: `Tunnel ${tunnels.length + 1}`,
    enabled: true,
    ssh: defaultSsh(),
  }
}

export function nextLocalIp(services: Pick<ServiceConfig, 'localIp'>[]): string {
  const used = new Set(services.map((service) => service.localIp))
  for (let index = 10; index < 250; index += 1) {
    const candidate = `127.77.0.${index}`
    if (!used.has(candidate)) return candidate
  }
  return '127.77.1.10'
}

export function tunnelName(tunnels: Pick<TunnelConfig, 'id' | 'name'>[], tunnelId: string): string {
  return tunnels.find((tunnel) => tunnel.id === tunnelId)?.name || tunnelId
}
