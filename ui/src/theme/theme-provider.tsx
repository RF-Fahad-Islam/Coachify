import { createContext, useEffect, useState, type ReactNode } from "react";
export { useTheme } from "./use-theme";

export type Palette = "orange" | "blue" | "emerald" | "violet" | "rose" | "slate";
export type Mode = "light" | "dark" | "system";
export type FontFamily = "plus-jakarta" | "inter" | "outfit" | "system";
export type Density = "compact" | "comfortable" | "spacious";
export type UiScale = 90 | 100 | 110 | 125;
export type Radius = "sharp" | "rounded" | "soft";

export interface ThemePreferences {
  palette: Palette;
  mode: Mode;
  fontFamily: FontFamily;
  density: Density;
  uiScale: UiScale;
  radius: Radius;
}

export interface ThemeContextValue extends ThemePreferences {
  setPalette: (palette: Palette) => void;
  setMode: (mode: Mode) => void;
  setFontFamily: (font: FontFamily) => void;
  setDensity: (density: Density) => void;
  setUiScale: (scale: UiScale) => void;
  setRadius: (radius: Radius) => void;
  resetDefaults: () => void;
}

export const DEFAULT_THEME_PREFERENCES: ThemePreferences = {
  palette: "orange",
  mode: "light",
  fontFamily: "system",
  density: "spacious",
  uiScale: 100,
  radius: "rounded",
};

const STORAGE_KEY = "coachify_ui_theme_prefs";

export const ThemeContext = createContext<ThemeContextValue | null>(null);

function loadStoredPreferences(): ThemePreferences {
  if (typeof window === "undefined") return DEFAULT_THEME_PREFERENCES;
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return DEFAULT_THEME_PREFERENCES;
    const parsed = JSON.parse(raw) as Partial<ThemePreferences>;
    return {
      ...DEFAULT_THEME_PREFERENCES,
      ...parsed,
    };
  } catch {
    return DEFAULT_THEME_PREFERENCES;
  }
}

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [prefs, setPrefs] = useState<ThemePreferences>(loadStoredPreferences);

  useEffect(() => {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
    } catch {
      // Ignored if storage is unavailable
    }

    const root = document.documentElement;

    // Apply Palette
    root.setAttribute("data-palette", prefs.palette);

    // Apply Light/Dark Mode
    const isDark =
      prefs.mode === "dark" ||
      (prefs.mode === "system" &&
        window.matchMedia("(prefers-color-scheme: dark)").matches);

    if (isDark) {
      root.classList.add("dark");
    } else {
      root.classList.remove("dark");
    }

    // Apply Typography
    root.setAttribute("data-font", prefs.fontFamily);

    // Apply Information Density
    root.setAttribute("data-density", prefs.density);

    // Apply Border Radius
    root.setAttribute("data-radius", prefs.radius);

    // Apply UI Zoom Scale
    root.style.setProperty("--ui-scale", String(prefs.uiScale / 100));
    (root.style as unknown as Record<string, string>).zoom = String(prefs.uiScale / 100);
  }, [prefs]);

  // Handle system color scheme changes if mode is set to 'system'
  useEffect(() => {
    if (prefs.mode !== "system") return;

    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const handler = (e: MediaQueryListEvent) => {
      const root = document.documentElement;
      if (e.matches) {
        root.classList.add("dark");
      } else {
        root.classList.remove("dark");
      }
    };

    media.addEventListener("change", handler);
    return () => media.removeEventListener("change", handler);
  }, [prefs.mode]);

  const setPalette = (palette: Palette) => setPrefs((p) => ({ ...p, palette }));
  const setMode = (mode: Mode) => setPrefs((p) => ({ ...p, mode }));
  const setFontFamily = (fontFamily: FontFamily) =>
    setPrefs((p) => ({ ...p, fontFamily }));
  const setDensity = (density: Density) => setPrefs((p) => ({ ...p, density }));
  const setUiScale = (uiScale: UiScale) => setPrefs((p) => ({ ...p, uiScale }));
  const setRadius = (radius: Radius) => setPrefs((p) => ({ ...p, radius }));
  const resetDefaults = () => setPrefs(DEFAULT_THEME_PREFERENCES);

  const value: ThemeContextValue = {
    ...prefs,
    setPalette,
    setMode,
    setFontFamily,
    setDensity,
    setUiScale,
    setRadius,
    resetDefaults,
  };

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}
