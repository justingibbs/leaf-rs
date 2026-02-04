// Hook for managing theme (dark mode)
import { useEffect } from "react";
import { useSettingsStore } from "../stores/settingsStore";

/**
 * Applies the theme based on settings.
 * Should be called at the app root level.
 */
export function useTheme() {
  const { appConfig, loadAppConfig } = useSettingsStore();

  // Load app config on mount
  useEffect(() => {
    loadAppConfig();
  }, [loadAppConfig]);

  // Apply theme when config changes
  useEffect(() => {
    const theme = appConfig?.theme || "system";
    applyTheme(theme);

    // Listen for system theme changes if using system preference
    if (theme === "system") {
      const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
      const handler = () => applyTheme("system");
      mediaQuery.addEventListener("change", handler);
      return () => mediaQuery.removeEventListener("change", handler);
    }
  }, [appConfig?.theme]);
}

function applyTheme(theme: string) {
  const root = document.documentElement;

  if (theme === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.toggle("dark", prefersDark);
  } else {
    root.classList.toggle("dark", theme === "dark");
  }
}
