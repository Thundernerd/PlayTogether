// PROTOTYPE (issue #1): three app-wide visual styles, switchable with ?style=.
// Throwaway code: lives on a throwaway branch, never merged to main.
// Everything that reads this is gated on import.meta.dev.

export type PrototypeStyleKey = 'current' | 'a' | 'b' | 'c'

export interface PrototypeStyle {
  key: PrototypeStyleKey
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
      'For friends who treat game night like a board-game evening. Warm cream paper, ink outlines, serif headings and chunky offset shadows make it feel tactile, cosy and hand-made.',
  },
  {
    key: 'b',
    name: 'LAN Terminal',
    rationale:
      'For the PC crowd who live in Discord and config files. A pitch-black HUD with acid-lime accents, monospace everything, square corners and tight density feels fast, technical and a bit loud.',
  },
  {
    key: 'c',
    name: 'Couch Co-op',
    rationale:
      'For casual groups and non-gamer partners. Soft lavender pastels, rounded friendly type, big radii, airy spacing and gentle shadows keep it calm, welcoming and low-stakes.',
  },
]

export function resolvePrototypeStyle(value: unknown): PrototypeStyleKey {
  return PROTOTYPE_STYLES.find((s) => s.key === value)?.key ?? 'current'
}
