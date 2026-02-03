# LEAF Rust Rewrite Plan

## Overview

This document outlines the plan to rewrite LEAF from Python/FastAPI to a native Rust application with a TypeScript/React frontend using Tauri. The result will be a single downloadable executable that requires no runtime dependencies.

### Technical Inspiration

[Goose](https://github.com/block/goose) by Block is a key technical reference for this rewrite. Goose is a desktop AI agent built with Rust backend and TypeScript/React frontend using Tauri — the same architecture we're targeting. Their codebase demonstrates patterns for:
- Structuring a Rust workspace with multiple crates
- Tauri 2.x IPC between Rust and TypeScript
- Multi-provider LLM integration
- Desktop app distribution

### Goals

1. **Single binary distribution** — Download and run, no Python/Node required
2. **Native performance** — Faster startup, lower memory, better responsiveness
3. **Smaller package size** — ~15-20MB vs ~200MB+ with Python bundled
4. **Modern architecture** — Rust safety guarantees, async-first design
5. **Cross-platform ready** — Same codebase for macOS, Windows, Linux (macOS first)

### Key Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Generated code language | **TypeScript/JavaScript** | Wide ecosystem, easy to sandbox with Deno |
| Backwards compatibility | **No** | Clean break enables simpler architecture |
| LLM providers | **Multi-provider** | OpenAI, Anthropic, Google, Ollama out of box |
| Initial platform | **macOS** | Ship fast, iterate, expand later |
| Desktop framework | **Tauri 2.x** | Already planned, Rust-native, small binaries |
| JS runtime | **Deno** | Secure by default, TypeScript native, easy to embed |

---

## Architecture

### High-Level Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                           LEAF (Tauri App)                               │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                      React + TypeScript UI                          │ │
│  │            (Project Picker, Chat, Cards, Event Queue)               │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                 │                                        │
│                          Tauri Commands                                  │
│                                 │                                        │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                         Rust Backend                                │ │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐ │ │
│  │  │  Agent   │ │  Cards   │ │ Watcher  │ │ Executor │ │   MCP    │ │ │
│  │  └──────────┘ └──────────┘ └──────────┘ └──────────┘ └──────────┘ │ │
│  │                                                                     │ │
│  │  ┌──────────────────────────────────────────────────────────────┐  │ │
│  │  │                      Event Bus (tokio)                        │  │ │
│  │  └──────────────────────────────────────────────────────────────┘  │ │
│  │                                                                     │ │
│  │  ┌──────────────────────────────────────────────────────────────┐  │ │
│  │  │                    SQLite (rusqlite/SQLx)                     │  │ │
│  │  └──────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                    Bundled Deno Runtime                             │ │
│  │              (Sandboxed TypeScript/JavaScript Execution)            │ │
│  └────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────┘
```

### Crate Structure

```
leaf-rs/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── leaf-core/                # Core types, traits, utilities
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── types.rs          # Card, Event, Execution, etc.
│   │   │   ├── config.rs         # App and project configuration
│   │   │   ├── error.rs          # Error types
│   │   │   └── events.rs         # Event bus traits
│   │   └── Cargo.toml
│   │
│   ├── leaf-db/                  # Database layer
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── models.rs         # SQLite models
│   │   │   ├── migrations.rs     # Schema migrations
│   │   │   └── queries.rs        # CRUD operations
│   │   └── Cargo.toml
│   │
│   ├── leaf-watcher/             # File system monitoring
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── watcher.rs        # notify-based watcher
│   │   │   └── debounce.rs       # Event debouncing
│   │   └── Cargo.toml
│   │
│   ├── leaf-agent/               # LLM agent & code generation
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── agent.rs          # Agent orchestration
│   │   │   ├── providers/        # LLM provider implementations
│   │   │   │   ├── mod.rs
│   │   │   │   ├── openai.rs
│   │   │   │   ├── anthropic.rs
│   │   │   │   ├── google.rs
│   │   │   │   └── ollama.rs
│   │   │   ├── tools.rs          # Agent tools (propose_card, etc.)
│   │   │   ├── prompts.rs        # System prompts
│   │   │   └── codegen.rs        # TypeScript code generation
│   │   └── Cargo.toml
│   │
│   ├── leaf-executor/            # Sandboxed code execution
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── sandbox.rs        # Deno sandbox wrapper
│   │   │   ├── runner.rs         # Execution orchestration
│   │   │   └── retry.rs          # Retry logic
│   │   └── Cargo.toml
│   │
│   ├── leaf-mcp/                 # MCP client
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── client.rs         # MCP protocol client
│   │   │   ├── registry.rs       # Server registry
│   │   │   └── protocol.rs       # JSON-RPC types
│   │   └── Cargo.toml
│   │
│   └── leaf-app/                 # Tauri application
│       ├── src/
│       │   ├── main.rs           # Tauri entry point
│       │   ├── commands/         # Tauri commands (IPC)
│       │   │   ├── mod.rs
│       │   │   ├── projects.rs
│       │   │   ├── cards.rs
│       │   │   ├── sessions.rs   # Chat session management
│       │   │   ├── chat.rs       # Message sending & streaming
│       │   │   ├── executions.rs
│       │   │   └── mcp.rs
│       │   ├── state.rs          # App state management
│       │   └── events.rs         # Event emission to frontend
│       ├── tauri.conf.json
│       └── Cargo.toml
│
├── ui/                           # React + TypeScript frontend
│   ├── src/
│   │   ├── App.tsx
│   │   ├── main.tsx
│   │   ├── components/
│   │   │   ├── layout/
│   │   │   ├── chat/
│   │   │   ├── cards/
│   │   │   ├── projects/
│   │   │   ├── events/
│   │   │   └── ui/               # shadcn components
│   │   ├── hooks/
│   │   │   ├── useTauriCommand.ts
│   │   │   ├── useTauriEvent.ts
│   │   │   ├── useProjects.ts
│   │   │   ├── useCards.ts
│   │   │   ├── useChatSessions.ts  # Session management
│   │   │   └── useChat.ts          # Message streaming
│   │   ├── stores/
│   │   │   ├── projectStore.ts
│   │   │   └── eventStore.ts
│   │   ├── pages/
│   │   │   ├── ProjectPicker.tsx
│   │   │   ├── Dashboard.tsx
│   │   │   ├── CardDetail.tsx
│   │   │   └── Settings.tsx
│   │   └── lib/
│   │       ├── tauri.ts          # Tauri API wrappers
│   │       └── types.ts          # Shared types
│   ├── package.json
│   ├── tsconfig.json
│   ├── vite.config.ts
│   └── tailwind.config.js
│
├── deno/                         # Bundled Deno runtime
│   └── README.md                 # Instructions for bundling
│
└── scripts/
    ├── build.sh                  # Build script
    ├── bundle-deno.sh            # Bundle Deno binary
    └── release.sh                # Release automation
```

---

## Core Components

### 1. leaf-core

Foundation types and traits shared across all crates.

```rust
// crates/leaf-core/src/types.rs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub last_opened_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: String,
    pub name: String,
    pub description: String,
    pub user_prompt: String,
    pub source_session_id: Option<String>,  // ChatSession that created this card
    pub trigger: TriggerConfig,
    pub program: ProgramConfig,
    pub program_path: PathBuf,
    pub timeout_seconds: u32,
    pub retry_count: u32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_run_at: Option<DateTime<Utc>>,
    pub run_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TriggerConfig {
    FileCreated { folder: String, pattern: String },
    FileModified { folder: String, pattern: String },
    Schedule { cron: String },
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgramConfig {
    pub language: ProgramLanguage,
    pub entrypoint: String,
    pub dependencies: Vec<String>,  // npm packages
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgramLanguage {
    TypeScript,
    JavaScript,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub payload: serde_json::Value,
    pub status: EventStatus,
    pub matched_cards: Vec<String>,
    pub execution_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EventStatus {
    Pending,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Execution {
    pub id: String,
    pub card_id: String,
    pub event_id: String,
    pub status: ExecutionStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

// ===== Chat Session Types =====

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: ChatSessionStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatSessionStatus {
    Active,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub session_id: String,
    pub role: MessageRole,
    pub content: String,
    pub tool_calls: Option<Vec<ToolCall>>,
    pub tool_result: Option<serde_json::Value>,
    pub metadata: Option<MessageMetadata>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageRole {
    User,
    Assistant,
    System,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageMetadata {
    pub card_id: Option<String>,        // If this message created a card
    pub card_proposed: Option<bool>,    // If this message proposed a card
    pub tokens_used: Option<u32>,       // Token count for billing/tracking
}
```

### 2. leaf-agent

Multi-provider LLM integration with tool support.

```rust
// crates/leaf-agent/src/providers/mod.rs

use async_trait::async_trait;

#[async_trait]
pub trait LlmProvider: Send + Sync {
    async fn chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<ChatResponse, AgentError>;

    async fn stream_chat(
        &self,
        messages: Vec<ChatMessage>,
        tools: Option<Vec<Tool>>,
    ) -> Result<impl Stream<Item = ChatChunk>, AgentError>;
}

pub struct OpenAiProvider { /* ... */ }
pub struct AnthropicProvider { /* ... */ }
pub struct GoogleProvider { /* ... */ }
pub struct OllamaProvider { /* ... */ }

// Provider factory based on config
pub fn create_provider(config: &ProviderConfig) -> Box<dyn LlmProvider> {
    match config.provider_type {
        ProviderType::OpenAi => Box::new(OpenAiProvider::new(config)),
        ProviderType::Anthropic => Box::new(AnthropicProvider::new(config)),
        ProviderType::Google => Box::new(GoogleProvider::new(config)),
        ProviderType::Ollama => Box::new(OllamaProvider::new(config)),
    }
}
```

**Agent Tools:**

| Tool | Description |
|------|-------------|
| `propose_card` | Propose a new card for user approval |
| `create_card_now` | Create a card immediately |
| `read_file` | Read a file from the project |
| `write_file` | Write a file to the project |
| `list_directory` | List files in a directory |
| `call_mcp_tool` | Call an MCP server tool |
| `list_mcp_tools` | List available MCP tools |

### 3. leaf-executor

Deno-based sandboxed execution for TypeScript/JavaScript.

```rust
// crates/leaf-executor/src/sandbox.rs

use std::process::Stdio;
use tokio::process::Command;

pub struct DenoSandbox {
    deno_path: PathBuf,
    project_root: PathBuf,
}

impl DenoSandbox {
    pub async fn execute(
        &self,
        program_path: &Path,
        event_payload: &serde_json::Value,
        timeout: Duration,
    ) -> Result<ExecutionResult, ExecutorError> {
        let args = vec![
            "run",
            "--allow-read=.",           // Read only project folder
            "--allow-write=.",          // Write only project folder
            "--allow-net=none",         // No network by default
            "--no-prompt",              // Don't prompt for permissions
            program_path.to_str().unwrap(),
        ];

        let mut child = Command::new(&self.deno_path)
            .args(&args)
            .current_dir(&self.project_root)
            .env("LEAF_PROJECT_ROOT", &self.project_root)
            .env("LEAF_EVENT_PAYLOAD", event_payload.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        // Handle timeout
        let result = tokio::time::timeout(timeout, child.wait_with_output()).await;

        match result {
            Ok(Ok(output)) => Ok(ExecutionResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&output.stdout).to_string(),
                stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            }),
            Ok(Err(e)) => Err(ExecutorError::ProcessError(e)),
            Err(_) => {
                child.kill().await?;
                Err(ExecutorError::Timeout)
            }
        }
    }
}
```

**Generated TypeScript Program Template:**

```typescript
// .leaf/programs/csv-analyzer/main.ts

// Environment variables provided by LEAF
const projectRoot = Deno.env.get("LEAF_PROJECT_ROOT")!;
const eventPayload = JSON.parse(Deno.env.get("LEAF_EVENT_PAYLOAD")!);

// Type definitions
interface FileEvent {
  path: string;
  folder: string;
  filename: string;
  size: number;
}

// Main automation logic
async function main() {
  const event = eventPayload as FileEvent;
  const filePath = `${projectRoot}/${event.path}`;

  // Read the CSV file
  const content = await Deno.readTextFile(filePath);
  const lines = content.split("\n");

  // Analyze
  const rowCount = lines.length - 1; // Exclude header
  const columns = lines[0].split(",");

  // Generate report
  const report = {
    filename: event.filename,
    rowCount,
    columnCount: columns.length,
    columns,
    analyzedAt: new Date().toISOString(),
  };

  // Write output
  const outputPath = `${projectRoot}/.leaf/outputs/report_${Date.now()}.json`;
  await Deno.writeTextFile(outputPath, JSON.stringify(report, null, 2));

  console.log(`Report generated: ${outputPath}`);
}

main().catch(console.error);
```

### 4. leaf-watcher

File system monitoring using the `notify` crate.

```rust
// crates/leaf-watcher/src/watcher.rs

use notify::{RecommendedWatcher, RecursiveMode, Watcher, Event};
use tokio::sync::mpsc;

pub struct FileWatcher {
    watcher: RecommendedWatcher,
    event_tx: mpsc::Sender<FileEvent>,
    debouncer: Debouncer,
}

impl FileWatcher {
    pub fn new(project_path: PathBuf) -> Result<(Self, mpsc::Receiver<FileEvent>), WatcherError> {
        let (tx, rx) = mpsc::channel(100);

        let watcher = notify::recommended_watcher(move |res: Result<Event, _>| {
            if let Ok(event) = res {
                // Convert notify event to LEAF FileEvent
                // Apply debouncing
                // Send to channel
            }
        })?;

        Ok((Self { watcher, event_tx: tx, debouncer: Debouncer::new() }, rx))
    }

    pub fn watch(&mut self, path: &Path, pattern: &str) -> Result<(), WatcherError> {
        self.watcher.watch(path, RecursiveMode::Recursive)?;
        // Register pattern for filtering
        Ok(())
    }
}
```

### 5. leaf-mcp

MCP client for external tool integration.

```rust
// crates/leaf-mcp/src/client.rs

use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

pub struct McpClient {
    process: Child,
    request_id: AtomicU64,
}

impl McpClient {
    pub async fn connect(config: &McpServerConfig) -> Result<Self, McpError> {
        let process = Command::new(&config.command)
            .args(&config.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let mut client = Self { process, request_id: AtomicU64::new(0) };
        client.initialize().await?;
        Ok(client)
    }

    pub async fn list_tools(&mut self) -> Result<Vec<Tool>, McpError> {
        self.send_request("tools/list", json!({})).await
    }

    pub async fn call_tool(
        &mut self,
        name: &str,
        arguments: serde_json::Value,
    ) -> Result<serde_json::Value, McpError> {
        self.send_request("tools/call", json!({
            "name": name,
            "arguments": arguments
        })).await
    }
}
```

### 6. leaf-app (Tauri)

Main application with IPC commands.

```rust
// crates/leaf-app/src/commands/cards.rs

use tauri::State;
use leaf_core::types::Card;

#[tauri::command]
pub async fn list_cards(
    state: State<'_, AppState>,
) -> Result<Vec<Card>, String> {
    let project = state.current_project().await
        .ok_or("No project selected")?;

    let cards = state.db.list_cards(&project.id).await
        .map_err(|e| e.to_string())?;

    Ok(cards)
}

#[tauri::command]
pub async fn create_card(
    state: State<'_, AppState>,
    card: CardCreate,
) -> Result<Card, String> {
    let project = state.current_project().await
        .ok_or("No project selected")?;

    // Generate program from agent
    let program = state.agent.generate_code(&card).await
        .map_err(|e| e.to_string())?;

    // Write program to disk
    let program_path = project.path.join(".leaf/programs").join(&card.slug);
    tokio::fs::create_dir_all(&program_path).await
        .map_err(|e| e.to_string())?;
    tokio::fs::write(program_path.join("main.ts"), &program.source).await
        .map_err(|e| e.to_string())?;

    // Save to database
    let card = state.db.create_card(&project.id, card, program_path).await
        .map_err(|e| e.to_string())?;

    // Start watching if file trigger
    if let TriggerConfig::FileCreated { folder, pattern } = &card.trigger {
        state.watcher.watch(&project.path.join(folder), pattern).await
            .map_err(|e| e.to_string())?;
    }

    Ok(card)
}

#[tauri::command]
pub async fn trigger_card(
    state: State<'_, AppState>,
    card_id: String,
) -> Result<Execution, String> {
    let project = state.current_project().await
        .ok_or("No project selected")?;

    let card = state.db.get_card(&card_id).await
        .map_err(|e| e.to_string())?
        .ok_or("Card not found")?;

    let execution = state.executor.run(&card, None).await
        .map_err(|e| e.to_string())?;

    Ok(execution)
}
```

---

## Frontend (React + TypeScript)

The frontend remains React/TypeScript but communicates via Tauri commands instead of HTTP/WebSocket.

### Tauri Command Hooks

```typescript
// ui/src/hooks/useTauriCommand.ts

import { invoke } from "@tauri-apps/api/core";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";

export function useCards() {
  return useQuery({
    queryKey: ["cards"],
    queryFn: () => invoke<Card[]>("list_cards"),
  });
}

export function useCreateCard() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (card: CardCreate) => invoke<Card>("create_card", { card }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["cards"] });
    },
  });
}

export function useTriggerCard() {
  return useMutation({
    mutationFn: (cardId: string) => invoke<Execution>("trigger_card", { cardId }),
  });
}

// ===== Chat Session Hooks =====

export function useChatSessions() {
  return useQuery({
    queryKey: ["chat-sessions"],
    queryFn: () => invoke<ChatSession[]>("list_chat_sessions"),
  });
}

export function useChatSession(sessionId: string) {
  return useQuery({
    queryKey: ["chat-session", sessionId],
    queryFn: () => invoke<ChatSession>("get_chat_session", { sessionId }),
    enabled: !!sessionId,
  });
}

export function useSessionMessages(sessionId: string) {
  return useQuery({
    queryKey: ["session-messages", sessionId],
    queryFn: () => invoke<Message[]>("get_session_messages", { sessionId }),
    enabled: !!sessionId,
  });
}

export function useCreateChatSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => invoke<ChatSession>("create_chat_session"),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["chat-sessions"] });
    },
  });
}

export function useSendMessage() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ sessionId, content }: { sessionId: string; content: string }) =>
      invoke<Message>("send_message", { sessionId, content }),
    onSuccess: (_, { sessionId }) => {
      queryClient.invalidateQueries({ queryKey: ["session-messages", sessionId] });
    },
  });
}

export function useArchiveSession() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sessionId: string) => invoke<void>("archive_chat_session", { sessionId }),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["chat-sessions"] });
    },
  });
}
```

### Tauri Event Hooks

```typescript
// ui/src/hooks/useTauriEvent.ts

import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";

export function useEventStream(onEvent: (event: LeafEvent) => void) {
  useEffect(() => {
    const unlisten = listen<LeafEvent>("leaf-event", (event) => {
      onEvent(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onEvent]);
}

export function useChatStream(onChunk: (chunk: string) => void) {
  useEffect(() => {
    const unlisten = listen<string>("chat-chunk", (event) => {
      onChunk(event.payload);
    });

    return () => {
      unlisten.then((fn) => fn());
    };
  }, [onChunk]);
}
```

---

## Deno Bundling Strategy

Deno can be bundled with the application or downloaded on first run.

### Option A: Bundle with App (Recommended)

```bash
# scripts/bundle-deno.sh

# Download Deno for target platform
DENO_VERSION="1.40.0"
PLATFORM="aarch64-apple-darwin"  # For macOS ARM

curl -fsSL "https://github.com/denoland/deno/releases/download/v${DENO_VERSION}/deno-${PLATFORM}.zip" -o deno.zip
unzip deno.zip -d deno/
rm deno.zip

# Include in Tauri bundle
# Update tauri.conf.json to include deno binary in resources
```

**Tauri Resource Config:**

```json
{
  "bundle": {
    "resources": [
      "deno/deno"
    ]
  }
}
```

### Option B: Download on First Run

```rust
// First-run setup
async fn ensure_deno_installed(app_data_dir: &Path) -> Result<PathBuf, Error> {
    let deno_path = app_data_dir.join("bin/deno");

    if !deno_path.exists() {
        // Download Deno
        let url = format!(
            "https://github.com/denoland/deno/releases/download/v{}/deno-{}.zip",
            DENO_VERSION, platform()
        );
        download_and_extract(&url, &deno_path).await?;
    }

    Ok(deno_path)
}
```

---

## Database Schema

Same structure as Python version, using `rusqlite` or `SQLx`.

```sql
-- Schema stored in .leaf/leaf.db per project

CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    user_prompt TEXT NOT NULL,
    source_session_id TEXT REFERENCES chat_sessions(id),  -- Which conversation created this card
    trigger_config TEXT NOT NULL,      -- JSON
    program_config TEXT NOT NULL,      -- JSON
    program_path TEXT NOT NULL,
    timeout_seconds INTEGER DEFAULT 300,
    retry_count INTEGER DEFAULT 3,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    last_run_at TEXT,
    run_count INTEGER DEFAULT 0
);

CREATE TABLE events (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    timestamp TEXT NOT NULL,
    payload TEXT NOT NULL,             -- JSON
    status TEXT DEFAULT 'pending',
    matched_cards TEXT,                -- JSON array
    execution_id TEXT
);

CREATE TABLE executions (
    id TEXT PRIMARY KEY,
    card_id TEXT REFERENCES cards(id),
    event_id TEXT REFERENCES events(id),
    status TEXT NOT NULL,
    started_at TEXT,
    completed_at TEXT,
    result TEXT,                       -- JSON
    error TEXT,
    stdout TEXT,
    stderr TEXT
);

-- Chat sessions (multiple conversations per project)
CREATE TABLE chat_sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT DEFAULT 'active',      -- active, archived
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Messages within a chat session
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role TEXT NOT NULL,                -- user, assistant, system, tool
    content TEXT NOT NULL,
    tool_calls TEXT,                   -- JSON array of tool calls
    tool_result TEXT,                  -- JSON result from tool
    metadata TEXT,                     -- JSON (card_id, tokens_used, etc.)
    created_at TEXT NOT NULL
);

CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,
    command TEXT,
    args TEXT,                         -- JSON array
    url TEXT,
    enabled BOOLEAN DEFAULT TRUE,
    created_at TEXT NOT NULL
);

-- Indexes for efficient queries
CREATE INDEX idx_cards_session ON cards(source_session_id);
CREATE INDEX idx_sessions_project ON chat_sessions(project_id);
CREATE INDEX idx_sessions_status ON chat_sessions(status);
CREATE INDEX idx_messages_session ON messages(session_id);
CREATE INDEX idx_messages_created ON messages(created_at);
```

---

## Chat Sessions UI

The chat interface supports multiple conversation sessions, similar to Claude.ai or ChatGPT.

```
┌─────────────────────────────────────────────────────────────────────────┐
│  LEAF    Work Reports                           [← Projects]  [⚙]       │
├──────────────────────┬──────────────────────────────────────────────────┤
│                      │                                                   │
│  CHAT SESSIONS       │              ACTIVE CONVERSATION                  │
│  ──────────────      │                                                   │
│                      │  CSV File Analyzer                                │
│  [+ New Chat]        │  ─────────────────────────────────────────────   │
│                      │                                                   │
│  ┌────────────────┐  │  You: When a CSV file is added to inbox/,        │
│  │ CSV File       │  │       analyze it and create a summary report     │
│  │ Analyzer       │  │                                                   │
│  │ 2 min ago  ●   │  │  LEAF: I'll create a Card for that. Here's what  │
│  └────────────────┘  │        I'm planning:                              │
│                      │                                                   │
│  ┌────────────────┐  │  ┌─────────────────────────────────────────────┐ │
│  │ Image Resizer  │  │  │  📊 CSV Analyzer                             │ │
│  │ Setup          │  │  │  Trigger: New *.csv files in inbox/         │ │
│  │ Yesterday      │  │  │  Action: Analyze with pandas, generate JSON │ │
│  └────────────────┘  │  │                                              │ │
│                      │  │  [View Code]  [Create Card]  [Modify]       │ │
│  ┌────────────────┐  │  └─────────────────────────────────────────────┘ │
│  │ Project Setup  │  │                                                   │
│  │ 3 days ago     │  │                                                   │
│  └────────────────┘  │  ┌─────────────────────────────────────────────┐ │
│                      │  │ Ask LEAF anything...                    [↵] │ │
│  ─────────────────   │  └─────────────────────────────────────────────┘ │
│  ARCHIVED            │                                                   │
│                      ├──────────────────────────────────────────────────┤
│  ┌────────────────┐  │                                                   │
│  │ Old experiment │  │  CARDS CREATED IN THIS SESSION                   │
│  │ Jan 15         │  │  ─────────────────────────────────────────────   │
│  └────────────────┘  │                                                   │
│                      │  ┌─────────────────────────────────────────────┐ │
│                      │  │  📊 CSV Analyzer  •  Created just now        │ │
│                      │  │  [View Card]                                  │ │
│                      │  └─────────────────────────────────────────────┘ │
│                      │                                                   │
└──────────────────────┴──────────────────────────────────────────────────┘
```

**Key Features:**

- **Session List**: Shows all conversations, sorted by recency
- **Auto-generated Titles**: First message summarized as title (editable)
- **Active Indicator**: Shows which session has recent activity
- **Archive Support**: Hide old sessions without deleting
- **Card Provenance**: Shows cards created from the current session
- **Seamless Switching**: Click a session to load its conversation history

---

## LLM Provider Configuration

Support multiple providers with a unified interface.

```toml
# ~/.leaf/config.toml

[llm]
default_provider = "anthropic"

[llm.providers.anthropic]
api_key = "sk-ant-..."
model = "claude-sonnet-4-20250514"

[llm.providers.openai]
api_key = "sk-..."
model = "gpt-4o"

[llm.providers.google]
api_key = "..."
model = "gemini-2.0-flash"

[llm.providers.ollama]
base_url = "http://localhost:11434"
model = "llama3.2"
```

**UI Settings Page:**

```
┌─────────────────────────────────────────────────────────────────────┐
│  Settings > AI Providers                                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Default Provider: [Anthropic ▼]                                     │
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Anthropic Claude                                         [✓]    ││
│  │ API Key: sk-ant-api03-••••••••••••                              ││
│  │ Model: claude-sonnet-4-20250514                                 ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ OpenAI                                                   [ ]    ││
│  │ API Key: Not configured                                         ││
│  │ [Configure]                                                     ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Google Gemini                                            [ ]    ││
│  │ API Key: Not configured                                         ││
│  │ [Configure]                                                     ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │ Ollama (Local)                                           [ ]    ││
│  │ URL: http://localhost:11434                                     ││
│  │ [Configure]                                                     ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: Foundation (Weeks 1-2)

**Goal:** Basic Tauri app with project management

- [ ] Set up Rust workspace with crate structure
- [ ] Implement `leaf-core` types
- [ ] Implement `leaf-db` with SQLite
- [ ] Create basic Tauri app shell
- [ ] Implement project CRUD (create, open, list)
- [ ] Port React UI components (use existing code where possible)
- [ ] Wire up project picker UI

**Deliverable:** App that can create/open projects

### Phase 2: File Watching & Events (Weeks 3-4)

**Goal:** Real-time file monitoring and event queue

- [ ] Implement `leaf-watcher` with notify crate
- [ ] Set up event bus with tokio channels
- [ ] Implement Tauri event emission to frontend
- [ ] Port event queue UI components
- [ ] Add event persistence to SQLite
- [ ] Test debouncing and pattern matching

**Deliverable:** Live event queue showing file system changes

### Phase 3: Cards & Triggers (Weeks 5-6)

**Goal:** Card management and trigger matching

- [ ] Card CRUD in `leaf-db`
- [ ] Trigger matching logic in `leaf-core`
- [ ] Card list/detail UI components
- [ ] Manual trigger ("Run Now") functionality
- [ ] Card enable/disable toggle

**Deliverable:** Full card management (without AI generation)

### Phase 4: Execution Engine (Weeks 7-8)

**Goal:** Sandboxed TypeScript execution with Deno

- [ ] Bundle or download Deno
- [ ] Implement `leaf-executor` sandbox
- [ ] Execution lifecycle (start, complete, fail)
- [ ] Retry logic
- [ ] stdout/stderr capture
- [ ] Execution history UI

**Deliverable:** Cards can execute TypeScript programs

### Phase 5: LLM Agent (Weeks 9-11)

**Goal:** Multi-provider LLM integration with code generation

- [ ] Implement provider trait and factory
- [ ] OpenAI provider
- [ ] Anthropic provider
- [ ] Google Gemini provider
- [ ] Ollama provider
- [ ] Agent tool system (propose_card, etc.)
- [ ] TypeScript code generation
- [ ] Chat UI with streaming responses
- [ ] Provider settings UI

**Deliverable:** Full chat-to-card workflow

### Phase 6: MCP Integration (Week 12)

**Goal:** External tool access

- [ ] Implement `leaf-mcp` client
- [ ] Server configuration UI
- [ ] Agent MCP tool access
- [ ] Test with filesystem and fetch servers

**Deliverable:** Agent can use MCP tools

### Phase 7: Polish & Release (Weeks 13-14)

**Goal:** Production-ready macOS release

- [ ] Error handling and user-friendly messages
- [ ] Loading states and optimistic updates
- [ ] Keyboard shortcuts
- [ ] Dark mode refinement
- [ ] App icon and branding
- [ ] DMG installer
- [ ] Code signing for macOS
- [ ] README and documentation
- [ ] GitHub releases automation

**Deliverable:** Downloadable macOS app

---

## Dependencies

### Rust Crates

```toml
# Cargo.toml (workspace)

[workspace]
members = [
    "crates/leaf-core",
    "crates/leaf-db",
    "crates/leaf-watcher",
    "crates/leaf-agent",
    "crates/leaf-executor",
    "crates/leaf-mcp",
    "crates/leaf-app",
]

[workspace.dependencies]
# Async runtime
tokio = { version = "1", features = ["full"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"

# Database
rusqlite = { version = "0.31", features = ["bundled"] }

# File watching
notify = "6"

# HTTP client (for LLM APIs)
reqwest = { version = "0.12", features = ["json", "stream"] }

# Tauri
tauri = { version = "2", features = [] }

# Error handling
thiserror = "1"
anyhow = "1"

# Utilities
uuid = { version = "1", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
glob = "0.3"
async-trait = "0.1"
futures = "0.3"
tracing = "0.1"
```

### Frontend Dependencies

```json
{
  "dependencies": {
    "@tauri-apps/api": "^2.0.0",
    "@tanstack/react-query": "^5.0.0",
    "react": "^18.3.0",
    "react-dom": "^18.3.0",
    "zustand": "^4.5.0",
    "tailwindcss": "^3.4.0",
    "@radix-ui/react-*": "latest",
    "lucide-react": "^0.400.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.0.0",
    "typescript": "^5.4.0",
    "vite": "^5.0.0",
    "@vitejs/plugin-react": "^4.0.0"
  }
}
```

---

## Migration Notes

Since we're doing a clean break (no backwards compatibility), existing Python LEAF projects won't automatically work with the Rust version. Users will need to recreate their cards.

**What changes for users:**

| Aspect | Python LEAF | Rust LEAF |
|--------|-------------|-----------|
| Generated code | Python | TypeScript |
| Runtime | UV + Python | Deno (bundled) |
| Dependencies | pip packages | npm packages (via Deno) |
| Card programs | `main.py` | `main.ts` |
| Environment vars | Same | Same |
| Project structure | Same `.leaf/` folder | Same structure |
| Database | SQLite | SQLite (same schema) |

**Migration path for users:**

1. Install new LEAF
2. Open existing project folder (creates new `.leaf/`)
3. Re-describe automations in chat
4. Agent generates TypeScript versions

---

## Open Questions

1. **Deno vs QuickJS:** Deno is larger (~40MB) but full-featured. QuickJS is tiny but limited. Deno preferred for ecosystem compatibility.

2. **Network access for cards:** Should cards be able to make HTTP requests? Current plan: opt-in with `--allow-net` flag.

3. **npm package support:** Deno supports npm packages. Should we enable this for generated code?

4. **Hot reload for cards:** Should editing a card's code auto-reload? Or require explicit "save"?

5. **Card templates:** Pre-built cards for common tasks (CSV analysis, image processing, etc.)?

---

## Success Metrics

| Metric | Target |
|--------|--------|
| App bundle size | < 30MB |
| Startup time | < 2 seconds |
| Memory usage (idle) | < 100MB |
| Card execution latency | < 500ms overhead |
| First card creation | < 5 minutes for new user |

---

## Getting Started (Development)

```bash
# Prerequisites
rustup default stable
cargo install tauri-cli
npm install -g pnpm

# Clone and setup
git clone https://github.com/yourorg/leaf-rs.git
cd leaf-rs

# Install frontend dependencies
cd ui && pnpm install && cd ..

# Run in development
cargo tauri dev

# Build for release
cargo tauri build
```

---

## References

- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [Deno Manual](https://docs.deno.com/)
- [notify crate](https://docs.rs/notify/)
- [Goose (Block)](https://github.com/block/goose) - Reference architecture
- [Original LEAF Spec](./LEAF_SPEC.md)
