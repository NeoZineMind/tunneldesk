import type { RouteRecordRaw } from 'vue-router'
import AppShell from '@/layouts/AppShell.vue'

export type RouteKey = 'overview' | 'tunnels' | 'services' | 'settings' | 'diagnostics' | 'logs'

const overviewPage = () => import('@/features/overview/OverviewPage.vue')
const tunnelsPage = () => import('@/features/tunnels/TunnelsPage.vue')
const servicesPage = () => import('@/features/services/ServicesPage.vue')
const settingsPage = () => import('@/features/settings/SettingsPage.vue')
const diagnosticsPage = () => import('@/features/diagnostics/DiagnosticsPage.vue')
const logsPage = () => import('@/features/logs/LogsPage.vue')

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    component: AppShell,
    redirect: '/overview',
    children: [
      { path: 'overview', name: 'overview', component: overviewPage, meta: { title: '总览' } },
      { path: 'tunnels', name: 'tunnels', component: tunnelsPage, meta: { title: '隧道' } },
      { path: 'services', name: 'services', component: servicesPage, meta: { title: '服务' } },
      { path: 'settings', name: 'settings', component: settingsPage, meta: { title: '设置' } },
      {
        path: 'diagnostics',
        name: 'diagnostics',
        component: diagnosticsPage,
        meta: { title: '诊断' },
      },
      { path: 'logs', name: 'logs', component: logsPage, meta: { title: '日志' } },
    ],
  },
]
