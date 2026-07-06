// https://nuxt.com/docs/api/configuration/nuxt-config
const apiProxy = process.env.NUXT_API_PROXY || 'http://127.0.0.1:8080'

export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  runtimeConfig: {
    public: {
      // Empty = same-origin requests (works behind a reverse proxy).
      // Set NUXT_PUBLIC_API_BASE only when the API is on a different public URL.
      apiBase: process.env.NUXT_PUBLIC_API_BASE ?? '',
    },
  },
  routeRules: {
    '/api/**': { proxy: `${apiProxy}/api/**` },
  },
  app: {
    head: {
      title: 'PlayTogether',
      meta: [
        { name: 'description', content: 'Compare Steam wishlists and pick games to play together' },
      ],
    },
  },
})
