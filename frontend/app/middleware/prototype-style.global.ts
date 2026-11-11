// PROTOTYPE (issue #1): carry ?style= across in-app navigation so the chosen
// visual style survives clicks on NuxtLinks and router.push calls. Dev only.
export default defineNuxtRouteMiddleware((to, from) => {
  if (!import.meta.dev) return
  const style = from.query.style
  if (typeof style !== 'string' || to.query.style != null) return
  return navigateTo({ path: to.path, query: { ...to.query, style }, hash: to.hash }, { replace: true })
})
