<template>
  <div class="app-shell">
    <header class="app-header">
      <NuxtLink to="/" class="brand">PlayWishlist</NuxtLink>
    </header>
    <main class="app-main">
      <NuxtPage />
    </main>
  </div>
</template>

<style>
:root {
  color-scheme: dark;
  --bg: #0f1419;
  --surface: #1a2332;
  --surface-2: #243044;
  --border: #334155;
  --text: #e2e8f0;
  --muted: #94a3b8;
  --accent: #38bdf8;
  --accent-2: #22c55e;
  --danger: #f87171;
  --warning: #fbbf24;
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: Inter, system-ui, -apple-system, sans-serif;
  background: radial-gradient(circle at top, #172033 0%, var(--bg) 55%);
  color: var(--text);
  min-height: 100vh;
}

a {
  color: var(--accent);
  text-decoration: none;
}

.app-shell {
  min-height: 100vh;
}

.app-header {
  border-bottom: 1px solid var(--border);
  padding: 1rem 1.5rem;
  backdrop-filter: blur(8px);
  background: rgba(15, 20, 25, 0.8);
}

.brand {
  font-weight: 700;
  font-size: 1.125rem;
  letter-spacing: 0.02em;
}

.app-main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
}

.card {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 12px;
  padding: 1.25rem;
}

.btn {
  appearance: none;
  border: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--text);
  border-radius: 8px;
  padding: 0.625rem 1rem;
  font: inherit;
  cursor: pointer;
  transition: border-color 0.15s, background 0.15s;
}

.btn:hover:not(:disabled) {
  border-color: var(--accent);
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-primary {
  background: linear-gradient(135deg, #0ea5e9, #0284c7);
  border-color: transparent;
  color: white;
  font-weight: 600;
}

.input {
  width: 100%;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  border-radius: 8px;
  padding: 0.625rem 0.75rem;
  font: inherit;
}

.input:focus {
  outline: 2px solid rgba(56, 189, 248, 0.35);
  border-color: var(--accent);
}

.label {
  display: block;
  font-size: 0.875rem;
  color: var(--muted);
  margin-bottom: 0.375rem;
}

.grid-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1rem;
}

.section-title {
  margin: 0 0 1rem;
  font-size: 1.25rem;
}

.section-subtitle {
  margin: -0.5rem 0 1rem;
  color: var(--muted);
  font-size: 0.9375rem;
}
</style>
