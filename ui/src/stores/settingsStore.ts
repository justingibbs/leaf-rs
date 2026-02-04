import { create } from "zustand";
import type { AppConfig, ProjectConfig, AppConfigUpdate, ProjectConfigUpdate } from "../types";
import { api } from "../lib/tauri";

interface SettingsState {
  // App config
  appConfig: AppConfig | null;
  appConfigLoading: boolean;
  appConfigError: string | null;

  // Project config
  projectConfig: ProjectConfig | null;
  projectConfigLoading: boolean;
  projectConfigError: string | null;

  // Modal state
  isSettingsOpen: boolean;
  activeTab: "llm" | "appearance" | "execution" | "mcp";

  // Actions
  loadAppConfig: () => Promise<void>;
  updateAppConfig: (updates: AppConfigUpdate) => Promise<void>;
  loadProjectSettings: () => Promise<void>;
  updateProjectSettings: (updates: ProjectConfigUpdate) => Promise<void>;

  // Modal actions
  openSettings: (tab?: "llm" | "appearance" | "execution" | "mcp") => void;
  closeSettings: () => void;
  setActiveTab: (tab: "llm" | "appearance" | "execution" | "mcp") => void;
}

export const useSettingsStore = create<SettingsState>((set, get) => ({
  // Initial state
  appConfig: null,
  appConfigLoading: false,
  appConfigError: null,

  projectConfig: null,
  projectConfigLoading: false,
  projectConfigError: null,

  isSettingsOpen: false,
  activeTab: "llm",

  // Load app config from backend
  loadAppConfig: async () => {
    set({ appConfigLoading: true, appConfigError: null });
    try {
      const config = await api.getAppConfig();
      set({ appConfig: config, appConfigLoading: false });
    } catch (error) {
      set({ appConfigError: String(error), appConfigLoading: false });
    }
  },

  // Update app config
  updateAppConfig: async (updates: AppConfigUpdate) => {
    set({ appConfigLoading: true, appConfigError: null });
    try {
      const config = await api.updateAppConfig(updates);
      set({ appConfig: config, appConfigLoading: false });
    } catch (error) {
      set({ appConfigError: String(error), appConfigLoading: false });
    }
  },

  // Load project settings from backend
  loadProjectSettings: async () => {
    set({ projectConfigLoading: true, projectConfigError: null });
    try {
      const config = await api.getProjectSettings();
      set({ projectConfig: config, projectConfigLoading: false });
    } catch (error) {
      set({ projectConfigError: String(error), projectConfigLoading: false });
    }
  },

  // Update project settings
  updateProjectSettings: async (updates: ProjectConfigUpdate) => {
    set({ projectConfigLoading: true, projectConfigError: null });
    try {
      const config = await api.updateProjectSettings(updates);
      set({ projectConfig: config, projectConfigLoading: false });
    } catch (error) {
      set({ projectConfigError: String(error), projectConfigLoading: false });
    }
  },

  // Open settings modal
  openSettings: (tab = "llm") => {
    set({ isSettingsOpen: true, activeTab: tab });
    // Load configs when opening
    get().loadAppConfig();
    get().loadProjectSettings();
  },

  // Close settings modal
  closeSettings: () => {
    set({ isSettingsOpen: false });
  },

  // Set active tab
  setActiveTab: (tab) => {
    set({ activeTab: tab });
  },
}));
