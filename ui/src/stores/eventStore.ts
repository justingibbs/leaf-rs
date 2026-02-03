// Event store for managing file events and watcher state
import { create } from "zustand";
import type { Event, WatchPath } from "../types";
import { api } from "../lib/tauri";

interface EventState {
  // Events list
  events: Event[];
  isLoadingEvents: boolean;

  // Watcher state
  isWatcherRunning: boolean;
  watchedPaths: string[];
  watchPaths: WatchPath[];

  // Actions
  loadEvents: (limit?: number) => Promise<void>;
  addEvent: (event: Event) => void;
  updateEventStatus: (eventId: string, status: Event["status"]) => void;

  // Watcher actions
  startWatcher: () => Promise<void>;
  stopWatcher: () => Promise<void>;
  checkWatcherStatus: () => Promise<void>;
  loadWatchPaths: () => Promise<void>;
  addWatchPath: (path: string, patterns: string[]) => Promise<void>;
  removeWatchPath: (path: string) => Promise<void>;

  // Reset
  reset: () => void;
}

export const useEventStore = create<EventState>((set, get) => ({
  // Initial state
  events: [],
  isLoadingEvents: false,
  isWatcherRunning: false,
  watchedPaths: [],
  watchPaths: [],

  // Load events from the backend
  loadEvents: async (limit?: number) => {
    set({ isLoadingEvents: true });
    try {
      const events = await api.listEvents(limit);
      set({ events, isLoadingEvents: false });
    } catch (error) {
      console.error("Failed to load events:", error);
      set({ isLoadingEvents: false });
    }
  },

  // Add a new event (from real-time updates)
  addEvent: (event: Event) => {
    set((state) => ({
      events: [event, ...state.events].slice(0, 100), // Keep last 100 events
    }));
  },

  // Update an event's status (from real-time updates)
  updateEventStatus: (eventId: string, status: Event["status"]) => {
    set((state) => ({
      events: state.events.map((e) =>
        e.id === eventId ? { ...e, status } : e
      ),
    }));
  },

  // Start the file watcher
  startWatcher: async () => {
    try {
      const paths = await api.startWatcher();
      set({ isWatcherRunning: true, watchedPaths: paths });
    } catch (error) {
      console.error("Failed to start watcher:", error);
    }
  },

  // Stop the file watcher
  stopWatcher: async () => {
    try {
      await api.stopWatcher();
      set({ isWatcherRunning: false, watchedPaths: [] });
    } catch (error) {
      console.error("Failed to stop watcher:", error);
    }
  },

  // Check if watcher is running
  checkWatcherStatus: async () => {
    try {
      const isRunning = await api.isWatcherRunning();
      set({ isWatcherRunning: isRunning });
    } catch (error) {
      console.error("Failed to check watcher status:", error);
    }
  },

  // Load watch paths from config
  loadWatchPaths: async () => {
    try {
      const watchPaths = await api.getWatchPaths();
      set({ watchPaths });
    } catch (error) {
      console.error("Failed to load watch paths:", error);
    }
  },

  // Add a new watch path
  addWatchPath: async (path: string, patterns: string[]) => {
    try {
      await api.addWatchPath(path, patterns);
      await get().loadWatchPaths();
    } catch (error) {
      console.error("Failed to add watch path:", error);
    }
  },

  // Remove a watch path
  removeWatchPath: async (path: string) => {
    try {
      await api.removeWatchPath(path);
      await get().loadWatchPaths();
    } catch (error) {
      console.error("Failed to remove watch path:", error);
    }
  },

  // Reset store state
  reset: () => {
    set({
      events: [],
      isLoadingEvents: false,
      isWatcherRunning: false,
      watchedPaths: [],
      watchPaths: [],
    });
  },
}));
