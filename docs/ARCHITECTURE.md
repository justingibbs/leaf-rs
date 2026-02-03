# LEAF Architecture

This document describes the technical architecture of LEAF (Local Event-Driven Automation Framework).

## System Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LEAF Application                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │   FastAPI   │    │  PydanticAI │    │    File     │    │     MCP     │  │
│  │   Server    │◄──►│    Agent    │    │   Watcher   │    │   Client    │  │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘  │
│         │                  │                  │                  │          │
│         ▼                  ▼                  ▼                  ▼          │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                           Event Bus                                  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│         │                  │                  │                  │          │
│         ▼                  ▼                  ▼                  ▼          │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐  │
│  │    Cards    │    │  Execution  │    │   Project   │    │  Database   │  │
│  │   Registry  │◄──►│   Engine    │    │   Context   │    │  (SQLite)   │  │
│  └─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘  │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
                    ┌─────────────────────────────────┐
                    │        External Services        │
                    │  ┌─────────┐    ┌───────────┐  │
                    │  │Pydantic │    │    MCP    │  │
                    │  │   AI    │    │  Servers  │  │
                    │  │ Gateway │    │           │  │
                    │  └─────────┘    └───────────┘  │
                    └─────────────────────────────────┘
```

## Component Overview

### API Layer (`src/leaf/api/`)

The FastAPI-based HTTP and WebSocket API layer.

| Module | Purpose |
|--------|---------|
| `app_routes.py` | Application-level endpoints (config) |
| `project_routes.py` | Project and chat management |
| `execution_routes.py` | Execution history and manual triggers |
| `mcp_routes.py` | MCP server management |
| `websocket.py` | Real-time event streaming and chat |
| `debug_routes.py` | Development/debugging endpoints |

### Agent (`src/leaf/agent/`)

PydanticAI-powered conversational agent for creating automations.

| Module | Purpose |
|--------|---------|
| `leaf_agent.py` | Agent configuration and tools |
| `prompts.py` | System prompts and templates |
| `tools.py` | File operations and card proposals |

**Agent Tools:**
- `propose_card` - Propose an automation for user confirmation
- `create_card_now` - Create an automation immediately
- `read_file` - Read project files
- `write_file` - Write project files
- `list_directory` - List directory contents
- `call_mcp_tool` - Call external MCP tools
- `list_mcp_tools` - Discover available MCP tools

### Cards (`src/leaf/cards/`)

Card (automation) management and trigger matching.

| Module | Purpose |
|--------|---------|
| `registry.py` | CRUD operations for cards |
| `models.py` | Pydantic models for cards API |
| `matcher.py` | Match events to card triggers |

### Core (`src/leaf/core/`)

Core infrastructure components.

| Module | Purpose |
|--------|---------|
| `config.py` | Application configuration |
| `events.py` | Event bus for emit/subscribe |

### Database (`src/leaf/db/`)

SQLite database layer using SQLModel.

| Module | Purpose |
|--------|---------|
| `models.py` | Database table definitions |
| `session.py` | Session management |

### Execution (`src/leaf/execution/`)

Sandboxed execution engine for running card programs.

| Module | Purpose |
|--------|---------|
| `sandbox.py` | UV-based isolated execution |
| `runner.py` | Execution orchestration with retries |

### MCP (`src/leaf/mcp/`)

Model Context Protocol client integration.

| Module | Purpose |
|--------|---------|
| `protocol.py` | MCP message type definitions |
| `client.py` | stdio-based MCP client |
| `config.py` | Server configuration management |
| `registry.py` | Active connection registry |
| `agent_tools.py` | PydanticAI integration |
| `card_helper.py` | Helper for card programs |

### Projects (`src/leaf/projects/`)

Project lifecycle management.

| Module | Purpose |
|--------|---------|
| `manager.py` | Project CRUD operations |
| `context.py` | Current project context |
| `models.py` | Project data models |

### Watcher (`src/leaf/watcher/`)

File system monitoring using watchfiles.

| Module | Purpose |
|--------|---------|
| `file_watcher.py` | Directory watching and event emission |

## Data Flow

### 1. Card Creation Flow

```
User Message
    │
    ▼
┌─────────────┐
│  FastAPI    │
│  /api/chat  │
└──────┬──────┘
       │
       ▼
┌─────────────┐     ┌─────────────┐
│  PydanticAI │────►│  Pydantic   │
│    Agent    │◄────│  AI Gateway │
└──────┬──────┘     └─────────────┘
       │
       │ propose_card / create_card_now
       ▼
┌─────────────┐
│    Cards    │
│   Registry  │
└──────┬──────┘
       │
       ├────► SQLite (card record)
       │
       └────► .leaf/programs/{slug}/main.py
```

### 2. Event-Driven Execution Flow

```
File Created/Modified
         │
         ▼
┌─────────────────┐
│   File Watcher  │
│   (watchfiles)  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Event Bus    │
│ emit_file_event │
└────────┬────────┘
         │
         ├────► SQLite (event record)
         │
         ├────► WebSocket (broadcast)
         │
         ▼
┌─────────────────┐
│  Card Matcher   │
│ find_matching   │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Execution Runner│
│  (with retries) │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│    Sandbox      │
│   (UV venv)     │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Card Program   │
│    main.py      │
└─────────────────┘
```

### 3. MCP Tool Call Flow

```
Agent/Card Program
         │
         │ call_mcp_tool(server_id, tool_name, args)
         ▼
┌─────────────────┐
│   MCP Registry  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐     ┌─────────────────┐
│   MCP Client    │────►│   MCP Server    │
│   (JSON-RPC)    │◄────│   (subprocess)  │
└─────────────────┘     └─────────────────┘
```

## Database Schema

Each project has its own SQLite database at `.leaf/leaf.db`.

### Tables

#### `cards`
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Card ID (card_xxx) |
| name | TEXT | Human-readable name |
| description | TEXT | Card description |
| user_prompt | TEXT | Original user request |
| trigger_config | JSON | Trigger configuration |
| program_config | JSON | Program settings |
| program_path | TEXT | Path to program directory |
| timeout_seconds | INT | Execution timeout |
| retry_count | INT | Max retry attempts |
| enabled | BOOL | Whether card is active |
| allowed_outputs | JSON | Permitted output types |
| created_at | DATETIME | Creation timestamp |
| updated_at | DATETIME | Last update timestamp |
| last_run_at | DATETIME | Last execution time |
| run_count | INT | Total executions |

#### `events`
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Event ID (evt_xxx) |
| type | TEXT | Event type (file.created, etc.) |
| timestamp | DATETIME | When event occurred |
| payload | JSON | Event-specific data |
| status | TEXT | pending/processing/completed/failed |
| matched_cards | JSON | Cards that matched this event |
| execution_id | TEXT FK | Associated execution |
| parent_event_id | TEXT FK | Parent event (for chains) |

#### `executions`
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Execution ID (exec_xxx) |
| card_id | TEXT FK | Card that was executed |
| event_id | TEXT FK | Triggering event |
| status | TEXT | pending/running/completed/failed |
| started_at | DATETIME | Start time |
| completed_at | DATETIME | Completion time |
| result | JSON | Execution metadata |
| error | TEXT | Error message if failed |
| stdout | TEXT | Program stdout |
| stderr | TEXT | Program stderr |

#### `chat_messages`
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Message ID (msg_xxx) |
| role | TEXT | user/assistant |
| content | TEXT | Message content |
| message_metadata | JSON | Additional metadata |
| created_at | DATETIME | Timestamp |

#### `mcp_servers`
| Column | Type | Description |
|--------|------|-------------|
| id | TEXT PK | Server ID |
| name | TEXT | Display name |
| type | TEXT | stdio/http |
| command | TEXT | Server command |
| args | JSON | Command arguments |
| url | TEXT | HTTP URL (if applicable) |
| enabled | BOOL | Whether enabled |
| created_at | DATETIME | Creation timestamp |

## Configuration Files

### Application Config (`~/.config/leaf/`)

| File | Purpose |
|------|---------|
| `config.json` | Application settings |
| `mcp.json` | Global MCP server configurations |

### Project Config (`.leaf/`)

| File/Directory | Purpose |
|----------------|---------|
| `leaf.db` | Project SQLite database |
| `programs/` | Card program directories |

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `PYDANTIC_AI_API_KEY` | Pydantic AI Gateway API key | Required |
| `LEAF_MODEL` | AI model to use | google-gla:gemini-2.0-flash |
| `LEAF_PORT` | API server port | 8000 |
| `LEAF_CONFIG_DIR` | Config directory path | ~/.config/leaf |

## Security Considerations

### Sandbox Isolation
- Each card runs in its own UV-managed virtual environment
- Programs cannot access files outside the project directory
- Environment variables are controlled

### Path Validation
- All file operations validate paths are within project bounds
- `.leaf/` directory is protected from direct writes
- Symlink traversal is prevented

### MCP Security
- MCP servers run as subprocesses with controlled stdio
- Built-in servers are disabled by default
- Each server must be explicitly enabled
