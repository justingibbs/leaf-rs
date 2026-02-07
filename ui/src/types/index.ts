// TypeScript types matching Rust types from leaf-core

export interface Project {
  id: string;
  name: string;
  path: string;
  created_at: string;
  last_opened_at: string;
}

export interface Card {
  id: string;
  project_id: string;
  name: string;
  description: string;
  trigger: TriggerConfig;
  program: ProgramConfig;
  enabled: boolean;
  session_id: string | null;
  created_at: string;
  updated_at: string;
}

export type TriggerConfig =
  | { type: "file_created"; watch_path: string; patterns: string[] }
  | { type: "file_modified"; watch_path: string; patterns: string[] }
  | { type: "schedule"; cron: string }
  | { type: "manual" };

export interface ProgramConfig {
  language: string;
  entrypoint: string;
  dependencies: string[];
  timeout_secs: number;
  max_retries: number;
}

export interface Event {
  id: string;
  project_id: string;
  event_type: EventType;
  payload: EventPayload;
  status: EventStatus;
  matched_cards: string[];
  created_at: string;
  processed_at: string | null;
}

export type EventType = "file_created" | "file_modified" | "schedule" | "manual";

export type EventStatus = "pending" | "processing" | "completed" | "failed";

export type EventPayload =
  | { type: "file"; path: string; size?: number; mime_type?: string }
  | { type: "schedule"; scheduled_time: string }
  | { type: "manual"; input?: string };

export interface Execution {
  id: string;
  card_id: string;
  event_id: string | null;
  status: ExecutionStatus;
  attempt: number;
  stdout: string;
  stderr: string;
  exit_code: number | null;
  started_at: string;
  completed_at: string | null;
  duration_ms: number | null;
}

export type ExecutionStatus =
  | "pending"
  | "running"
  | "success"
  | "failed"
  | "timeout"
  | "cancelled";

export interface ChatSession {
  id: string;
  project_id: string;
  title: string;
  status: SessionStatus;
  created_at: string;
  updated_at: string;
}

export type SessionStatus = "active" | "completed" | "archived";

export interface Message {
  id: string;
  session_id: string;
  role: MessageRole;
  content: string;
  tool_calls: ToolCall[];
  created_at: string;
}

export type MessageRole = "user" | "assistant" | "system" | "tool";

export interface ToolCall {
  id: string;
  name: string;
  arguments: unknown;
  result?: unknown;
}

// Input types for API calls
export interface CreateCardInput {
  name: string;
  description: string;
  trigger?: TriggerConfig;
  program?: ProgramConfig;
  session_id?: string;
}

export interface UpdateCardInput {
  name?: string;
  description?: string;
  trigger?: TriggerConfig;
  program?: ProgramConfig;
  enabled?: boolean;
}

export interface RecentProjectInfo {
  name: string;
  path: string;
  last_opened: string;
}

// Watch path configuration
export interface WatchPath {
  path: string;
  patterns: string[];
  enabled: boolean;
  debounce_ms: number;
}

// MCP server configuration
export interface McpServer {
  id: string;
  project_id: string;
  name: string;
  command: string;
  args: string[];
  env: Record<string, string>;
  enabled: boolean;
  created_at: string;
}

// Input for creating an MCP server
export interface CreateMcpServerInput {
  name: string;
  command: string;
  args?: string[];
  env?: Record<string, string>;
}

// MCP tool summary
export interface McpToolSummary {
  server_name: string;
  server_id: string;
  name: string;
  description: string | null;
}

// MCP server connection status
export interface McpServerStatus {
  connected: boolean;
  tool_count: number;
}

// App configuration (from Rust AppConfig)
export interface AppConfig {
  dataDir: string;
  logDir: string;
  logLevel: string;
  defaultLlmProvider: string;
  theme: "light" | "dark" | "system";
}

// App config update input
export interface AppConfigUpdate {
  defaultLlmProvider?: string;
  theme?: string;
  logLevel?: string;
}

// LLM settings for a project
export interface LlmSettings {
  provider: string;
  model: string;
  api_key: string | null;
  base_url: string | null;
  temperature: number;
  max_tokens: number;
}

// Execution settings for a project
export interface ExecutionSettings {
  timeout_secs: number;
  max_retries: number;
  allow_network: boolean;
  deno_permissions: string[];
}

// Project configuration
export interface ProjectConfig {
  name: string;
  watchPaths: WatchPath[];
  llmSettings: LlmSettings;
  executionSettings: ExecutionSettings;
}

// LLM settings update input (camelCase to match Rust serde rename_all)
export interface LlmSettingsUpdate {
  provider?: string;
  model?: string;
  apiKey?: string | null;
  baseUrl?: string | null;
  temperature?: number;
  maxTokens?: number;
}

// Execution settings update input (camelCase to match Rust serde rename_all)
export interface ExecutionSettingsUpdate {
  timeoutSecs?: number;
  maxRetries?: number;
  allowNetwork?: boolean;
  denoPermissions?: string[];
}

// Project config update input
export interface ProjectConfigUpdate {
  name?: string;
  llmSettings?: LlmSettingsUpdate;
  executionSettings?: ExecutionSettingsUpdate;
}

// LEAF events emitted from Rust to frontend
export type LeafEvent =
  | { type: "project_opened"; payload: Project }
  | { type: "project_closed"; payload: { project_id: string } }
  | { type: "project_updated"; payload: Project }
  | { type: "card_created"; payload: Card }
  | { type: "card_updated"; payload: Card }
  | { type: "card_deleted"; payload: { card_id: string } }
  | { type: "card_enabled"; payload: { card_id: string } }
  | { type: "card_disabled"; payload: { card_id: string } }
  | { type: "file_detected"; payload: { project_id: string; path: string; event_type: string } }
  | { type: "watcher_started"; payload: { project_id: string; paths: string[] } }
  | { type: "watcher_stopped"; payload: { project_id: string } }
  | { type: "watcher_error"; payload: { project_id: string; error: string } }
  | { type: "event_created"; payload: Event }
  | { type: "event_processing"; payload: { event_id: string; matched_cards: string[] } }
  | { type: "event_completed"; payload: { event_id: string; status: EventStatus } }
  | { type: "execution_started"; payload: Execution }
  | { type: "execution_progress"; payload: { execution_id: string; stdout: string; stderr: string } }
  | { type: "execution_completed"; payload: { execution_id: string; status: ExecutionStatus; exit_code: number | null } }
  | { type: "session_created"; payload: ChatSession }
  | { type: "session_updated"; payload: ChatSession }
  | { type: "message_received"; payload: Message }
  | { type: "agent_thinking"; payload: { session_id: string } }
  | { type: "agent_tool_call"; payload: { session_id: string; tool_name: string; arguments: unknown } }
  // MCP events
  | { type: "mcp_server_connected"; payload: { server_id: string; server_name: string; tool_count: number } }
  | { type: "mcp_server_disconnected"; payload: { server_id: string; server_name: string } }
  | { type: "mcp_server_error"; payload: { server_id: string; server_name: string; error: string } }
  | { type: "mcp_tool_called"; payload: { session_id: string; server_name: string; tool_name: string } }
  | { type: "mcp_tool_result"; payload: { session_id: string; server_name: string; tool_name: string; success: boolean } }
  // System events
  | { type: "error"; payload: { context: string; message: string } }
  | { type: "warning"; payload: { context: string; message: string } };
