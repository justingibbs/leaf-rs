# LEAF-RS - Claude Context

## What is LEAF?

LEAF (Local Event-Driven Automation Framework) is a desktop app that lets users create automations through natural language. Users describe what they want, an LLM generates TypeScript code, and that code runs automatically when files are added to watched folders.

This is a **Rust rewrite** of the original Python LEAF, targeting a single downloadable binary with no runtime dependencies.

## Current Status

**Phase 2: File Watching & Events** — Complete. Ready for Phase 3.

| Phase | Status | Description |
|-------|--------|-------------|
| 1 | ✅ Complete | Foundation (Tauri app, core types, SQLite) |
| 2 | ✅ Complete | File Watching & Events |
| 3 | 🔲 Pending | Cards & Triggers |
| 4 | 🔲 Pending | Execution Engine (Deno sandbox) |
| 5 | 🔲 Pending | LLM Agent (multi-provider) |
| 6 | 🔲 Pending | MCP Integration |
| 7 | 🔲 Pending | Polish & Release |

## Documentation

| Document | Description |
|----------|-------------|
| [context/CONCEPTS.md](context/CONCEPTS.md) | **Concepts & Synchronizations** - READ FIRST for any implementation |
| [context/RUST_REWRITE_PLAN.md](context/RUST_REWRITE_PLAN.md) | Full rewrite plan and architecture |
| [context/LEAF_SPEC.md](context/LEAF_SPEC.md) | Original specification |
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | System design reference |
| [docs/API.md](docs/API.md) | API design reference |
| [docs/CARDS.md](docs/CARDS.md) | Cards guide |

## Architectural Model: Concepts & Synchronizations

LEAF uses the [Concepts & Synchronizations](https://essenceofsoftware.com/) model (Daniel Jackson, MIT). This is **mandatory reading** before implementing features.

**Key principles:**
1. **Concepts are independent** - Each concept (Project, Watcher, Card, Event, Execution) has its own state and actions, defined in isolation
2. **Synchronizations coordinate** - Rules like `Project.open() → Watcher.start()` define how concepts interact
3. **Triggers are self-evaluating** - Call `card.trigger.evaluate(event)` rather than external matching logic
4. **Orchestration lives in leaf-app** - The Tauri app layer implements synchronization rules

**Before implementing any feature:**
1. Check which concepts are involved
2. Review existing synchronization rules
3. Add new sync rules if needed
4. Ensure triggers evaluate themselves

See [context/CONCEPTS.md](context/CONCEPTS.md) for full concept definitions, sync rules, and implementation guidance.

**Currently implemented synchronizations:**
- `Project.open() → Watcher.start()` - `leaf-app/src/state.rs:133`
- `Project.close() → Watcher.stop()` - `leaf-app/src/commands/projects.rs:83`
- `Watcher.detect() → Event.create()` - `leaf-app/src/events.rs:65`
- `Event.create() → TriggerConfig.evaluate()` - `leaf-app/src/events.rs:108`

## Reference Implementation

The original Python implementation is at `../leaf`. Use it for:
- Understanding existing behavior
- Porting frontend components from `../leaf/frontend`
- API contract reference

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Framework | Tauri 2.x |
| Backend | Rust |
| Database | SQLite (rusqlite) |
| File Watching | notify crate |
| Code Execution | Deno (bundled) |
| LLM Providers | OpenAI, Anthropic, Google, Ollama |
| Frontend | React + TypeScript |
| Styling | Tailwind CSS |
| State Management | Zustand + TanStack Query |

## Project Structure

```
leaf-rs/
├── Cargo.toml                    # Workspace root
├── crates/
│   ├── leaf-core/                # Core types, traits, utilities
│   ├── leaf-db/                  # Database layer (SQLite)
│   ├── leaf-watcher/             # File system monitoring
│   ├── leaf-agent/               # LLM agent & code generation
│   ├── leaf-executor/            # Sandboxed code execution (Deno)
│   ├── leaf-mcp/                 # MCP client
│   └── leaf-app/                 # Tauri application
├── ui/                           # React + TypeScript frontend
│   ├── src/
│   │   ├── components/
│   │   ├── hooks/
│   │   ├── stores/
│   │   └── pages/
│   ├── package.json
│   └── vite.config.ts
├── context/                      # Specs and planning docs
└── docs/                         # Documentation
```

## Key Commands

```bash
# Prerequisites
rustup default stable
cargo install tauri-cli
npm install -g pnpm

# Development
cargo tauri dev                   # Run app in development
cargo build                       # Build Rust crates only
cargo test                        # Run Rust tests
cargo clippy                      # Lint Rust code

# Frontend only
cd ui && pnpm install             # Install frontend deps
cd ui && pnpm dev                 # Run frontend dev server

# Release
cargo tauri build                 # Build distributable app
```

## Crate Responsibilities

| Crate | Purpose |
|-------|---------|
| `leaf-core` | Shared types (Card, Event, Execution, ChatSession, Message), traits, error types |
| `leaf-db` | SQLite operations, migrations, CRUD for all entities |
| `leaf-watcher` | File system monitoring with notify, debouncing, pattern matching |
| `leaf-agent` | LLM provider abstraction, tool system, TypeScript code generation |
| `leaf-executor` | Deno sandbox wrapper, execution lifecycle, retry logic |
| `leaf-mcp` | MCP protocol client, server registry, tool discovery |
| `leaf-app` | Tauri commands (IPC), app state, event emission to frontend |

## Core Concepts

### Projects
A folder with `.leaf/` containing database, card programs, and outputs. Each project is self-contained.

### Cards
An automation unit: trigger + TypeScript program + settings. Created via chat with the AI agent.

**Trigger Types:**
- `file_created` - File added to watched folder
- `file_modified` - File changed
- `manual` - Triggered via UI
- `schedule` - Cron-based (future)

### Chat Sessions
Multiple conversations per project. Each session can create cards. Cards track which session created them.

### Execution
Cards run in Deno sandbox with:
- Limited filesystem access (project folder only)
- No network by default
- Configurable timeout
- Automatic retries
- stdout/stderr capture

## Tauri IPC Pattern

Frontend communicates with Rust via Tauri commands:

```typescript
// Frontend
import { invoke } from "@tauri-apps/api/core";
const cards = await invoke<Card[]>("list_cards");
```

```rust
// Backend
#[tauri::command]
pub async fn list_cards(state: State<'_, AppState>) -> Result<Vec<Card>, String> {
    // ...
}
```

Events flow from Rust to frontend via Tauri events:

```rust
// Backend emits
app_handle.emit("leaf-event", &event)?;
```

```typescript
// Frontend listens
import { listen } from "@tauri-apps/api/event";
listen<LeafEvent>("leaf-event", (event) => { /* ... */ });
```

## Code Style

### Rust
- Use `thiserror` for error types
- Use `anyhow` for application errors
- Async with tokio
- All public types derive `Debug, Clone, Serialize, Deserialize`
- Use `tracing` for logging
- Run `cargo clippy` before committing

### TypeScript
- Strict mode enabled
- Use TypeScript types, not `any`
- React functional components with hooks
- TanStack Query for server state
- Zustand for client state

## Key Differences from Python Version

| Aspect | Python LEAF | Rust LEAF |
|--------|-------------|-----------|
| Generated code | Python | TypeScript |
| Runtime | UV + Python | Deno (bundled) |
| IPC | HTTP/WebSocket | Tauri commands/events |
| Distribution | Requires Python | Single binary |
| LLM Framework | PydanticAI | Custom multi-provider |
