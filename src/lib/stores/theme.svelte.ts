export type ThemeId = "dark" | "light" | "high-contrast" | "monokai" | "solarized" | "custom";

export interface Theme {
  id: ThemeId;
  label: string;
  vars: Record<string, string>;
}

export const THEMES: Theme[] = [
  {
    id: "dark",
    label: "Parallax Dark",
    vars: {
      "--bg-void": "#090b10",
      "--bg-base": "#0d1117",
      "--bg-surface": "#161b22",
      "--bg-elevated": "#1c232e",
      "--bg-overlay": "#21293a",
      "--bg-input": "#131820",
      "--accent-primary": "#7c6eff",
      "--accent-primary-dim": "rgba(124,110,255,0.15)",
      "--accent-secondary": "#36d9c4",
      "--text-primary": "#e6edf3",
      "--text-secondary": "#8b949e",
      "--text-muted": "#484f58",
      "--border-subtle": "rgba(48,54,61,0.8)",
      "--border-default": "rgba(48,54,61,1)",
      "--border-accent": "rgba(124,110,255,0.4)",
    },
  },
  {
    id: "light",
    label: "Parallax Light",
    vars: {
      "--bg-void": "#f0f2f5",
      "--bg-base": "#ffffff",
      "--bg-surface": "#f6f8fa",
      "--bg-elevated": "#ffffff",
      "--bg-overlay": "#eaeef2",
      "--bg-input": "#f6f8fa",
      "--accent-primary": "#6750e8",
      "--accent-primary-dim": "rgba(103,80,232,0.1)",
      "--accent-secondary": "#0d9488",
      "--text-primary": "#1c2128",
      "--text-secondary": "#57606a",
      "--text-muted": "#8c959f",
      "--border-subtle": "rgba(208,215,222,0.8)",
      "--border-default": "rgba(208,215,222,1)",
      "--border-accent": "rgba(103,80,232,0.35)",
    },
  },
  {
    id: "high-contrast",
    label: "High Contrast",
    vars: {
      "--bg-void": "#000000",
      "--bg-base": "#000000",
      "--bg-surface": "#0a0a0a",
      "--bg-elevated": "#111111",
      "--bg-overlay": "#1a1a1a",
      "--bg-input": "#0a0a0a",
      "--accent-primary": "#ffff00",
      "--accent-primary-dim": "rgba(255,255,0,0.15)",
      "--accent-secondary": "#00ffcc",
      "--text-primary": "#ffffff",
      "--text-secondary": "#cccccc",
      "--text-muted": "#888888",
      "--border-subtle": "rgba(255,255,255,0.2)",
      "--border-default": "rgba(255,255,255,0.4)",
      "--border-accent": "rgba(255,255,0,0.6)",
    },
  },
  {
    id: "monokai",
    label: "Monokai",
    vars: {
      "--bg-void": "#1a1a1a",
      "--bg-base": "#272822",
      "--bg-surface": "#2d2e27",
      "--bg-elevated": "#3e3d32",
      "--bg-overlay": "#49483e",
      "--bg-input": "#2d2e27",
      "--accent-primary": "#a6e22e",
      "--accent-primary-dim": "rgba(166,226,46,0.15)",
      "--accent-secondary": "#66d9e8",
      "--text-primary": "#f8f8f2",
      "--text-secondary": "#cfcfc2",
      "--text-muted": "#75715e",
      "--border-subtle": "rgba(117,113,94,0.3)",
      "--border-default": "rgba(117,113,94,0.6)",
      "--border-accent": "rgba(166,226,46,0.4)",
    },
  },
  {
    id: "solarized",
    label: "Solarized Dark",
    vars: {
      "--bg-void": "#00212b",
      "--bg-base": "#002b36",
      "--bg-surface": "#073642",
      "--bg-elevated": "#083f4d",
      "--bg-overlay": "#0d4a5c",
      "--bg-input": "#073642",
      "--accent-primary": "#268bd2",
      "--accent-primary-dim": "rgba(38,139,210,0.15)",
      "--accent-secondary": "#2aa198",
      "--text-primary": "#fdf6e3",
      "--text-secondary": "#93a1a1",
      "--text-muted": "#586e75",
      "--border-subtle": "rgba(88,110,117,0.3)",
      "--border-default": "rgba(88,110,117,0.6)",
      "--border-accent": "rgba(38,139,210,0.4)",
    },
  },
];

const STORAGE_KEY = "parallax_theme";
const CUSTOM_CSS_KEY = "parallax_custom_css";

function loadSaved(): ThemeId {
  try { return (localStorage.getItem(STORAGE_KEY) as ThemeId) ?? "dark"; }
  catch { return "dark"; }
}

function loadCustomCss(): string {
  try { return localStorage.getItem(CUSTOM_CSS_KEY) ?? ""; }
  catch { return ""; }
}

export const activeTheme = $state<{ id: ThemeId; customCss: string }>({
  id: loadSaved(),
  customCss: loadCustomCss(),
});

export function applyTheme(id: ThemeId, customCss?: string) {
  activeTheme.id = id;
  if (customCss !== undefined) activeTheme.customCss = customCss;

  const root = document.documentElement;
  if (id !== "custom") {
    const theme = THEMES.find(t => t.id === id);
    if (theme) {
      for (const [k, v] of Object.entries(theme.vars)) {
        root.style.setProperty(k, v);
      }
    }
  }

  // Apply custom CSS override
  let styleEl = document.getElementById("parallax-custom-css") as HTMLStyleElement | null;
  if (!styleEl) {
    styleEl = document.createElement("style");
    styleEl.id = "parallax-custom-css";
    document.head.appendChild(styleEl);
  }
  styleEl.textContent = activeTheme.customCss;

  try {
    localStorage.setItem(STORAGE_KEY, id);
    localStorage.setItem(CUSTOM_CSS_KEY, activeTheme.customCss);
  } catch {}
}
