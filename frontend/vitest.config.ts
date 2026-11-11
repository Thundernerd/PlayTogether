import { defineVitestConfig } from '@nuxt/test-utils/config'

// Runs tests inside the Nuxt environment (auto-imports, `~` alias) with happy-dom,
// so DOM-dependent code such as the bbcode sanitiser exercises its browser path.
export default defineVitestConfig({
  test: {
    environment: 'nuxt',
    environmentOptions: {
      nuxt: {
        domEnvironment: 'happy-dom',
      },
    },
    include: ['test/**/*.test.ts'],
  },
})
