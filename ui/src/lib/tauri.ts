// Tauri API wrapper for type-safe commands
import { invoke } from "@tauri-apps/api/core";
import type { Project, RecentProjectInfo } from "../types";

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

  // TODO: Add more commands as they're implemented in Rust
  // Card commands
  // listCards: (): Promise<Card[]> => invoke("list_cards"),
  // createCard: (name: string, description: string): Promise<Card> =>
  //   invoke("create_card", { name, description }),

  // Session commands
  // listSessions: (): Promise<ChatSession[]> => invoke("list_sessions"),
  // createSession: (title: string): Promise<ChatSession> =>
  //   invoke("create_session", { title }),
};
