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
/* Tabletop Zine: warm cream paper, ink outlines, serif headings and hard
   offset shadows. The only theme; see issue #4. */
:root {
  color-scheme: light;

  /* Palette */
  --bg: #f4ecdc;
  --surface: #fffaf0;
  --surface-2: #ebe0c9;
  --surface-bright: #fff;
  --ink: #2b2118;
  --border: var(--ink);
  --text: var(--ink);
  --muted: #655441;
  /* Darker than the prototype's #85735f so placeholders clear 4.5:1 on white
     with margin (5.3:1). */
  --placeholder: #7a6853;
  --accent: #a8361a;
  --accent-2: #3d6628;
  --danger: #9e2218;
  /* A deep amber rather than a bright yellow, so warning text reads at 6:1+
     on the cream and tinted surfaces. */
  --warning: #7d5200;
  --highlight: #f2c14e;
  --highlight-tint: #fff4d6;

  /* Page chrome: dotted paper and a double rule under the header */
  --body-bg:
    radial-gradient(rgba(43, 33, 24, 0.09) 1px, transparent 1.2px) 0 0 / 18px 18px,
    var(--bg);
  --header-bg: var(--surface);
  --header-rule: 3px double var(--ink);

  /* Lines */
  --line: 2px solid var(--border);
  --line-thin: 1.5px solid var(--border);
  --line-dashed: 2px dashed var(--border);
  --line-heavy: 3px solid var(--border);

  /* Controls */
  --primary-bg: var(--accent);
  --primary-text: var(--surface);
  /* Accent, not --highlight: yellow on cream is 1.6:1, under the 3:1 a focus
     indicator needs. */
  --focus-ring: var(--accent);
  --focus-outline: var(--focus-outline);
  --selected-bg: var(--ink);
  --selected-text: var(--surface);

  /* Status */
  --success-text: var(--accent-2);
  --success-tint: #e5efd9;
  --warning-text: var(--warning);
  --warning-tint: var(--highlight-tint);
  --danger-text: var(--danger);
  --danger-tint: #fde4dc;
  --discount-bg: var(--accent);
  --discount-text: var(--surface);

  /* Ratings */
  --rating-average-text: var(--warning);
  --rating-filled: var(--accent);
  /* Darker than the prototype's #9c8a70: 3.9:1 instead of 3.2:1 on --surface,
     comfortably over the 3:1 icons need. */
  --rating-empty: #8f7c62;

  /* Overlays */
  --overlay-bg: rgba(43, 33, 24, 0.6);

  /* Hard offset shadows */
  --shadow-sm: 2px 2px 0 var(--ink);
  --shadow-btn: 3px 3px 0 var(--ink);
  --shadow-btn-hover: 4px 4px 0 var(--ink);
  --shadow-btn-active: 1px 1px 0 var(--ink);
  --shadow-tile: 4px 4px 0 var(--ink);
  --shadow-tile-hover: 6px 6px 0 var(--ink);
  --shadow-card: 5px 5px 0 var(--ink);
  --shadow-modal: 10px 10px 0 var(--ink);

  /* Media (always dark, whatever the theme) */
  --media-letterbox: var(--ink);
  --media-scrim: rgba(0, 0, 0, 0.35);
  --on-media: #fff;

  /* Radii */
  --radius-xs: 2px;
  --radius-sm: 4px;
  --radius-pill: 999px;

  /* Typography */
  --font-body: Karla, 'Helvetica Neue', Helvetica, Arial, sans-serif;
  --font-heading: Fraunces, Georgia, serif;
}

* {
  box-sizing: border-box;
}

::selection {
  background: var(--highlight);
  color: var(--text);
}

body {
  margin: 0;
  font-family: var(--font-body);
  font-size: 16.5px;
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
  font-weight: 800;
  letter-spacing: -0.01em;
}

h3,
h4,
h5,
h6 {
  font-weight: 600;
}

h1 {
  font-size: 2.25rem;
}

a {
  color: var(--accent);
  text-decoration: underline;
  text-decoration-thickness: 2px;
  text-underline-offset: 3px;
}

input[type='checkbox'] {
  accent-color: var(--accent);
}

.app-shell {
  min-height: 100vh;
}

.app-header {
  border-bottom: var(--header-rule);
  padding: 1rem 1.5rem;
  background: var(--header-bg);
}

.brand {
  color: var(--text);
  font-size: 1.5rem;
  text-decoration: none;
}

.brand::before {
  content: '♣ ';
  color: var(--accent);
}

.app-main {
  max-width: 1200px;
  margin: 0 auto;
  padding: 2rem 1.5rem 4rem;
}

.card {
  background: var(--surface);
  border: var(--line);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-card);
  padding: 1.5rem;
}

.btn {
  appearance: none;
  border: var(--line);
  background: var(--surface);
  color: var(--text);
  border-radius: var(--radius-sm);
  box-shadow: var(--shadow-btn);
  padding: 0.625rem 1rem;
  font: inherit;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.1s, box-shadow 0.1s;
}

.btn:hover:not(:disabled) {
  transform: translate(-1px, -1px);
  box-shadow: var(--shadow-btn-hover);
}

.btn:active:not(:disabled) {
  transform: translate(2px, 2px);
  box-shadow: var(--shadow-btn-active);
}

.btn:focus-visible {
  outline: var(--focus-outline);
  outline-offset: 2px;
}

.btn:disabled {
  opacity: 0.55;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary-bg);
  color: var(--primary-text);
}

.input {
  width: 100%;
  border: var(--line);
  background: var(--surface-bright);
  color: var(--text);
  border-radius: var(--radius-sm);
  padding: 0.625rem 0.75rem;
  font: inherit;
}

.input::placeholder {
  color: var(--placeholder);
}

.input:focus {
  outline: var(--focus-outline);
  outline-offset: 1px;
}

.label {
  display: block;
  font-size: 0.75rem;
  font-weight: 700;
  letter-spacing: 0.08em;
  text-transform: uppercase;
  color: var(--muted);
  margin-bottom: 0.375rem;
}

.grid-cards {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 1.25rem;
}

.section-title {
  margin: 0 0 1rem;
  font-size: 1.5rem;
}

.section-subtitle {
  margin: -0.5rem 0 1rem;
  color: var(--muted);
  font-size: 0.9375rem;
}

/* Steam tags on game cards, shared rows and the detail modal; each sets its
   own size. */
.tag {
  border: var(--line-thin);
  border-radius: var(--radius-xs);
  background: transparent;
  color: var(--text);
  text-transform: lowercase;
}

.error-message {
  margin: 0;
  padding: 0.5rem 0.75rem;
  border: var(--line);
  background: var(--danger-tint);
  color: var(--danger-text);
  font-weight: 700;
}

@media (prefers-reduced-motion: reduce) {
  .btn {
    transition: none;
  }
}
</style>
