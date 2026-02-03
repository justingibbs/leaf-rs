# LEAF - Local Event-Driven Automation Framework

## Specification v2.2

---

## Overview

LEAF is a local-first desktop application that enables users to create event-driven automations through natural language. Inspired by HyperCard's approachability and Claude Cowork's sandboxed agent model, LEAF provides a constrained, predictable environment where an LLM can observe, reason, write code, and act on files within a user-defined project folder.

### Core Concepts

- **LEAF App**: Installed once on the user's machine, manages multiple projects
- **Project**: A self-contained automation environment tied to a specific folder
- **Card**: An automation within a project (trigger + generated code + settings)
- **Event Queue**: Real-time visibility into all events and executions within a project

### Core Workflow

1. User opens LEAF and creates or selects a **Project**
2. User describes what they want: *"Analyze CSV files dropped here and generate a summary report"*
3. LEAF's agent creates a **Card** and writes Python code to perform the analysis
4. When files are added to the watched folder, the Card triggers automatically
5. The analysis runs, a report is generated, and everything is visible in the **Event Queue**

### Core Principles

1. **Fully Local** — No cloud sync, no external services except LLM API calls and MCP servers
2. **Project-Based** — Each project is self-contained with its own database, cards, and queue
3. **Sandboxed** — All code execution and file writes confined to the project folder
4. **Event-Driven** — Cards react to file system events, schedules, or manual triggers
5. **Code-Generating** — The LLM writes Python programs to accomplish user goals
6. **Typed Interactions** — LLM operates within a fixed vocabulary of inputs, outputs, and actions
7. **Transparent** — All events, executions, and agent reasoning visible in a real-time queue
8. **Portable** — Projects can be zipped and moved to another machine

---

## Technology Stack

| Layer | Technology | Notes |
|-------|------------|-------|
| Desktop Shell | Tauri 2.x | Rust backend, WebView frontend, ~5MB bundle |
| Backend | FastAPI | Async Python, WebSocket support |
| Package Manager | UV | Fast, Rust-based, replaces pip/venv/pyenv |
| AI Framework | PydanticAI | Agents, structured outputs, MCP integration |
| Model Access | Pydantic AI Gateway | Multi-provider, cost control, native format passthrough |
| Workflow Engine | Temporal | Durable execution, crash recovery, retries |
| Database | SQLite via SQLModel | Per-project local persistence |
| File Watching | watchfiles | Rust-based, fast, cross-platform |
| UI Framework | React + TypeScript | Tauri frontend |
| Styling | Tailwind CSS | Rapid iteration |

---

## Architecture

### Two-Level Structure

```
┌─────────────────────────────────────────────────────────────────┐
│                         LEAF APP                                 │
│                    (installed on machine)                        │
│                                                                  │
│  App Config: ~/.leaf/                                           │
│  ├── config.json      (app settings: theme, default model)      │
│  └── projects.json    (registry of known projects)              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
                              │
           ┌──────────────────┼──────────────────┐
           ▼                  ▼                  ▼
┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐
│   Project A     │  │   Project B     │  │   Project C     │
│ ~/Work/Reports  │  │ ~/Personal/Tax  │  │ ~/Dev/Scripts   │
│                 │  │                 │  │                 │
│ .leaf/          │  │ .leaf/          │  │ .leaf/          │
│ ├── leaf.db     │  │ ├── leaf.db     │  │ ├── leaf.db     │
│ ├── config.json │  │ ├── config.json │  │ ├── config.json │
│ ├── programs/   │  │ ├── programs/   │  │ ├── programs/   │
│ └── ...         │  │ └── ...         │  │ └── ...         │
└─────────────────┘  └─────────────────┘  └─────────────────┘
```

### App-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                        Tauri Shell (macOS)                       │
│   • Window management                                            │
│   • System tray                                                  │
│   • Native file dialogs                                          │
│   • Notifications                                                │
└─────────────────────────────┬───────────────────────────────────┘
                              │ localhost:8000 (HTTP + WebSocket)
┌─────────────────────────────▼───────────────────────────────────┐
│                      FastAPI Application                         │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    API Routes                               │ │
│  │  /api/projects   /api/cards   /api/events   /api/chat      │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                  WebSocket Manager                          │ │
│  │              (real-time event streaming)                    │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌───────────────┬──────────────────┬────────────────────────┐ │
│  │               │                  │                        │ │
│  ▼               ▼                  ▼                        ▼ │
│ ┌─────────┐ ┌──────────────┐ ┌─────────────┐ ┌─────────────┐  │
│ │ Project │ │ File Watcher │ │    Card     │ │  Chat/Agent │  │
│ │ Manager │ │ (watchfiles) │ │  Registry   │ │  Interface  │  │
│ └────┬────┘ └──────┬───────┘ └──────┬──────┘ └──────┬──────┘  │
│      │             │                │               │          │
│      └─────────────┴────────────────┴───────────────┘          │
│                              │                                  │
│  ┌───────────────────────────▼────────────────────────────────┐ │
│  │                     Event Bus                               │ │
│  │                  (asyncio Queue)                            │ │
│  └───────────────────────────┬────────────────────────────────┘ │
│                              │                                  │
│  ┌───────────────────────────▼────────────────────────────────┐ │
│  │                  Temporal Worker                            │ │
│  │  • Card Execution Workflow                                  │ │
│  │  • Sandbox setup and execution                              │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │                    PydanticAI Agent                         │ │
│  │  • Card creation from natural language                      │ │
│  │  • Python code generation                                   │ │
│  │  • MCP tool access                                          │ │
│  └────────────────────────────────────────────────────────────┘ │
│                                                                  │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │              Per-Project SQLite Database                    │ │
│  │        cards │ events │ executions │ chat_messages          │ │
│  └────────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│                    Temporal Server (embedded)                     │
│   • Workflow persistence    • Crash recovery                     │
│   • Automatic retries       • Execution history                  │
└──────────────────────────────────────────────────────────────────┘
```

---

## Concepts & Synchronizations

LEAF is built using the **Concepts & Synchronizations** model, where independent, self-contained modules (concepts) are connected through explicit synchronization rules. This makes the event-driven architecture transparent and extensible.

> Reference: [MIT Concepts & Synchronizations](https://news.mit.edu/2025/mit-researchers-propose-new-model-for-legible-modular-software-1106)

### Why This Model?

1. **Transparency** — Users can see exactly what triggers what
2. **Debugging** — Trace any behavior through the sync chain
3. **Extensibility** — Add new concepts (webhooks, schedules) with new syncs
4. **Testing** — Each concept tests in isolation; syncs test separately
5. **AI-Friendly** — The agent can reason about and explain syncs to users

### Concept Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LEAF CONCEPT MAP                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐   │
│  │ Project │────────▶│ Watcher │────────▶│  Event  │────────▶│  Queue  │   │
│  └─────────┘  opens   └─────────┘ emits   └─────────┘ pushes  └─────────┘   │
│       │                                        │                     │       │
│       │                                        │ matches             │       │
│       ▼                                        ▼                     ▼       │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐   │
│  │   MCP   │◀────────│  Agent  │◀────────│  Chat   │         │   UI    │   │
│  └─────────┘  tools   └─────────┘ responds└─────────┘         └─────────┘   │
│                            │                                                 │
│                            │ generates                                       │
│                            ▼                                                 │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐                        │
│  │ Program │◀────────│  Card   │◀────────│ Trigger │                        │
│  └─────────┘  has     └─────────┘  has    └─────────┘                        │
│       │                    │                   │                             │
│       │                    │ runs              │ fires                       │
│       ▼                    ▼                   │                             │
│  ┌─────────┐         ┌─────────┐◀──────────────┘                            │
│  │ Sandbox │◀────────│Execution│                                            │
│  └─────────┘  uses    └─────────┘                                            │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

### Concept Definitions

#### 1. Project

**Purpose**: Self-contained automation environment tied to a folder

| State | Description |
|-------|-------------|
| id | Unique identifier |
| name | Human-readable name |
| path | Folder path |
| config | Project-specific settings |
| active | Whether currently open |

| Action | Description |
|--------|-------------|
| create(path, name) | Initialize new project |
| open(id) | Activate project |
| close() | Deactivate project |
| configure(settings) | Update settings |

---

#### 2. Watcher

**Purpose**: Monitors filesystem for changes within a project

| State | Description |
|-------|-------------|
| project_id | Which project this watches |
| paths | Folders being monitored |
| patterns | File patterns (*.csv, etc.) |
| running | Whether actively watching |

| Action | Description |
|--------|-------------|
| start(project_path) | Begin monitoring |
| stop() | Stop monitoring |
| add_path(folder, pattern) | Watch additional folder |
| remove_path(folder) | Stop watching folder |

| Emits | Description |
|-------|-------------|
| file_created(path) | New file detected |
| file_modified(path) | File changed |
| file_deleted(path) | File removed |

---

#### 3. Event

**Purpose**: Immutable record of something that happened

| State | Description |
|-------|-------------|
| id | Unique identifier |
| type | Event type (file.created, card.completed, etc.) |
| timestamp | When it occurred |
| payload | Event-specific data |
| status | pending, processing, completed, failed |
| source_id | What generated this event |

| Action | Description |
|--------|-------------|
| emit(type, payload) | Create new event |
| process() | Mark as being handled |
| complete() | Mark as done |
| fail(error) | Mark as failed |

---

#### 4. Card

**Purpose**: An automation definition (trigger + program + settings)

| State | Description |
|-------|-------------|
| id | Unique identifier |
| name | Human-readable name |
| description | What it does |
| user_prompt | Original user request |
| enabled | Whether active |
| program_path | Path to generated code |
| run_count | How many times executed |

| Action | Description |
|--------|-------------|
| create(spec) | Create new card |
| update(changes) | Modify card |
| enable() | Activate card |
| disable() | Deactivate card |
| delete() | Remove card |

---

#### 5. Trigger

**Purpose**: Condition that activates a card

| State | Description |
|-------|-------------|
| card_id | Which card this triggers |
| type | file_created, file_modified, schedule, manual |
| folder | Watched folder (relative to project) |
| pattern | Glob pattern (*.csv) |
| cron | Cron expression (for schedule) |

| Action | Description |
|--------|-------------|
| evaluate(event) | Check if event matches |
| fire(event) | Activate the associated card |

---

#### 6. Execution

**Purpose**: A single run of a card's program

| State | Description |
|-------|-------------|
| id | Unique identifier |
| card_id | Which card is running |
| event_id | What triggered it |
| status | pending, running, completed, failed |
| started_at | When execution began |
| completed_at | When execution finished |
| result | Output data |
| stdout | Program stdout |
| stderr | Program stderr |
| retry_count | Retries remaining |

| Action | Description |
|--------|-------------|
| start(card, event) | Begin execution |
| complete(result) | Finish successfully |
| fail(error) | Finish with error |
| retry() | Attempt again |
| cancel() | Abort execution |

---

#### 7. Sandbox

**Purpose**: Isolated environment for running generated code

| State | Description |
|-------|-------------|
| execution_id | Which execution this serves |
| working_dir | Program's working directory |
| allowed_write_paths | Where program can write |
| process | Running process handle |
| timeout | Max execution time |

| Action | Description |
|--------|-------------|
| setup(execution) | Create isolated environment |
| run(program, args) | Execute the program |
| teardown() | Clean up environment |

---

#### 8. Program

**Purpose**: Generated Python code for a card

| State | Description |
|-------|-------------|
| card_id | Which card owns this |
| path | Directory containing code |
| entrypoint | Main file (main.py) |
| dependencies | Required packages |
| source | The actual code |

| Action | Description |
|--------|-------------|
| generate(spec) | Create new program |
| update(changes) | Modify code |
| validate() | Check for errors |

---

#### 9. Chat

**Purpose**: Conversation interface with the AI

| State | Description |
|-------|-------------|
| project_id | Which project context |
| messages | Conversation history |
| pending | Whether awaiting response |

| Action | Description |
|--------|-------------|
| send(message) | User sends message |
| receive(response) | Agent responds |
| clear() | Reset conversation |

---

#### 10. Agent

**Purpose**: AI that understands requests and generates automations

| State | Description |
|-------|-------------|
| model | Which LLM to use |
| system_prompt | Base instructions |
| tools | Available MCP tools |
| context | Current conversation + project state |

| Action | Description |
|--------|-------------|
| respond(message) | Generate response |
| propose_card(spec) | Suggest a new card |
| generate_code(card) | Write program for card |
| refine(feedback) | Improve based on feedback |

---

#### 11. Queue

**Purpose**: Real-time stream of system activity (what the UI shows)

| State | Description |
|-------|-------------|
| project_id | Which project |
| items | Ordered list of events/executions |
| subscribers | Connected WebSocket clients |

| Action | Description |
|--------|-------------|
| push(item) | Add to queue |
| subscribe(client) | Connect client |
| unsubscribe(client) | Disconnect client |
| broadcast(item) | Send to all subscribers |

---

#### 12. MCP

**Purpose**: External tool access via Model Context Protocol

| State | Description |
|-------|-------------|
| project_id | Which project |
| servers | Configured MCP servers |
| connections | Active server connections |

| Action | Description |
|--------|-------------|
| register(server_config) | Add MCP server |
| connect(server_id) | Establish connection |
| disconnect(server_id) | Close connection |
| call_tool(server, tool, args) | Invoke a tool |

---

### Synchronizations

Synchronizations are explicit rules that describe how concepts interact. They make the event flow transparent and debuggable.

#### Project Lifecycle

```
Project.open(id)
  -> Watcher.start(project.path)
  -> Queue.subscribe(project.id)

Project.close()
  -> Watcher.stop()
  -> Queue.unsubscribe()
```

#### File Watching → Events

```
Watcher.file_created(path)
  -> Event.emit(type="file.created", payload={path, folder, filename, size})

Watcher.file_modified(path)
  -> Event.emit(type="file.modified", payload={path, folder, filename, size})

Watcher.file_deleted(path)
  -> Event.emit(type="file.deleted", payload={path, folder, filename})
```

#### Events → Queue (everything visible)

```
Event.emit(*)
  -> Queue.push(event)
  -> Queue.broadcast(event)
```

#### Events → Trigger Matching

```
Event.emit(type="file.*")
  -> for each Card.enabled where Trigger.evaluate(event) == true:
       Trigger.fire(card, event)

Event.emit(type="schedule.tick")
  -> for each Card.enabled where Trigger.type == "schedule":
       if Trigger.cron.matches(now):
         Trigger.fire(card, event)
```

#### Trigger → Execution

```
Trigger.fire(card, event)
  -> Execution.start(card, event)
  -> Event.emit(type="card.triggered", payload={card_id, event_id})
```

#### Execution Lifecycle

```
Execution.start(card, event)
  -> Sandbox.setup(execution)
  -> Sandbox.run(card.program, event.payload)

Sandbox.run.success(stdout, result)
  -> Execution.complete(result)
  -> Sandbox.teardown()

Sandbox.run.failure(stderr, error)
  -> Execution.fail(error)
  -> Sandbox.teardown()

Execution.complete(result)
  -> Card.run_count += 1
  -> Event.emit(type="card.completed", payload={card_id, execution_id, result})

Execution.fail(error) when execution.retry_count > 0
  -> Execution.retry()
  -> execution.retry_count -= 1

Execution.fail(error) when execution.retry_count == 0
  -> Event.emit(type="card.failed", payload={card_id, execution_id, error})
```

#### Chat → Agent → Card Creation

```
Chat.send(message)
  -> Agent.respond(message, context={project, cards, recent_events})

Agent.respond.text(response)
  -> Chat.receive(response)

Agent.propose_card(spec)
  -> Chat.receive(card_preview)
  -> await User.confirm or User.reject

User.confirm(card_spec)
  -> Agent.generate_code(card_spec)
  -> Program.generate(code)
  -> Card.create(card_with_program)
  -> Event.emit(type="card.created", payload={card_id})

User.reject(feedback)
  -> Agent.refine(feedback)
```

#### Agent → MCP Tools

```
Agent.needs_tool(tool_name, args)
  -> MCP.call_tool(server, tool_name, args)

MCP.call_tool.success(result)
  -> Agent.receive_tool_result(result)

MCP.call_tool.failure(error)
  -> Agent.receive_tool_error(error)
```

#### Card Management

```
Card.create(spec)
  -> Trigger.register(card.trigger)
  -> if Trigger.type == "file.*":
       Watcher.add_path(trigger.folder, trigger.pattern)

Card.enable()
  -> Trigger.arm(card.trigger)

Card.disable()
  -> Trigger.disarm(card.trigger)

Card.delete()
  -> Trigger.unregister(card.trigger)
  -> Program.delete(card.program_path)
```

---

## App-Level Configuration

### Location

```
~/.leaf/                              # App-level config directory
├── config.json                       # App settings
└── projects.json                     # Registry of known projects
```

### App Config Schema

```json
// ~/.leaf/config.json
{
  "version": "1.0",
  "theme": "system",                  // "light", "dark", "system"
  "default_model": "gateway/google:gemini-2.5-flash",
  "api_keys": {
    "pydantic_ai_gateway": "..."
  },
  "telemetry": false
}
```

### Projects Registry

```json
// ~/.leaf/projects.json
{
  "version": "1.0",
  "projects": [
    {
      "id": "proj_abc123",
      "name": "Work Reports",
      "path": "/Users/me/Documents/WorkReports",
      "created_at": "2025-01-31T10:00:00Z",
      "last_opened_at": "2025-01-31T14:30:00Z"
    },
    {
      "id": "proj_def456",
      "name": "Personal Finance",
      "path": "/Users/me/Documents/PersonalFinance",
      "created_at": "2025-01-15T09:00:00Z",
      "last_opened_at": "2025-01-30T18:00:00Z"
    }
  ]
}
```

---

## Project Model

### Project Folder Structure

When a user creates or opens a project, LEAF initializes the `.leaf/` folder:

```
~/Documents/WorkReports/              # Project root (user's folder)
├── .leaf/                            # LEAF project data
│   ├── config.json                   # Project-specific settings
│   ├── leaf.db                       # SQLite database (cards, events, executions)
│   ├── programs/                     # Generated Python programs
│   │   ├── csv-analyzer/
│   │   │   ├── main.py
│   │   │   └── pyproject.toml
│   │   └── image-resizer/
│   │       ├── main.py
│   │       └── pyproject.toml
│   ├── logs/                         # Execution logs
│   └── outputs/                      # Generated reports/outputs
├── inbox/                            # Example: user's watched folder
├── reports/                          # Example: user's output folder
└── ...                               # User's other folders and files
```

### Project Config Schema

```json
// .leaf/config.json
{
  "version": "1.0",
  "project_id": "proj_abc123",
  "name": "Work Reports",
  "created_at": "2025-01-31T10:00:00Z",
  "model": "gateway/google:gemini-2.5-flash",  // Override app default
  "mcp_servers": [
    {
      "id": "postgres-main",
      "name": "Main Database",
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-postgres", "postgresql://localhost/mydb"]
    }
  ]
}
```

### Permissions Model

| Location | Read | Write | Execute |
|----------|------|-------|---------|
| Inside project folder (excluding .leaf/) | ✅ | ✅ | ❌ |
| .leaf/programs/ | ✅ | ✅ (LEAF only) | ✅ (sandboxed) |
| .leaf/outputs/ | ✅ | ✅ | ❌ |
| .leaf/ (other) | ✅ | ✅ (LEAF only) | ❌ |
| Outside project folder | ✅ (with permission) | ❌ | ❌ |

### Project Portability

Projects are fully portable:

```bash
# Zip a project
zip -r work-reports.zip ~/Documents/WorkReports

# Move to another machine, unzip
unzip work-reports.zip -d ~/Documents/

# Open in LEAF → automatically registers in projects.json
```

---

## User Interface

### Project Picker (Launch Screen)

When LEAF opens, the user sees the Project Picker:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  LEAF                                                              [⚙]  │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│                         YOUR PROJECTS                                    │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  📁 Work Reports                                                   │ │
│  │     ~/Documents/WorkReports                                        │ │
│  │     Last opened: 2 hours ago                                       │ │
│  │     3 cards • 127 events today                                     │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  📁 Personal Finance                                               │ │
│  │     ~/Documents/PersonalFinance                                    │ │
│  │     Last opened: Yesterday                                         │ │
│  │     2 cards • 15 events today                                      │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  📁 Dev Scripts                                                    │ │
│  │     ~/Dev/Scripts                                                  │ │
│  │     Last opened: 3 days ago                                        │ │
│  │     5 cards • 0 events today                                       │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │  [+ New Project]              [Open Existing Folder]               │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

### Project View (Main UI)

After selecting a project:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  LEAF    Work Reports                           [← Projects]  [⚙]       │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │                          CHAT PANEL                                 │ │
│  │                                                                     │ │
│  │  You: When a CSV file is added to inbox/, analyze it and create    │ │
│  │       a summary report                                              │ │
│  │                                                                     │ │
│  │  LEAF: I'll create a Card for that. Here's what I'm planning:      │ │
│  │                                                                     │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │  📊 CSV Analyzer                                             │   │ │
│  │  │  Trigger: New *.csv files in inbox/                         │   │ │
│  │  │  Action: Analyze with pandas, generate JSON report          │   │ │
│  │  │                                                              │   │ │
│  │  │  [View Code]  [Create Card]  [Modify]                       │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  │                                                                     │ │
│  │  ┌─────────────────────────────────────────────────────────────┐   │ │
│  │  │ Ask LEAF anything...                                    [↵] │   │ │
│  │  └─────────────────────────────────────────────────────────────┘   │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│                                                                          │
├──────────────────────┬───────────────────────────────────────────────────┤
│                      │                                                   │
│   CARDS              │              EVENT QUEUE                          │
│   ─────              │   ─────────────────────────────────────────────   │
│                      │                                                   │
│   ┌────────────────┐ │   ┌──────────────────────────────────────────┐   │
│   │ 📊 CSV Analyzer│ │   │ 10:32:15  file.created  inbox/sales.csv  │   │
│   │    ● Active    │ │   │           → CSV Analyzer                  │   │
│   │    Runs: 12    │ │   │           ⟳ Processing...                │   │
│   └────────────────┘ │   ├──────────────────────────────────────────┤   │
│                      │   │ 10:31:42  card.completed  CSV Analyzer   │   │
│   ┌────────────────┐ │   │           ✓ report_data_20250131.json    │   │
│   │ 🖼 Image Resize│ │   ├──────────────────────────────────────────┤   │
│   │    ● Active    │ │   │ 10:31:40  file.created  inbox/data.csv   │   │
│   │    Runs: 5     │ │   │           → CSV Analyzer                  │   │
│   └────────────────┘ │   │           ✓ Completed (2.3s)             │   │
│                      │   └──────────────────────────────────────────┘   │
│   [+ New Card]       │                                                   │
│                      │                                                   │
└──────────────────────┴───────────────────────────────────────────────────┘
```

### Card Detail View

When a card is selected:

```
┌─────────────────────────────────────────────────────────────────────────┐
│  CSV Analyzer                                            [Edit] [Delete] │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  TRIGGER                                                                 │
│  ──────────────────────────────────────────────────────────────────     │
│  When: New file created                                                  │
│  Folder: inbox/                                                          │
│  Pattern: *.csv                                                          │
│  Status: ● Watching                                                      │
│                                                                          │
│  PROGRAM                                                                 │
│  ──────────────────────────────────────────────────────────────────     │
│  ┌────────────────────────────────────────────────────────────────────┐ │
│  │ main.py                                                             │ │
│  │ ─────────────────────────────────────────────────────────────────  │ │
│  │ import pandas as pd                                                 │ │
│  │ from pathlib import Path                                            │ │
│  │ ...                                                                 │ │
│  └────────────────────────────────────────────────────────────────────┘ │
│  Dependencies: pandas, numpy                                             │
│                                                                          │
│  RECENT EXECUTIONS                                                       │
│  ──────────────────────────────────────────────────────────────────     │
│  │ 10:31:40  sales.csv     ✓ 2.3s   report_sales_20250131.json       │ │
│  │ 10:15:22  inventory.csv ✓ 1.8s   report_inventory_20250131.json   │ │
│  │ 09:45:00  orders.csv    ✗ Error  "Column 'date' not found"        │ │
│                                                                          │
│  [Run Manually]  [View All Executions]                                   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

---

## Database Schema

Each project has its own SQLite database at `.leaf/leaf.db`.

**Note**: No `workspaces` or `projects` table in the project DB — that's managed at the app level in `~/.leaf/projects.json`.

```sql
-- Cards
CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    user_prompt TEXT NOT NULL,
    trigger_config TEXT NOT NULL,      -- JSON
    program_config TEXT NOT NULL,      -- JSON
    program_path TEXT NOT NULL,
    timeout_seconds INTEGER DEFAULT 300,
    retry_count INTEGER DEFAULT 3,
    enabled BOOLEAN DEFAULT TRUE,
    allowed_outputs TEXT NOT NULL,     -- JSON array
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_run_at DATETIME,
    run_count INTEGER DEFAULT 0
);

-- Events
CREATE TABLE events (
    id TEXT PRIMARY KEY,
    type TEXT NOT NULL,
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    payload TEXT NOT NULL,             -- JSON
    status TEXT DEFAULT 'pending',     -- pending, processing, completed, failed
    matched_cards TEXT,                -- JSON array of card IDs
    execution_id TEXT,
    parent_event_id TEXT REFERENCES events(id)
);

-- Executions (card runs)
CREATE TABLE executions (
    id TEXT PRIMARY KEY,
    card_id TEXT REFERENCES cards(id),
    event_id TEXT REFERENCES events(id),
    status TEXT NOT NULL,              -- pending, running, completed, failed
    started_at DATETIME,
    completed_at DATETIME,
    result TEXT,                       -- JSON
    error TEXT,
    stdout TEXT,
    stderr TEXT
);

-- MCP Server configurations (per-project)
CREATE TABLE mcp_servers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    type TEXT NOT NULL,                -- stdio, http
    command TEXT,
    args TEXT,                         -- JSON array
    url TEXT,
    enabled BOOLEAN DEFAULT TRUE,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Chat history
CREATE TABLE chat_messages (
    id TEXT PRIMARY KEY,
    role TEXT NOT NULL,                -- user, assistant
    content TEXT NOT NULL,
    metadata TEXT,                     -- JSON (card_id if card was created, etc.)
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_events_type ON events(type);
CREATE INDEX idx_events_status ON events(status);
CREATE INDEX idx_events_timestamp ON events(timestamp);
CREATE INDEX idx_executions_card ON executions(card_id);
CREATE INDEX idx_executions_status ON executions(status);
CREATE INDEX idx_chat_created ON chat_messages(created_at);
```

---

## Card Model

A Card represents a complete automation: the trigger, the code, and the expected behavior.

### Card Definition Schema

```python
from pydantic import BaseModel
from typing import Literal
from datetime import datetime

class TriggerConfig(BaseModel):
    type: Literal["file_created", "file_modified", "schedule", "manual"]
    folder: str | None = None          # Relative to project root
    pattern: str | None = None         # Glob pattern, e.g., "*.csv"
    cron: str | None = None            # For schedule triggers

class GeneratedProgram(BaseModel):
    language: Literal["python"] = "python"
    entrypoint: str = "main.py"        # Relative to program folder
    dependencies: list[str] = []       # pip packages

class Card(BaseModel):
    id: str
    name: str
    description: str

    # What the user originally asked for
    user_prompt: str

    # Trigger configuration
    trigger: TriggerConfig

    # Generated program
    program: GeneratedProgram
    program_path: str                  # e.g., ".leaf/programs/csv-analyzer"

    # Execution settings
    timeout_seconds: int = 300
    retry_count: int = 3
    enabled: bool = True

    # Allowed interactions (AG-UI style constraints)
    allowed_outputs: list[str] = ["message", "progress", "file_created"]

    # Metadata
    created_at: datetime
    updated_at: datetime
    last_run_at: datetime | None = None
    run_count: int = 0
```

### Example Card

```json
{
  "id": "card_abc123",
  "name": "CSV Analyzer",
  "description": "Analyzes CSV files and generates summary reports",
  "user_prompt": "When a CSV file is added to the inbox folder, analyze it and create a summary report with row count, column stats, and any anomalies",
  "trigger": {
    "type": "file_created",
    "folder": "inbox",
    "pattern": "*.csv"
  },
  "program": {
    "language": "python",
    "entrypoint": "main.py",
    "dependencies": ["pandas", "numpy"]
  },
  "program_path": ".leaf/programs/csv-analyzer",
  "timeout_seconds": 300,
  "retry_count": 3,
  "enabled": true,
  "allowed_outputs": ["message", "progress", "file_created"],
  "created_at": "2025-01-31T10:00:00Z",
  "updated_at": "2025-01-31T10:00:00Z",
  "last_run_at": null,
  "run_count": 0
}
```

---

## Event System

### Event Types

| Type | Payload | Description |
|------|---------|-------------|
| `file.created` | `{path, folder, filename, size}` | New file detected |
| `file.modified` | `{path, folder, filename, size}` | File changed |
| `file.deleted` | `{path, folder, filename}` | File removed |
| `schedule.triggered` | `{cron, scheduled_time}` | Cron schedule fired |
| `card.triggered` | `{card_id, trigger_event_id}` | Card execution started |
| `card.completed` | `{card_id, execution_id, result}` | Card execution finished |
| `card.failed` | `{card_id, execution_id, error}` | Card execution failed |
| `user.chat` | `{message}` | User sent chat message |
| `agent.response` | `{message, card_id?}` | Agent responded |

### Event Schema

```python
from pydantic import BaseModel
from typing import Literal, Any
from datetime import datetime

class Event(BaseModel):
    id: str
    type: str
    timestamp: datetime
    payload: dict[str, Any]

    # Processing state
    status: Literal["pending", "processing", "completed", "failed"] = "pending"
    matched_cards: list[str] = []

    # If this event triggered an execution
    execution_id: str | None = None

    # Parent event (for card.completed -> file.created chain)
    parent_event_id: str | None = None
```

### Event Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  File System                                                     │
│  ~/project/inbox/data.csv (created)                             │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  File Watcher (watchfiles)                                       │
│  Detects: CREATE inbox/data.csv                                  │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Event Bus                                                       │
│  Event: {type: "file.created", path: "inbox/data.csv", ...}     │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Card Matcher                                                    │
│  Matches: CSV Analyzer card (pattern: "*.csv", folder: "inbox") │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Temporal Workflow                                               │
│  Starts: CardExecutionWorkflow(card_id, event)                  │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Sandbox Executor                                                │
│  Runs: uv run main.py "inbox/data.csv" "/project" "/outputs"   │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  Event Bus                                                       │
│  Event: {type: "card.completed", result: {...}}                 │
└─────────────────────────┬───────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│  WebSocket                                                       │
│  Broadcasts to UI → Event Queue updates in real-time            │
└─────────────────────────────────────────────────────────────────┘
```

---

## API Routes

### App-Level Routes

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/app/config` | Get app configuration |
| PUT | `/api/app/config` | Update app configuration |
| GET | `/api/projects` | List all known projects |
| POST | `/api/projects` | Create new project (init folder) |
| POST | `/api/projects/open` | Open existing folder as project |
| DELETE | `/api/projects/{id}` | Remove project from registry (doesn't delete files) |

### Project-Level Routes (require active project)

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/project` | Get current project info |
| PUT | `/api/project` | Update project settings |
| GET | `/api/cards` | List all cards in project |
| POST | `/api/cards` | Create a new card |
| GET | `/api/cards/{id}` | Get card details |
| PUT | `/api/cards/{id}` | Update card |
| DELETE | `/api/cards/{id}` | Delete card |
| POST | `/api/cards/{id}/run` | Manually trigger card |
| GET | `/api/events` | List events (with pagination) |
| GET | `/api/events/{id}` | Get event details |
| GET | `/api/executions` | List executions |
| GET | `/api/executions/{id}` | Get execution details |
| POST | `/api/chat` | Send chat message |
| GET | `/api/chat/history` | Get chat history |
| WS | `/ws` | WebSocket for real-time events |

---

## Project Structure

```
leaf/
├── tauri/                          # Tauri shell
│   ├── src/
│   │   └── main.rs
│   ├── tauri.conf.json
│   └── Cargo.toml
│
├── frontend/                       # React frontend
│   ├── src/
│   │   ├── components/
│   │   │   ├── ProjectPicker.tsx   # Project selection screen
│   │   │   ├── ChatPanel.tsx
│   │   │   ├── CardList.tsx
│   │   │   ├── CardDetail.tsx
│   │   │   ├── EventQueue.tsx
│   │   │   └── Settings.tsx
│   │   ├── hooks/
│   │   │   ├── useWebSocket.ts
│   │   │   ├── useProjects.ts
│   │   │   └── useCards.ts
│   │   ├── App.tsx
│   │   └── main.tsx
│   ├── package.json
│   └── vite.config.ts
│
├── src/
│   └── leaf/
│       ├── __init__.py
│       ├── main.py                 # FastAPI app entry
│       │
│       ├── api/
│       │   ├── __init__.py
│       │   ├── app_routes.py       # App-level routes (/api/app, /api/projects)
│       │   ├── project_routes.py   # Project-level routes (/api/cards, etc.)
│       │   └── websocket.py        # WebSocket handler
│       │
│       ├── core/
│       │   ├── __init__.py
│       │   ├── config.py           # App + project config management
│       │   └── events.py           # Event bus
│       │
│       ├── projects/
│       │   ├── __init__.py
│       │   ├── manager.py          # Project CRUD, initialization
│       │   ├── registry.py         # projects.json management
│       │   └── context.py          # Current project context
│       │
│       ├── watcher/
│       │   ├── __init__.py
│       │   └── file_watcher.py     # watchfiles integration
│       │
│       ├── cards/
│       │   ├── __init__.py
│       │   ├── models.py           # Card, Trigger schemas
│       │   ├── registry.py         # Card CRUD
│       │   └── matcher.py          # Event → Card matching
│       │
│       ├── execution/
│       │   ├── __init__.py
│       │   ├── sandbox.py          # Sandboxed execution
│       │   └── workflows.py        # Temporal workflows
│       │
│       ├── agent/
│       │   ├── __init__.py
│       │   ├── leaf_agent.py       # PydanticAI agent
│       │   ├── prompts.py          # System prompts
│       │   └── primitives.py       # Interaction types
│       │
│       ├── mcp/
│       │   ├── __init__.py
│       │   └── client.py           # MCP server management
│       │
│       └── db/
│           ├── __init__.py
│           ├── models.py           # SQLModel schemas
│           └── session.py          # Database session (per-project)
│
├── tests/
│   ├── test_projects.py
│   ├── test_cards.py
│   ├── test_execution.py
│   └── test_agent.py
│
├── pyproject.toml
├── uv.lock
└── README.md
```

---

## Implementation Phases

### Phase 1: Foundation
- [ ] FastAPI app skeleton with WebSocket support
- [ ] App-level config management (~/.leaf/)
- [ ] Project registry (projects.json CRUD)
- [ ] Project initialization (create .leaf/ folder structure)
- [ ] Per-project SQLite database setup with SQLModel
- [ ] Basic project switching (set current project context)

### Phase 2: File Watching & Events
- [ ] watchfiles integration for folder monitoring (per-project)
- [ ] Event bus (asyncio Queue + broadcast)
- [ ] Event persistence to project SQLite
- [ ] WebSocket streaming of events to frontend
- [ ] Event Queue UI component with real-time updates

### Phase 3: Cards & Triggers
- [ ] Card schema and CRUD API
- [ ] Trigger matching (event → cards)
- [ ] Card list and detail UI
- [ ] Manual trigger ("Run Now" button)

### Phase 4: PydanticAI Agent
- [ ] PydanticAI agent setup with AI Gateway
- [ ] Chat interface UI
- [ ] Card creation from natural language
- [ ] Python code generation
- [ ] Interaction primitives (confirmation, choice, etc.)

### Phase 5: Execution Engine
- [ ] Sandbox executor (UV-managed venv creation, isolated execution)
- [ ] Temporal integration (embedded server)
- [ ] CardExecutionWorkflow implementation
- [ ] Execution history and logging
- [ ] Retry logic

### Phase 6: MCP Integration
- [ ] MCP client manager
- [ ] MCP server configuration UI
- [ ] Agent with MCP tools
- [ ] Cards that query external data

### Phase 7: Polish
- [ ] Project Picker UI
- [ ] Error handling and user-friendly messages
- [ ] Onboarding flow (first project setup)
- [ ] System tray integration
- [ ] Keyboard shortcuts
- [ ] Dark mode

---

## Dependencies

```toml
# pyproject.toml
[project]
name = "leaf"
version = "0.1.0"
description = "Local Event-Driven Automation Framework"
requires-python = ">=3.11"
dependencies = [
    # Web framework
    "fastapi>=0.115.0",
    "uvicorn[standard]>=0.32.0",
    "websockets>=14.0",

    # AI
    "pydantic-ai>=0.0.30",

    # Database
    "sqlmodel>=0.0.22",
    "aiosqlite>=0.20.0",

    # File watching
    "watchfiles>=1.0.0",

    # Workflow engine
    "temporalio>=1.7.0",

    # Utilities
    "python-slugify>=8.0.0",
    "python-dotenv>=1.0.0",
]

[dependency-groups]
dev = [
    "pytest>=8.0.0",
    "pytest-asyncio>=0.24.0",
    "ruff>=0.8.0",
]

[tool.ruff]
line-length = 100
target-version = "py311"

[tool.ruff.lint]
select = ["E", "F", "I", "UP"]

[tool.pytest.ini_options]
asyncio_mode = "auto"
```

---

## Environment Variables

```bash
# Required
PYDANTIC_AI_GATEWAY_API_KEY=your-api-key

# Optional
LEAF_PORT=8000
LEAF_LOG_LEVEL=info
LEAF_MODEL=gateway/google:gemini-2.5-flash
LEAF_CONFIG_DIR=~/.leaf              # Override app config location
```

---

## Security Considerations

### Project Isolation

1. **Separate databases**: Each project has its own SQLite database
2. **Sandboxed to folder**: Generated code can only write to project folder
3. **Independent event queues**: Events don't leak between projects

### Sandboxed Execution

1. **Isolated environments**: Each card's program runs in its own UV-managed environment
2. **Working directory**: Programs can only write to project folder
3. **Timeout**: All executions have configurable timeouts
4. **No shell expansion**: Commands run via `uv run` without shell=True
5. **Dependency isolation**: UV lockfiles ensure reproducible, isolated dependencies per card

### Code Generation Safety

1. **Review before creation**: Users see generated code before confirming
2. **Dependency allowlist**: Option to restrict pip packages
3. **Audit log**: All executions recorded with full stdout/stderr

### MCP Security

1. **Per-project servers**: MCP servers configured per-project
2. **User-configured only**: Only servers explicitly added by user
3. **Credential isolation**: API keys stored in system keychain (future)

---

## Open Questions

1. **Code editing**: Should users be able to edit generated code directly in the UI?
2. **Card versioning**: Track changes to cards/programs over time?
3. **Card sharing**: Export/import cards as JSON bundles?
4. **Project templates**: Pre-configured projects for common use cases?
5. **Remote Temporal**: Option to connect to external Temporal cluster?

---

## Getting Started

### Prerequisites

```bash
# Install UV
curl -LsSf https://astral.sh/uv/install.sh | sh

# Install Temporal CLI
brew install temporal

# Install Node.js (for frontend)
brew install node
```

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourorg/leaf.git
cd leaf

# Install Python dependencies with UV
uv sync

# Install frontend dependencies
cd frontend && npm install && cd ..

# Start all services (in separate terminals)

# Terminal 1: Temporal dev server
temporal server start-dev

# Terminal 2: FastAPI backend
uv run uvicorn leaf.main:app --reload

# Terminal 3: Tauri dev mode
cd tauri && cargo tauri dev
```

### Quick Commands

```bash
# Add a new dependency
uv add <package>

# Add a dev dependency
uv add --group dev <package>

# Run tests
uv run pytest

# Run linting
uv run ruff check .

# Run the app
uv run python -m leaf.main
```

### First Project End-to-End

1. Launch LEAF → Project Picker appears
2. Click "New Project" → Select/create a folder
3. LEAF initializes `.leaf/` in that folder
4. Enter the project → Main UI appears
5. Describe an automation in chat
6. Agent generates card + Python code
7. User confirms creation
8. Add a file to watched folder
9. Card executes via `uv run main.py`
10. Event appears in queue with result
