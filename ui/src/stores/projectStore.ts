import { create } from "zustand";
import type { Project } from "../types";
import { api } from "../lib/tauri";

interface ProjectState {
  project: Project | null;
  loading: boolean;
  error: string | null;

  openProject: (path: string) => Promise<void>;
  createProject: (path: string) => Promise<void>;
  closeProject: () => Promise<void>;
  setProject: (project: Project | null) => void;
}

export const useProjectStore = create<ProjectState>((set) => ({
  project: null,
  loading: false,
  error: null,

  openProject: async (path: string) => {
    set({ loading: true, error: null });
    try {
      const project = await api.openProject(path);
      set({ project, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  createProject: async (path: string) => {
    set({ loading: true, error: null });
    try {
      const project = await api.createProject(path);
      set({ project, loading: false });
    } catch (error) {
      set({ error: String(error), loading: false });
    }
  },

  closeProject: async () => {
    try {
      await api.closeProject();
      set({ project: null });
    } catch (error) {
      set({ error: String(error) });
    }
  },

  setProject: (project: Project | null) => {
    set({ project });
  },
}));
