import type { RuleObject } from 'ant-design-vue/es/form'
import type { ServiceConfig } from '@/shared/types'

export function isLoopbackIp(value: string): boolean {
  return /^127(?:\.(?:\d|[1-9]\d|1\d\d|2[0-4]\d|25[0-5])){3}$/.test(value)
}

export function isValidPort(value: number): boolean {
  return Number.isInteger(Number(value)) && Number(value) >= 1 && Number(value) <= 65535
}

export function isValidDomain(value: string): boolean {
  return value.length <= 253 && /^[a-zA-Z0-9._-]+$/.test(value)
}

export function findDuplicateListener(
  services: ServiceConfig[],
  candidate: Pick<ServiceConfig, 'localIp' | 'port' | 'id'>,
): ServiceConfig | undefined {
  return services.find((service) => {
    return service.id !== candidate.id && service.localIp === candidate.localIp && Number(service.port) === Number(candidate.port)
  })
}

export function requiredRule(message: string): RuleObject {
  return { required: true, message, trigger: 'blur' }
}

export function portRule(): RuleObject {
  return {
    type: 'number',
    validator: async (_rule, value: number) => {
      if (!isValidPort(Number(value))) {
        return Promise.reject(new Error('端口必须在 1-65535 之间'))
      }
      return Promise.resolve()
    },
    trigger: 'change',
  }
}

export function loopbackIpRule(): RuleObject {
  return {
    validator: async (_rule, value: string) => {
      if (!value || !isLoopbackIp(value)) {
        return Promise.reject(new Error('本地 IP 必须是 127.x.x.x 回环地址'))
      }
      return Promise.resolve()
    },
    trigger: 'blur',
  }
}

export function domainRule(): RuleObject {
  return {
    validator: async (_rule, value: string) => {
      if (!value) {
        return Promise.reject(new Error('请填写真实域名'))
      }
      if (!isValidDomain(value)) {
        return Promise.reject(new Error('域名只能包含字母、数字、点、连字符和下划线，且不能包含空格或引号'))
      }
      return Promise.resolve()
    },
    trigger: 'blur',
  }
}
