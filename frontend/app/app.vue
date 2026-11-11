<template>
  <div class="app-shell">
    <header class="app-header">
      <NuxtLink to="/" class="brand">PlayTogether</NuxtLink>
    </header>
    <main class="app-main">
      <NuxtPage />
    </main>
  </div>
</template>

<style>
:root {
  color-scheme: dark;

  /* Palette */
  --bg: #0f1419;
  --surface: #1a2332;
  --surface-2: #243044;
  --border: #334155;
  --text: #e2e8f0;
  --text-secondary: #cbd5e1;
  --muted: #94a3b8;
  --accent: #38bdf8;
  --accent-2: #22c55e;
  --danger: #f87171;
  --warning: #fbbf24;

  /* Page chrome */
  --body-bg: radial-gradient(circle at top, #172033 0%, var(--bg) 55%);
  --header-bg: rgba(15, 20, 25, 0.8);

  /* Controls */
  --primary-bg: linear-gradient(135deg, #0ea5e9, #0284c7);
  --primary-text: white;
  --focus-ring: rgba(56, 189, 248, 0.35);

  /* Accent states (active nav pill, running sync, floating button) */
  --accent-border: rgba(56, 189, 248, 0.5);
  --accent-text: #7dd3fc;
  --active-border: rgba(56, 189, 248, 0.55);
  --active-bg: rgba(56, 189, 248, 0.12);
  --active-text: #e0f2fe;

  /* Status */
  --success-border: rgba(34, 197, 94, 0.5);
  --success-text: #86efac;
  --danger-border: rgba(248, 113, 113, 0.5);
  --danger-text: #fca5a5;
  --danger-tint: rgba(248, 113, 113, 0.1);
  --discount-bg: #166534;
  --discount-text: #dcfce7;

  /* Ratings */
  --rating-avg: #fde68a;
  --rating-filled: #fbbf24;
  --rating-empty: #64748b;

  /* Overlays and elevation */
  --overlay-bg: rgba(2, 6, 23, 0.78);
  --overlay-control-bg: rgba(15, 20, 25, 0.85);
  --floating-bg: rgba(26, 35, 50, 0.95);
  --shadow-floating: 0 8px 24px rgba(0, 0, 0, 0.35);
  --shadow-modal: 0 24px 80px rgba(0, 0, 0, 0.45);

  /* Media (always dark, whatever the theme) */
  --media-letterbox: #000;
  --media-scrim: rgba(0, 0, 0, 0.35);
  --on-media: #fff;

  /* Radii */
  --radius-xs: 4px;
  --radius-sm: 8px;
  --radius-md: 10px;
  --radius-lg: 12px;
  --radius-xl: 14px;
  --radius-pill: 999px;

  /* Typography */
  --font-body: Inter, system-ui, -apple-system, sans-serif;
  --font-heading: var(--font-body);
}

* {
  box-sizing: border-box;
}

body {
  margin: 0;
  font-family: var(--font-body);
  background: var(--body-bg);
  color: var(--text);
  min-height: 100vh;
}

h1,
h2,
h3,
h4,
h5,
h6,
.brand {
  font-family: var(--font-heading);
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
  background: var(--header-bg);
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
  border-radius: var(--radius-lg);
  padding: 1.25rem;
}

.btn {
  appearance: none;
  border: 1px solid var(--border);
  background: var(--surface-2);
  color: var(--text);
  border-radius: var(--radius-sm);
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
  background: var(--primary-bg);
  border-color: transparent;
  color: var(--primary-text);
  font-weight: 600;
}

.input {
  width: 100%;
  border: 1px solid var(--border);
  background: var(--bg);
  color: var(--text);
  border-radius: var(--radius-sm);
  padding: 0.625rem 0.75rem;
  font: inherit;
}

.input:focus {
  outline: 2px solid var(--focus-ring);
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
