import { createRouter, createWebHashHistory } from 'vue-router'
import { routes } from './routes'

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
})

router.afterEach((to) => {
  const pageTitle = String(to.meta.title || 'TunnelDesk')
  document.title = pageTitle === 'TunnelDesk' ? 'TunnelDesk' : `${pageTitle} - TunnelDesk`
})
