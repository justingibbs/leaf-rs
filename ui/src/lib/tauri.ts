// Tauri API wrapper for type-safe commands
import { invoke } from "@tauri-apps/api/core";
import type {
  Card,
  CreateCardInput,
  Event,
  Project,
  RecentProjectInfo,
  UpdateCardInput,
  WatchPath,
} from "../types";

export const api = {
  // Project commands
  createProject: (path: string): Promise<Project> =>
    invoke("create_project", { path }),

  openProject: (path: string): Promise<Project> =>
    invoke("open_project", { path }),

  getCurrentProject: (): Promise<Project | null> =>
    invoke("get_current_project"),

  closeProject: (): Promise<void> => invoke("close_project"),

  listRecentProjects: (): Promise<RecentProjectInfo[]> =>
    invoke("list_recent_projects"),

  // Watcher commands
  startWatcher: (): Promise<string[]> => invoke("start_watcher"),

  stopWatcher: (): Promise<void> => invoke("stop_watcher"),

  isWatcherRunning: (): Promise<boolean> => invoke("is_watcher_running"),

  getWatchPaths: (): Promise<WatchPath[]> => invoke("get_watch_paths"),

  addWatchPath: (path: string, patterns: string[]): Promise<void> =>
    invoke("add_watch_path", { path, patterns }),

  removeWatchPath: (path: string): Promise<void> =>
    invoke("remove_watch_path", { path }),

  // Event commands
  listEvents: (limit?: number): Promise<Event[]> =>
    invoke("list_events", { limit }),

  getEvent: (eventId: string): Promise<Event | null> =>
    invoke("get_event", { eventId }),

  listPendingEvents: (): Promise<Event[]> => invoke("list_pending_events"),

  // Card commands
  listCards: (): Promise<Card[]> => invoke("list_cards"),

  getCard: (cardId: string): Promise<Card | null> =>
    invoke("get_card", { cardId }),

  createCard: (input: CreateCardInput): Promise<Card> =>
    invoke("create_card", { input }),

  updateCard: (cardId: string, input: UpdateCardInput): Promise<Card> =>
    invoke("update_card", { cardId, input }),

  deleteCard: (cardId: string): Promise<void> =>
    invoke("delete_card", { cardId }),

  enableCard: (cardId: string): Promise<Card> =>
    invoke("enable_card", { cardId }),

  disableCard: (cardId: string): Promise<Card> =>
    invoke("disable_card", { cardId }),

  triggerCard: (cardId: string): Promise<void> =>
    invoke("trigger_card", { cardId }),

  // TODO: Add more commands as they're implemented in Rust
  // Session commands
  // listSessions: (): Promise<ChatSession[]> => invoke("list_sessions"),
  // createSession: (title: string): Promise<ChatSession> =>
  //   invoke("create_session", { title }),
};
