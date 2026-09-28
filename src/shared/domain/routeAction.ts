export function normalizeRouteAction<T extends string>(
  value: unknown,
  allowed: readonly T[],
): T | null {
  const candidate = Array.isArray(value) ? value[0] : value
  return typeof candidate === 'string' && allowed.includes(candidate as T) ? (candidate as T) : null
}

export function omitRouteAction(query: LocationQuery): LocationQueryRaw {
  const rest = { ...query }
  delete rest.action
  return rest
}
import type { LocationQuery, LocationQueryRaw } from 'vue-router'
