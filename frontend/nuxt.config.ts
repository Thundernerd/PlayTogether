// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2025-07-15',
  devtools: { enabled: true },
  runtimeConfig: {
    public: {
      apiBase: '',
    },
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
