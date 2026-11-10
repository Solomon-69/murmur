import { commands, type Theme } from "@/bindings";

/**
 * Appearance theme handling.
 *
 * murmur ships a light, a dark and a true-black palette (see `theme.css`). This
 * module lets the user pick which one is used instead of always following
 * the OS:
 *  - `system` removes the override so the `prefers-color-scheme` media query
 *    governs (the historical behaviour).
 *  - `light` / `dark` / `black` set `data-theme` on the document root, whose
 *    higher-specificity CSS selectors win over the media query.
 *
 * `black` is stored as `theme: "dark"` plus `black_theme: true`, so older
 * builds that only know light/dark still read a sensible value.
 *
 * The choice is persisted in `AppSettings` (source of truth) and mirrored to
 * localStorage so it can be applied synchronously on boot, before React mounts,
 * avoiding a flash of the wrong palette.
 */

export type AppTheme = Theme | "black";

export const THEME_STORAGE_KEY = "murmur.theme";

export const THEME_OPTIONS: AppTheme[] = ["system", "light", "dark", "black"];

const isAppTheme = (value: unknown): value is AppTheme =>
  value === "system" ||
  value === "light" ||
  value === "dark" ||
  value === "black";

/** The theme the user sees, from the two stored settings. */
export const resolveTheme = (
  theme: Theme | undefined,
  black: boolean | undefined,
): AppTheme => (black ? "black" : (theme ?? "system"));

/** Apply a theme to the document root and remember it for the next launch. */
export const applyTheme = (theme: AppTheme): void => {
  const root = document.documentElement;
  if (theme === "system") {
    delete root.dataset.theme;
  } else {
    root.dataset.theme = theme;
  }
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch {
    // localStorage may be unavailable (e.g. private mode); the setting still
    // persists in AppSettings, so this only costs a one-frame flash on boot.
  }
};

/** Read the last-applied theme for synchronous boot-time application. */
export const getStoredTheme = (): AppTheme => {
  try {
    // Falls back to the key used before the rename to murmur.
    const stored =
      localStorage.getItem(THEME_STORAGE_KEY) ??
      localStorage.getItem("handy.theme");
    if (isAppTheme(stored)) return stored;
  } catch {
    // ignore
  }
  return "system";
};

/** Apply the persisted theme from AppSettings (the source of truth). */
export const syncThemeFromSettings = async (): Promise<void> => {
  try {
    const result = await commands.getAppSettings();
    if (result.status === "ok") {
      applyTheme(resolveTheme(result.data.theme, result.data.black_theme));
    }
  } catch (e) {
    console.warn("Failed to sync theme from settings:", e);
  }
};
