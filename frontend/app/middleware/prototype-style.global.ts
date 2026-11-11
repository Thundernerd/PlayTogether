// PROTOTYPE (issue #1): carry ?style= across in-app navigation so the chosen
// visual style survives clicks on NuxtLinks and router.push calls. Dev only.
import { resolvePrototypeStyle } from '~/utils/prototypeStyles'

export default defineNuxtRouteMiddleware((to, from) => {
  if (!import.meta.dev) return
  if (to.query.style != null) return
  const style = resolvePrototypeStyle(from.query.style)
  if (style === 'current') return
  return navigateTo({ path: to.path, query: { ...to.query, style }, hash: to.hash }, { replace: true })
})
