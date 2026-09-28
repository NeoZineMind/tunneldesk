import { describe, expect, it } from 'vitest'
import { normalizeRouteAction, omitRouteAction } from '@/shared/domain/routeAction'

describe('route actions', () => {
  it('accepts only declared single-use actions', () => {
    expect(normalizeRouteAction('create', ['create'] as const)).toBe('create')
    expect(normalizeRouteAction(['import'], ['import'] as const)).toBe('import')
    expect(normalizeRouteAction('unknown', ['create', 'import'] as const)).toBeNull()
    expect(normalizeRouteAction(undefined, ['create'] as const)).toBeNull()
  })

  it('removes action while preserving unrelated query values', () => {
    expect(omitRouteAction({ action: 'create', source: 'overview' })).toEqual({ source: 'overview' })
  })
})
