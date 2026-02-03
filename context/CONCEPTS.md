# LEAF Concepts & Synchronizations

This document describes LEAF's architecture using the **Concepts & Synchronizations** model — a software design approach where independent, self-contained modules (concepts) are connected through explicit synchronization rules.

> Reference: [MIT Concepts & Synchronizations](https://news.mit.edu/2025/mit-researchers-propose-new-model-for-legible-modular-software-1106)

---

## Why This Model?

| Benefit | Description |
|---------|-------------|
| **Transparency** | Users can see exactly what triggers what |
| **Debugging** | Trace any behavior through the sync chain |
| **Extensibility** | Add new concepts (webhooks, schedules) with new syncs |
| **Testing** | Each concept tests in isolation; syncs test separately |
| **AI-Friendly** | The agent can reason about and explain syncs to users |

---

## Concept Map

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              LEAF CONCEPT MAP                                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                              │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐   │
│  │ Project │────────▶│ Watcher │────────▶│  Event  │────────▶│  Queue  │   │
│  └─────────┘  opens   └─────────┘ emits   └─────────┘ pushes  └─────────┘   │
│       │                                        │                     │       │
│       │ has many                               │ matches             │       │
│       ▼                                        ▼                     ▼       │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐         ┌─────────┐   │
│  │   MCP   │◀────────│  Agent  │◀────────│  Chat   │         │   UI    │   │
│  └─────────┘  tools   └─────────┘ responds│ Session │         └─────────┘   │
│                            │               └─────────┘                       │
│                            │                    │ has many                   │
│                            │ generates          ▼                            │
│                            │               ┌─────────┐                       │
│                            │               │ Message │                       │
│                            ▼               └─────────┘                       │
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
│  Card.source_session_id ──────────────────────▶ ChatSession                 │
│  (provenance: which conversation created this card)                         │
│                                                                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Concepts

### 1. Project

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

### 2. Watcher

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

### 3. Event

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

### 4. Card

**Purpose**: An automation definition (trigger + program + settings)

| State | Description |
|-------|-------------|
| id | Unique identifier |
| name | Human-readable name |
| description | What it does |
| user_prompt | Original user request |
| source_session_id | ChatSession that created this card (provenance) |
| enabled | Whether active |
| program_path | Path to generated code |
| run_count | How many times executed |

| Action | Description |
|--------|-------------|
| create(spec, session_id) | Create new card linked to session |
| update(changes) | Modify card |
| enable() | Activate card |
| disable() | Deactivate card |
| delete() | Remove card |

---

### 5. Trigger

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

### 6. Execution

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

### 7. Sandbox

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

### 8. Program

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

### 9. ChatSession

**Purpose**: A discrete conversation with the AI agent within a project

A ChatSession represents a single conversation thread. Users can have multiple sessions per project, allowing them to work on different topics, review past conversations, and maintain clear provenance for cards created through chat.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| project_id | Which project this session belongs to |
| title | Human-readable title (auto-generated or user-defined) |
| messages | Ordered list of messages in this session |
| status | active, archived |
| created_at | When the session started |
| updated_at | Last message timestamp |

| Action | Description |
|--------|-------------|
| create(project_id) | Start a new session |
| send(message) | User sends a message |
| receive(response) | Agent responds |
| rename(title) | Update the session title |
| archive() | Mark session as archived (hidden from main list) |
| unarchive() | Restore an archived session |
| delete() | Permanently remove session and messages |

| Emits | Description |
|-------|-------------|
| session.created | New session started |
| session.message | Message added to session |
| session.archived | Session was archived |

**Relationships:**
- A Project has many ChatSessions
- A ChatSession has many Messages
- A Card references the ChatSession that created it (via `source_session_id`)

---

### 10. Message

**Purpose**: A single message within a chat session

| State | Description |
|-------|-------------|
| id | Unique identifier |
| session_id | Which session this belongs to |
| role | user, assistant, system, tool |
| content | Message text content |
| tool_calls | Tool invocations (if assistant message) |
| tool_result | Tool response (if tool message) |
| metadata | Additional data (card_id if card was created, etc.) |
| created_at | When the message was sent |

| Action | Description |
|--------|-------------|
| create(session_id, role, content) | Add message to session |

---

### 11. Agent

**Purpose**: AI that understands requests and generates automations

| State | Description |
|-------|-------------|
| model | Which LLM to use |
| system_prompt | Base instructions |
| tools | Available MCP tools |
| session | Current ChatSession context |

| Action | Description |
|--------|-------------|
| respond(session, message) | Generate response within session context |
| propose_card(spec) | Suggest a new card |
| generate_code(card) | Write program for card |
| refine(feedback) | Improve based on feedback |

---

### 12. Queue

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

### 13. MCP

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

## Synchronizations

Synchronizations are explicit rules that describe how concepts interact. They make the event flow transparent and debuggable.

### Project Lifecycle

```
Project.open(id)
  -> Watcher.start(project.path)
  -> Queue.subscribe(project.id)

Project.close()
  -> Watcher.stop()
  -> Queue.unsubscribe()
```

When a project opens, LEAF starts watching its folders and subscribes to its event queue. When it closes, everything stops cleanly.

---

### File Watching → Events

```
Watcher.file_created(path)
  -> Event.emit(type="file.created", payload={path, folder, filename, size})

Watcher.file_modified(path)
  -> Event.emit(type="file.modified", payload={path, folder, filename, size})

Watcher.file_deleted(path)
  -> Event.emit(type="file.deleted", payload={path, folder, filename})
```

The Watcher detects filesystem changes and converts them into Events. This is the primary input to the system.

---

### Events → Queue

```
Event.emit(*)
  -> Queue.push(event)
  -> Queue.broadcast(event)
```

Every event is pushed to the Queue and broadcast to all connected UI clients. This is what makes the Event Queue real-time.

---

### Events → Trigger Matching

```
Event.emit(type="file.*")
  -> for each Card.enabled where Trigger.evaluate(event) == true:
       Trigger.fire(card, event)

Event.emit(type="schedule.tick")
  -> for each Card.enabled where Trigger.type == "schedule":
       if Trigger.cron.matches(now):
         Trigger.fire(card, event)
```

When file events occur, LEAF checks all enabled Cards to see if their Triggers match. Matching triggers fire.

---

### Trigger → Execution

```
Trigger.fire(card, event)
  -> Execution.start(card, event)
  -> Event.emit(type="card.triggered", payload={card_id, event_id})
```

When a Trigger fires, an Execution begins. This is also emitted as an event so the UI can show it.

---

### Execution Lifecycle

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

Executions create sandboxed environments, run the program, and handle success/failure. Retries are automatic up to the configured limit.

---

### ChatSession Lifecycle

```
ChatSession.create(project_id)
  -> Generate title from first message (or "New Chat")
  -> Event.emit(type="session.created", payload={session_id})

ChatSession.archive()
  -> Event.emit(type="session.archived", payload={session_id})

ChatSession.delete()
  -> Message.delete_all(session_id)
  -> Event.emit(type="session.deleted", payload={session_id})
```

Sessions are created when users start new conversations. Archiving hides sessions from the main list without deleting history.

---

### ChatSession → Agent → Card Creation

```
ChatSession.send(message)
  -> Message.create(session_id, role="user", content=message)
  -> Agent.respond(session, message, context={project, cards, recent_events})

Agent.respond.text(response)
  -> Message.create(session_id, role="assistant", content=response)
  -> ChatSession.update(updated_at=now)

Agent.propose_card(spec)
  -> Message.create(session_id, role="assistant", content=card_preview)
  -> await User.confirm or User.reject

User.confirm(card_spec)
  -> Agent.generate_code(card_spec)
  -> Program.generate(code)
  -> Card.create(card_with_program, source_session_id=session.id)
  -> Message.create(session_id, role="assistant", metadata={card_id})
  -> Event.emit(type="card.created", payload={card_id, session_id})

User.reject(feedback)
  -> Message.create(session_id, role="user", content=feedback)
  -> Agent.refine(feedback)
```

The ChatSession/Agent flow is how users create Cards through natural language. Messages are persisted to the session, creating a complete history. Cards link back to their originating session for provenance.

---

### Agent → MCP Tools

```
Agent.needs_tool(tool_name, args)
  -> MCP.call_tool(server, tool_name, args)

MCP.call_tool.success(result)
  -> Agent.receive_tool_result(result)

MCP.call_tool.failure(error)
  -> Agent.receive_tool_error(error)
```

The Agent can access external data through MCP servers. Tool calls are explicit and their results flow back to the Agent.

---

### Card Management

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

When Cards are created, enabled, disabled, or deleted, the corresponding Triggers and Watchers are updated. This keeps the system in sync.

---

## Example Flow: CSV File Triggers Analysis

Here's how the concepts and syncs work together for a typical use case:

```
1. User drops sales.csv into inbox/

2. Watcher.file_created("inbox/sales.csv")
   -> Event.emit(type="file.created", payload={path: "inbox/sales.csv", ...})

3. Event.emit(*)
   -> Queue.push(event)           # UI shows "file.created inbox/sales.csv"
   -> Queue.broadcast(event)

4. Event.emit(type="file.created")
   -> Trigger.evaluate(event) for "CSV Analyzer" card
   -> Trigger matches (folder="inbox", pattern="*.csv")
   -> Trigger.fire(card, event)

5. Trigger.fire(card, event)
   -> Execution.start(card, event)
   -> Event.emit(type="card.triggered")  # UI shows "CSV Analyzer triggered"

6. Execution.start(card, event)
   -> Sandbox.setup(execution)
   -> Sandbox.run(program, {file: "inbox/sales.csv"})

7. Sandbox.run.success(stdout, result)
   -> Execution.complete(result)
   -> Sandbox.teardown()

8. Execution.complete(result)
   -> Card.run_count += 1
   -> Event.emit(type="card.completed", payload={result: "report.json created"})

9. Event.emit(type="card.completed")
   -> Queue.push(event)           # UI shows "CSV Analyzer completed"
   -> Queue.broadcast(event)
```

Every step is visible in the Event Queue. Every transition is an explicit synchronization.
