/**
 * Accent colours for Settings > Appearance > Colour.
 *
 * Keep in sync with:
 * - src-tauri/src/settings.rs OVERLAY_ACCENTS
 * - src/styles/theme.css :root[data-accent=...] (app UI, deeper variants)
 * - src/overlay/RecordingOverlay.css :root[data-accent=...] (bar, true hex)
 */

export interface AccentGroup {
  /** i18n key under settings.appearance.accent.groups, or null for Grey. */
  id: "neutral" | "earth" | "vibey" | "muted";
  colors: Record<string, string>;
}

export const ACCENT_GROUPS: AccentGroup[] = [
  { id: "neutral", colors: { default: "#4a4a50" } },
  {
    id: "earth",
    colors: {
      terracotta: "#c0643f",
      olive: "#6f7a3c",
      ochre: "#c48a2c",
      clay: "#9a5b47",
      moss: "#4f6b4a",
      sand: "#b39567",
    },
  },
  {
    id: "vibey",
    colors: {
      electric: "#3d5afe",
      coral: "#ff5a5f",
      violet: "#7c4dff",
      tangerine: "#ff8a1f",
      magenta: "#e83e9a",
      aqua: "#00b8c4",
    },
  },
  {
    id: "muted",
    colors: {
      sage: "#8fa58c",
      rose: "#c49a9a",
      slate: "#7f93a8",
      lilac: "#9e95b5",
      mushroom: "#a39587",
      fogteal: "#7fa3a0",
    },
  },
];

/** Every accent id mapped to its swatch colour. */
export const ACCENT_COLORS: Record<string, string> = Object.assign(
  {},
  ...ACCENT_GROUPS.map((g) => g.colors),
);

export const DEFAULT_ACCENT = "default";

/** Unknown or retired ids (pink, green, orange, purple, graphite) fall back to Grey. */
export function normalizeAccent(id: string | null | undefined): string {
  return id && Object.prototype.hasOwnProperty.call(ACCENT_COLORS, id)
    ? id
    : DEFAULT_ACCENT;
}

/** Shown as swatches in the main row; everything else lives under "More colours". */
export const MAIN_ACCENTS = [
  "default",
  "clay",
  "moss",
  "sand",
  "slate",
  "rose",
] as const;

const MAIN_SET = new Set<string>(MAIN_ACCENTS);

/** Earth / Vibey / Muted groups without the main-row colours. */
export const MORE_ACCENT_GROUPS: AccentGroup[] = ACCENT_GROUPS.filter(
  (g) => g.id !== "neutral",
)
  .map((g) => ({
    id: g.id,
    colors: Object.fromEntries(
      Object.entries(g.colors).filter(([id]) => !MAIN_SET.has(id)),
    ),
  }))
  .filter((g) => Object.keys(g.colors).length > 0);

export function isMainAccent(id: string): boolean {
  return MAIN_SET.has(id);
}
