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
      // Tabletop Zine fonts: Fraunces for headings, Karla for body text.
      // Fallbacks live in --font-heading / --font-body in app.vue.
      link: [
        { rel: 'preconnect', href: 'https://fonts.googleapis.com' },
        { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' },
        {
          rel: 'stylesheet',
          href: 'https://fonts.googleapis.com/css2?family=Fraunces:opsz,wght@9..144,600;9..144,800&family=Karla:wght@400;600;700&display=swap',
        },
      ],
    },
  },
})
