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
