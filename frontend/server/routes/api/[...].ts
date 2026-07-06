import { proxyRequest } from 'h3'

export default defineEventHandler((event) => {
  const backend = (process.env.BACKEND_URL || 'http://127.0.0.1:8080').replace(/\/$/, '')
  return proxyRequest(event, `${backend}${event.path}`)
})
