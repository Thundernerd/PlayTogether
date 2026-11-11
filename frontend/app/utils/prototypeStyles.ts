// PROTOTYPE (issue #1): three app-wide visual styles, switchable with ?style=.
// Throwaway code: lives on a throwaway branch, never merged to main.
// Everything that reads this is gated on import.meta.dev.

export interface PrototypeStyle {
  key: string
  name: string
  rationale: string
}

export const PROTOTYPE_STYLES: PrototypeStyle[] = [
  {
    key: 'current',
    name: 'Current (dark slate)',
    rationale: "Today's look: dark slate background with sky-blue accents.",
  },
  {
    key: 'a',
    name: 'Tabletop Zine',
    rationale:
      'For friends who treat game night like a board-game evening: warm cream paper, ink outlines, serif headings and chunky offset shadows. Tactile, cosy, hand-made.',
  },
  {
    key: 'b',
    name: 'LAN Terminal',
    rationale:
      'For the PC crowd who live in Discord and config files: pitch-black HUD, acid-lime accents, monospace everything, square corners and tight density. Fast, technical, a bit loud.',
  },
  {
    key: 'c',
    name: 'Couch Co-op',
    rationale:
      'For casual groups and non-gamer partners: soft lavender pastels, rounded friendly type, big radii, airy spacing and gentle shadows. Calm, welcoming, low-stakes.',
  },
]

export function resolvePrototypeStyle(value: unknown): string {
  return PROTOTYPE_STYLES.some((s) => s.key === value) ? (value as string) : 'current'
}
