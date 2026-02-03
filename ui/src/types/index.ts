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
  | { type: "error"; payload: { context: string; message: string } }
  | { type: "warning"; payload: { context: string; message: string } };
