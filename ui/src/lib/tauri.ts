// Tauri API wrapper for type-safe commands
import { invoke } from "@tauri-apps/api/core";
import type {
  AppConfig,
  AppConfigUpdate,
  Card,
  ChatSession,
  CreateCardInput,
  CreateMcpServerInput,
  CreateStackInput,
  Event,
  McpServer,
  McpServerStatus,
  McpToolSummary,
  Message,
  Project,
  ProjectConfig,
  ProjectConfigUpdate,
  RecentProjectInfo,
  Stack,
  StackExecution,
  UpdateCardInput,
  UpdateStackInput,
  WatchPath,
} from "../types";

// Agent configuration for chat
export interface AgentConfig {
  api_key: string;
  provider?: "anthropic" | "openai" | "google" | "ollama";
  model?: string;
}

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

  // Stack commands
  listStacks: (): Promise<Stack[]> => invoke("list_stacks"),

  getStack: (stackId: string): Promise<Stack | null> =>
    invoke("get_stack", { stackId }),

  createStack: (input: CreateStackInput): Promise<Stack> =>
    invoke("create_stack", { input }),

  updateStack: (stackId: string, input: UpdateStackInput): Promise<Stack> =>
    invoke("update_stack", { stackId, input }),

  deleteStack: (stackId: string): Promise<void> =>
    invoke("delete_stack", { stackId }),

  enableStack: (stackId: string): Promise<Stack> =>
    invoke("enable_stack", { stackId }),

  disableStack: (stackId: string): Promise<Stack> =>
    invoke("disable_stack", { stackId }),

  // Card commands
  listCards: (stackId: string): Promise<Card[]> =>
    invoke("list_cards", { stackId }),

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

  triggerCard: (cardId: string): Promise<StackExecution> =>
    invoke("trigger_card", { cardId }),

  // Execution commands
  listExecutions: (limit?: number): Promise<StackExecution[]> =>
    invoke("list_executions", { limit }),

  getExecution: (executionId: string): Promise<StackExecution | null> =>
    invoke("get_execution", { executionId }),

  listExecutionsForEvent: (eventId: string): Promise<StackExecution[]> =>
    invoke("list_executions_for_event", { eventId }),

  // Session commands
  listSessions: (): Promise<ChatSession[]> => invoke("list_sessions"),

  listActiveSessions: (): Promise<ChatSession[]> =>
    invoke("list_active_sessions"),

  createSession: (title?: string): Promise<ChatSession> =>
    invoke("create_session", { title }),

  getSession: (sessionId: string): Promise<ChatSession | null> =>
    invoke("get_session", { sessionId }),

  updateSessionTitle: (sessionId: string, title: string): Promise<ChatSession> =>
    invoke("update_session_title", { sessionId, title }),

  archiveSession: (sessionId: string): Promise<ChatSession> =>
    invoke("archive_session", { sessionId }),

  unarchiveSession: (sessionId: string): Promise<ChatSession> =>
    invoke("unarchive_session", { sessionId }),

  deleteSession: (sessionId: string): Promise<void> =>
    invoke("delete_session", { sessionId }),

  // Chat commands
  getMessages: (sessionId: string): Promise<Message[]> =>
    invoke("get_messages", { sessionId }),

  /**
   * Send a message to the chat agent (streaming).
   * Returns immediately with the user message.
   * The agent response is emitted via Tauri events as it streams.
   */
  sendMessage: (
    sessionId: string,
    content: string,
    agentConfig: AgentConfig
  ): Promise<Message> =>
    invoke("send_message", { sessionId, content, agentConfig }),

  /**
   * Send a message and wait for the complete response (non-streaming).
   */
  sendMessageSync: (
    sessionId: string,
    content: string,
    agentConfig: AgentConfig
  ): Promise<Message> =>
    invoke("send_message_sync", { sessionId, content, agentConfig }),

  // MCP server commands
  listMcpServers: (): Promise<McpServer[]> => invoke("list_mcp_servers"),

  addMcpServer: (input: CreateMcpServerInput): Promise<McpServer> =>
    invoke("add_mcp_server", { input }),

  removeMcpServer: (serverId: string): Promise<void> =>
    invoke("remove_mcp_server", { serverId }),

  enableMcpServer: (serverId: string): Promise<McpServer> =>
    invoke("enable_mcp_server", { serverId }),

  disableMcpServer: (serverId: string): Promise<McpServer> =>
    invoke("disable_mcp_server", { serverId }),

  listMcpTools: (): Promise<McpToolSummary[]> => invoke("list_mcp_tools"),

  testMcpServer: (serverId: string): Promise<McpServerStatus> =>
    invoke("test_mcp_server", { serverId }),

  // Settings commands
  getAppConfig: (): Promise<AppConfig> => invoke("get_app_config"),

  updateAppConfig: (updates: AppConfigUpdate): Promise<AppConfig> =>
    invoke("update_app_config", { updates }),

  getProjectSettings: (): Promise<ProjectConfig> =>
    invoke("get_project_settings"),

  updateProjectSettings: (updates: ProjectConfigUpdate): Promise<ProjectConfig> =>
    invoke("update_project_settings", { updates }),
};
