// Appearance settings component
import { useState, useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";

const THEMES = [
  { value: "light", label: "Light", icon: "sun" },
  { value: "dark", label: "Dark", icon: "moon" },
  { value: "system", label: "System", icon: "computer" },
] as const;

export function AppearanceSettings() {
  const { appConfig, updateAppConfig, appConfigLoading, appConfigError } = useSettingsStore();

  const [theme, setTheme] = useState<"light" | "dark" | "system">("system");
  const [isSaving, setIsSaving] = useState(false);
  const [saveSuccess, setSaveSuccess] = useState(false);

  // Initialize from appConfig
  useEffect(() => {
    if (appConfig?.theme) {
      setTheme(appConfig.theme as "light" | "dark" | "system");
    }
  }, [appConfig]);

  const handleThemeChange = async (newTheme: "light" | "dark" | "system") => {
    setTheme(newTheme);
    setIsSaving(true);
    setSaveSuccess(false);

    try {
      await updateAppConfig({ theme: newTheme });
      setSaveSuccess(true);

      // Apply theme to document
      applyTheme(newTheme);

      setTimeout(() => setSaveSuccess(false), 2000);
    } finally {
      setIsSaving(false);
    }
  };

  if (appConfigLoading && !appConfig) {
    return (
      <div className="p-4 text-gray-500 text-sm">Loading settings...</div>
    );
  }

  return (
    <div className="space-y-6">
      {appConfigError && (
        <div className="p-3 bg-red-50 border border-red-200 rounded-md text-sm text-red-700">
          {appConfigError}
        </div>
      )}

      {/* Theme Selection */}
      <div>
        <label className="block text-sm font-medium text-gray-700 mb-3">
          Theme
        </label>
        <div className="grid grid-cols-3 gap-3">
          {THEMES.map((t) => (
            <button
              key={t.value}
              type="button"
              onClick={() => handleThemeChange(t.value)}
              className={`flex flex-col items-center justify-center px-4 py-4 rounded-lg border transition-colors ${
                theme === t.value
                  ? "bg-leaf-50 border-leaf-500 text-leaf-700"
                  : "bg-white border-gray-300 text-gray-700 hover:bg-gray-50 dark:bg-gray-800 dark:border-gray-600 dark:text-gray-300"
              }`}
            >
              <span className="text-2xl mb-1">
                {t.icon === "sun" && "☀️"}
                {t.icon === "moon" && "🌙"}
                {t.icon === "computer" && "💻"}
              </span>
              <span className="text-sm font-medium">{t.label}</span>
            </button>
          ))}
        </div>
        <p className="text-xs text-gray-500 mt-2">
          {theme === "system" && "Theme will follow your system preferences"}
          {theme === "light" && "Always use light theme"}
          {theme === "dark" && "Always use dark theme"}
        </p>
      </div>

      {/* Preview */}
      <div className="pt-4 border-t border-gray-200">
        <h4 className="text-sm font-medium text-gray-700 mb-3">Preview</h4>
        <div className="grid grid-cols-2 gap-4">
          {/* Light preview */}
          <div className="p-4 bg-white border border-gray-200 rounded-lg">
            <div className="h-3 w-16 bg-gray-300 rounded mb-2" />
            <div className="h-2 w-24 bg-gray-200 rounded mb-3" />
            <div className="flex gap-2">
              <div className="h-6 w-12 bg-green-500 rounded" />
              <div className="h-6 w-12 bg-gray-100 rounded" />
            </div>
          </div>

          {/* Dark preview */}
          <div className="p-4 bg-gray-900 border border-gray-700 rounded-lg">
            <div className="h-3 w-16 bg-gray-600 rounded mb-2" />
            <div className="h-2 w-24 bg-gray-700 rounded mb-3" />
            <div className="flex gap-2">
              <div className="h-6 w-12 bg-green-600 rounded" />
              <div className="h-6 w-12 bg-gray-700 rounded" />
            </div>
          </div>
        </div>
      </div>

      {/* Status indicator */}
      {(isSaving || saveSuccess) && (
        <div className={`text-sm ${saveSuccess ? "text-green-600" : "text-gray-500"}`}>
          {isSaving ? "Saving..." : "Theme saved!"}
        </div>
      )}
    </div>
  );
}

// Apply theme to document
function applyTheme(theme: "light" | "dark" | "system") {
  const root = document.documentElement;

  if (theme === "system") {
    const prefersDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.toggle("dark", prefersDark);
  } else {
    root.classList.toggle("dark", theme === "dark");
  }
}
