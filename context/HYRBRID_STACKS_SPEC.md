# LEAF Stacks & Cards — Implementation Specification

## Specification v2.0 (Rust / Tauri)

---

## Overview

This spec extends LEAF with a **Stack & Cards** workflow model inspired by Apple HyperCard, plus an **Artifact** tracking system and **Guided Workflow Setup**. It replaces the original Hybrid Stacks spec with Rust-native types, Tauri IPC commands, and alignment with the existing codebase.

### What Changes

1. **Stacks** — A new first-class concept. A Stack holds a trigger + an ordered pipeline of Cards. Users create, name, enable, and disable Stacks.
2. **Cards (redefined)** — A Card becomes a single program step within a Stack. Cards no longer have triggers. Every Card belongs to exactly one Stack.
3. **Artifacts** — Lightweight file tracking across the project. Files created by cards, the agent, or the user are registered in a database table with metadata and provenance.
4. **Guided Workflow Setup** — The agent walks users step-by-step through creating a complete workflow: folders, programs, stacks, and cards.

### Why Stack & Cards (not parent_id/subcards)

The original spec overloaded `Card` to mean both "a single automation" and "a container for child automations." Making Stack a first-class concept eliminates:

- **Ambiguous identity** — is a card a step or a pipeline?
- **Special-case execution** — "if has children, ignore own program"
- **Nesting policy questions** — nesting is structurally impossible (no FK exists for Stack-in-Stack)

### Design Principles

- **HyperCard honest** — A Stack contains Cards. Cards don't contain Cards.
- **One execution path** — The runner always executes Stack -> Cards in order. No branching.
- **Nesting is structurally impossible** — The schema enforces the depth limit.
- **Clean break** — The old `cards` and `executions` tables are dropped and replaced. No migration of existing data.
- **File-first** — Artifacts are real files on disk. The DB is an index, not the source of truth.
- **Guided, not magic** — The agent proposes each step; the user confirms.

---

## 1. Concept Map

```
+----------------------------------------------------------------------------+
|                         LEAF CONCEPT MAP (updated)                         |
+----------------------------------------------------------------------------+
|                                                                            |
|  +---------+         +---------+         +---------+       +---------+    |
|  | Project |-------->| Watcher |-------->|  Event  |------>|  Queue  |    |
|  +---------+  opens   +---------+ emits   +---------+pushes +---------+    |
|       |                                        |                   |       |
|       | has many                               | matches           |       |
|       v                                        v                   v       |
|  +---------+         +---------+         +---------+       +---------+    |
|  |  Stack  |<--------|TriggerCfg|<--------|  Event  |       |   UI    |    |
|  +---------+  has     +---------+evaluates+---------+       +---------+    |
|       |                                                                    |
|       | contains (ordered)                                                 |
|       v                                                                    |
|  +---------+         +---------+         +---------+                      |
|  |  Card   |-------->| Program |-------->| Sandbox |                      |
|  +---------+  has     +---------+  runs   +---------+                      |
|       |                                        |                           |
|       | produces                               | creates files             |
|       v                                        v                           |
|  +---------+                              +---------+                      |
|  |Artifact |<-----------------------------|Artifact |                      |
|  +---------+                              +---------+                      |
|                                                                            |
|  +---------+         +---------+         +---------+                      |
|  |   MCP   |<--------|  Agent  |<--------|  Chat   |                      |
|  +---------+  tools   +---------+responds | Session |                      |
|                            |               +---------+                      |
|                            | creates                                       |
|                            v                                               |
|                       +---------+                                          |
|                       |  Stack  | (with Cards)                             |
|                       +---------+                                          |
|                                                                            |
|  Stack.source_session_id -----------> ChatSession                          |
|  (provenance: which conversation created this stack)                       |
|                                                                            |
+----------------------------------------------------------------------------+
```

---

## 2. Stack

**Purpose**: A trigger + an ordered pipeline of Cards. The unit users create, name, enable, and disable.

| State | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier |
| `project_id` | `Uuid` | Which project this belongs to |
| `name` | `String` | Human-readable name (e.g., "CSV Pipeline") |
| `description` | `String` | What this workflow does |
| `trigger` | `TriggerConfig` | FileCreated, FileModified, Schedule, Manual |
| `enabled` | `bool` | Whether this stack is active |
| `source_session_id` | `Option<Uuid>` | ChatSession that created this stack (provenance) |
| `created_at` | `DateTime<Utc>` | When created |
| `updated_at` | `DateTime<Utc>` | Last modified |

| Action | Description |
|--------|-------------|
| `create(project_id, name, trigger)` | Create a new stack |
| `update(changes)` | Modify stack metadata or trigger |
| `enable()` | Activate the stack (trigger starts matching) |
| `disable()` | Deactivate the stack (trigger stops matching) |
| `delete()` | Remove stack and all its cards (CASCADE) |

**Key point:** A Stack always has at least one Card. When the agent creates a stack, it creates the first card simultaneously.

---

## 3. Card (redefined)

**Purpose**: A single program step within a Stack. Has no trigger of its own — it runs when its Stack fires.

| State | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier |
| `stack_id` | `Uuid` | Which stack this belongs to (NOT NULL) |
| `name` | `String` | Human-readable step name (e.g., "Validate Input") |
| `description` | `String` | What this step does |
| `program` | `ProgramConfig` | Language, entrypoint, timeout, retries |
| `program_path` | `String` | Explicit path relative to `.leaf/programs/` |
| `position` | `i32` | 0-indexed order within the stack |
| `enabled` | `bool` | Whether this card runs (disabled cards are skipped) |
| `created_at` | `DateTime<Utc>` | When created |
| `updated_at` | `DateTime<Utc>` | Last modified |

| Action | Description |
|--------|-------------|
| `create(stack_id, name, program, position)` | Add a card to a stack |
| `update(changes)` | Modify card program or metadata |
| `reorder(new_position)` | Move within the stack |
| `enable()` | Include in execution |
| `disable()` | Skip during execution |
| `delete()` | Remove card from stack |

**What changed from the old Card:**
- `trigger` moved to Stack — Cards don't have triggers
- `stack_id` replaces the old `project_id` ownership — every Card belongs to a Stack
- `session_id` (provenance) moved to Stack as `source_session_id`
- `position` is always meaningful
- `program_path` is explicit (stored in DB, not derived from card ID)

---

## 4. StackExecution & CardExecution

### StackExecution

**Purpose**: A single run of a Stack's card pipeline.

| State | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier |
| `stack_id` | `Uuid` | Which stack ran |
| `event_id` | `Option<Uuid>` | What triggered it (null for manual) |
| `status` | `ExecutionStatus` | Pending, Running, Success, Failed, Timeout, Cancelled |
| `card_count` | `i32` | Total enabled cards at start |
| `completed_cards` | `i32` | How many finished successfully |
| `failed_at_position` | `Option<i32>` | Which position failed (null if success) |
| `error` | `Option<String>` | Error message from the failing card |
| `started_at` | `DateTime<Utc>` | When execution began |
| `completed_at` | `Option<DateTime<Utc>>` | When execution finished |
| `duration_ms` | `Option<u64>` | Total pipeline duration |

### CardExecution

**Purpose**: A single run of one Card within a StackExecution.

| State | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier |
| `stack_execution_id` | `Uuid` | Parent pipeline run |
| `card_id` | `Uuid` | Which card ran |
| `position` | `i32` | Position at time of execution |
| `status` | `ExecutionStatus` | Pending, Running, Success, Failed, Timeout, Cancelled |
| `stdout` | `String` | Program stdout |
| `stderr` | `String` | Program stderr |
| `exit_code` | `Option<i32>` | Process exit code |
| `started_at` | `DateTime<Utc>` | When started |
| `completed_at` | `Option<DateTime<Utc>>` | When finished |
| `duration_ms` | `Option<u64>` | Step duration |

---

## 5. Artifacts

**Purpose**: Tracked file or directory within the project. Richer tracking with status lifecycle.

| State | Type | Description |
|-------|------|-------------|
| `id` | `Uuid` | Unique identifier |
| `project_id` | `Uuid` | Which project |
| `path` | `String` | Relative to project root (e.g., `output/report.csv`) |
| `filename` | `String` | Just the filename (e.g., `report.csv`) |
| `artifact_type` | `ArtifactType` | `File` or `Directory` |
| `mime_type` | `Option<String>` | e.g., `text/csv`, `application/json` (null for dirs) |
| `size_bytes` | `Option<u64>` | File size (null for dirs) |
| `created_by` | `String` | `"agent"`, `"card:<card_id>"`, `"user"` |
| `created_by_execution_id` | `Option<Uuid>` | If created during a stack execution |
| `status` | `ArtifactStatus` | `Active`, `Deleted`, `Modified` |
| `created_at` | `DateTime<Utc>` | When first tracked |
| `modified_at` | `Option<DateTime<Utc>>` | When last modified |
| `metadata` | `Option<serde_json::Value>` | Arbitrary key-value pairs |

| Action | Description |
|--------|-------------|
| `register(path, type, created_by)` | Track a new file/directory |
| `update_modified(path)` | Mark as modified, update timestamp |
| `mark_deleted(path)` | Mark as deleted (file removed from disk) |
| `scan(project_path)` | Detect and register existing files on demand |

### How Artifacts Are Created

1. **Agent creates a file/folder** — The `write_file` and `create_folder` tools register artifacts after writing.
2. **Card execution creates a file** — The runner snapshots the project folder before/after execution, registering new/modified files.
3. **User adds a file** — The file watcher detects `file_created`/`file_modified` events and registers artifacts.
4. **File deleted** — The watcher detects `file_deleted` and marks the artifact's status as `Deleted`.

### MIME Type Detection

Use Rust's built-in approach or the `mime_guess` crate:

```rust
fn detect_mime_type(filename: &str) -> Option<String> {
    mime_guess::from_path(filename)
        .first()
        .map(|m| m.to_string())
}
```

---

## 6. Database Schema

### Clean Break Strategy

Migration v2 drops the old `cards` and `executions` tables entirely and creates the new schema. The `events`, `chat_sessions`, `messages`, and `mcp_servers` tables are unchanged.

```sql
-- Migration v2: Stacks & Cards

-- Drop old tables
DROP TABLE IF EXISTS executions;
DROP TABLE IF EXISTS cards;

-- Stacks: the trigger + pipeline container
CREATE TABLE stacks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    trigger_config TEXT NOT NULL DEFAULT '{"type":"manual"}',
    enabled INTEGER NOT NULL DEFAULT 1,
    source_session_id TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_stacks_project_id ON stacks(project_id);
CREATE INDEX idx_stacks_source_session ON stacks(source_session_id);

-- Cards: individual program steps within a stack
CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    program_path TEXT NOT NULL,
    program_config TEXT NOT NULL DEFAULT '{"language":"typescript","entrypoint":"main.ts","dependencies":[],"timeout_secs":300,"max_retries":3}',
    position INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_cards_stack_id ON cards(stack_id);

-- Stack executions: one run of a full pipeline
CREATE TABLE stack_executions (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    event_id TEXT,
    status TEXT NOT NULL DEFAULT 'pending',
    card_count INTEGER NOT NULL,
    completed_cards INTEGER NOT NULL DEFAULT 0,
    failed_at_position INTEGER,
    error TEXT,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    duration_ms INTEGER,
    FOREIGN KEY (event_id) REFERENCES events(id) ON DELETE SET NULL
);

CREATE INDEX idx_stack_executions_stack_id ON stack_executions(stack_id);
CREATE INDEX idx_stack_executions_status ON stack_executions(status);

-- Card executions: one card's run within a stack execution
CREATE TABLE card_executions (
    id TEXT PRIMARY KEY,
    stack_execution_id TEXT NOT NULL REFERENCES stack_executions(id) ON DELETE CASCADE,
    card_id TEXT NOT NULL REFERENCES cards(id) ON DELETE CASCADE,
    position INTEGER NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    stdout TEXT NOT NULL DEFAULT '',
    stderr TEXT NOT NULL DEFAULT '',
    exit_code INTEGER,
    started_at TEXT NOT NULL,
    completed_at TEXT,
    duration_ms INTEGER
);

CREATE INDEX idx_card_executions_stack_exec ON card_executions(stack_execution_id);
CREATE INDEX idx_card_executions_card_id ON card_executions(card_id);

-- Artifacts: file tracking with status lifecycle
CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    path TEXT NOT NULL,
    filename TEXT NOT NULL,
    artifact_type TEXT NOT NULL,
    mime_type TEXT,
    size_bytes INTEGER,
    created_by TEXT NOT NULL DEFAULT 'user',
    created_by_execution_id TEXT,
    status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    modified_at TEXT,
    metadata TEXT,
    UNIQUE(project_id, path)
);

CREATE INDEX idx_artifacts_project_id ON artifacts(project_id);
CREATE INDEX idx_artifacts_path ON artifacts(path);
CREATE INDEX idx_artifacts_created_by ON artifacts(created_by);
CREATE INDEX idx_artifacts_status ON artifacts(status);
CREATE INDEX idx_artifacts_type ON artifacts(artifact_type);
```

### Existing Tables (unchanged)

- `events` — no changes, but `matched_cards` field semantically becomes "matched stacks" (contains stack IDs instead of card IDs)
- `chat_sessions` — no changes
- `messages` — no changes
- `mcp_servers` — no changes

---

## 7. Rust Types

All types follow the existing project conventions: `#[derive(Debug, Clone, Serialize, Deserialize)]`, `Uuid` for IDs, `DateTime<Utc>` for timestamps.

### Stack

```rust
// In crates/leaf-core/src/types.rs

/// A Stack - a trigger + ordered pipeline of Cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stack {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger: TriggerConfig,
    pub enabled: bool,
    pub source_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Stack {
    pub fn new(
        project_id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        trigger: TriggerConfig,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            project_id,
            name: name.into(),
            description: description.into(),
            trigger,
            enabled: true,
            source_session_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### Card (redefined)

```rust
/// A Card - a single program step within a Stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub id: Uuid,
    pub stack_id: Uuid,
    pub name: String,
    pub description: String,
    pub program: ProgramConfig,
    pub program_path: String,  // Relative to .leaf/programs/
    pub position: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Card {
    pub fn new(
        stack_id: Uuid,
        name: impl Into<String>,
        description: impl Into<String>,
        program_path: impl Into<String>,
        position: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            stack_id,
            name: name.into(),
            description: description.into(),
            program: ProgramConfig::default(),
            program_path: program_path.into(),
            position,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}
```

### StackExecution

```rust
/// A StackExecution - one run of a Stack's card pipeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackExecution {
    pub id: Uuid,
    pub stack_id: Uuid,
    pub event_id: Option<Uuid>,
    pub status: ExecutionStatus,
    pub card_count: i32,
    pub completed_cards: i32,
    pub failed_at_position: Option<i32>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl StackExecution {
    pub fn new(stack_id: Uuid, event_id: Option<Uuid>, card_count: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            stack_id,
            event_id,
            status: ExecutionStatus::Pending,
            card_count,
            completed_cards: 0,
            failed_at_position: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }
}
```

### CardExecution

```rust
/// A CardExecution - one card's run within a StackExecution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardExecution {
    pub id: Uuid,
    pub stack_execution_id: Uuid,
    pub card_id: Uuid,
    pub position: i32,
    pub status: ExecutionStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<u64>,
}

impl CardExecution {
    pub fn new(stack_execution_id: Uuid, card_id: Uuid, position: i32) -> Self {
        Self {
            id: Uuid::new_v4(),
            stack_execution_id,
            card_id,
            position,
            status: ExecutionStatus::Pending,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            started_at: Utc::now(),
            completed_at: None,
            duration_ms: None,
        }
    }
}
```

### Artifact

```rust
/// Artifact type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactType {
    File,
    Directory,
}

/// Artifact status lifecycle
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactStatus {
    #[default]
    Active,
    Modified,
    Deleted,
}

/// An Artifact - a tracked file or directory in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: Uuid,
    pub project_id: Uuid,
    pub path: String,
    pub filename: String,
    pub artifact_type: ArtifactType,
    pub mime_type: Option<String>,
    pub size_bytes: Option<u64>,
    pub created_by: String,
    pub created_by_execution_id: Option<Uuid>,
    pub status: ArtifactStatus,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

impl Artifact {
    pub fn new_file(
        project_id: Uuid,
        path: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
        Self {
            id: Uuid::new_v4(),
            project_id,
            path,
            filename,
            artifact_type: ArtifactType::File,
            mime_type: None,
            size_bytes: None,
            created_by: created_by.into(),
            created_by_execution_id: None,
            status: ArtifactStatus::Active,
            created_at: Utc::now(),
            modified_at: None,
            metadata: None,
        }
    }

    pub fn new_directory(
        project_id: Uuid,
        path: impl Into<String>,
        created_by: impl Into<String>,
    ) -> Self {
        let path = path.into();
        let filename = path.rsplit('/').next().unwrap_or(&path).to_string();
        Self {
            id: Uuid::new_v4(),
            project_id,
            path,
            filename,
            artifact_type: ArtifactType::Directory,
            mime_type: None,
            size_bytes: None,
            created_by: created_by.into(),
            created_by_execution_id: None,
            status: ArtifactStatus::Active,
            created_at: Utc::now(),
            modified_at: None,
            metadata: None,
        }
    }
}
```

### Types Removed

The old `Execution` struct is removed. It is replaced by `StackExecution` + `CardExecution`.

### Types Unchanged

`TriggerConfig`, `ProgramConfig`, `EventType`, `EventPayload`, `EventStatus`, `ExecutionStatus`, `Event`, `Project`, `ChatSession`, `Message`, `McpServer` — all remain as-is.

`TriggerConfig.evaluate()` stays the same — it just lives on `Stack.trigger` instead of the old `Card.trigger`.

---

## 8. Synchronizations

Following the Concepts & Synchronizations model from `context/CONCEPTS.md`.

### Trigger Matching (updated)

```
Event.emit(type="file.*")
  -> for each Stack where Stack.enabled && Stack.trigger.evaluate(event):
       StackExecution.start(stack, event)
```

**Key change:** We iterate over Stacks, not Cards. The self-evaluating trigger pattern is preserved.

### Stack Execution

```
StackExecution.start(stack, event)
  -> cards = Card.list(stack_id, enabled=true, order_by=position)
  -> stack_exec = StackExecution.create(stack, event, card_count=len(cards))
  -> LeafEvent::StackExecutionStarted { stack_id, execution_id, card_count }
  -> for each card in cards:
       CardExecution.run(card, stack_exec, event)

CardExecution.run(card, stack_exec, event)
  -> Sandbox.run(card.program, env={
       LEAF_PROJECT_ROOT,
       LEAF_CARD_ID: card.id,
       LEAF_STACK_ID: stack.id,
       LEAF_EXECUTION_ID: stack_exec.id,
       LEAF_EVENT_PAYLOAD: event.payload,
       LEAF_STEP_INDEX: card.position,
       LEAF_PREV_STDOUT: prev_card.stdout (truncated 10KB),
       LEAF_PREV_OUTPUT_PATH: .leaf/outputs/{stack_exec.id}/step_{position}.out
     })

CardExecution.complete(result)
  -> write stdout to .leaf/outputs/{stack_exec.id}/step_{position}.out
  -> stack_exec.completed_cards += 1
  -> LeafEvent::StackStepCompleted { execution_id, position, card_id, duration_ms }
  -> continue to next card

CardExecution.fail(error)
  -> stack_exec.status = Failed
  -> stack_exec.failed_at_position = position
  -> stack_exec.error = error
  -> LeafEvent::StackExecutionFailed { stack_id, execution_id, failed_position, error }
  -> STOP (do not execute remaining cards)

StackExecution.all_steps_complete()
  -> stack_exec.status = Success
  -> LeafEvent::StackExecutionCompleted { stack_id, execution_id, total_duration_ms }
```

**Single-card stacks** follow the exact same path. No special case — just a pipeline of length 1.

### Stack Management

```
Stack.create(project_id, name, trigger, cards)
  -> insert Stack row
  -> insert Card rows
  -> write program files to .leaf/programs/
  -> if trigger is file-based: Watcher.add_path(trigger.watch_path)
  -> LeafEvent::StackCreated(stack)

Stack.enable()
  -> if trigger is file-based: Watcher.add_path(trigger.watch_path)
  -> LeafEvent::StackEnabled { stack_id }

Stack.disable()
  -> LeafEvent::StackDisabled { stack_id }

Stack.delete()
  -> CASCADE deletes all Cards + CardExecutions + StackExecutions
  -> delete program files from .leaf/programs/
  -> LeafEvent::StackDeleted { stack_id }
```

### Card Management (within a Stack)

```
Card.create(stack_id, name, program, position)
  -> write program file to .leaf/programs/{program_path}
  -> LeafEvent::CardCreated(card)

Card.reorder(new_position)
  -> shift other cards' positions
  -> LeafEvent::CardReordered { card_id, stack_id, old_position, new_position }

Card.delete()
  -> shift remaining cards' positions to close gap
  -> delete program file
  -> LeafEvent::CardDeleted { card_id, stack_id }
```

### Agent -> Stack/Card Creation

```
ChatSession.send(message)
  -> Agent.respond(session, message, context={project, stacks, recent_events})

Agent.propose_stack(spec)
  -> Message.create(session_id, role="assistant", content=stack_preview)
  -> await User.confirm or User.reject

User.confirm(stack_spec)
  -> for each card in stack_spec.cards:
       Program.write(card.code)
  -> Stack.create(stack_with_cards, source_session_id=session.id)
  -> LeafEvent::StackCreated(stack)
```

### Artifact Tracking

```
CardExecution.complete(result)
  -> snapshot_after = scan_project_files(project_path)
  -> new_files = snapshot_after - snapshot_before
  -> for each new_file:
       Artifact.register(path, created_by="card:{card_id}",
                         created_by_execution_id=stack_exec.id)

Agent.write_file(path, content)
  -> Filesystem.write(path, content)
  -> Artifact.register(path, created_by="agent")

Agent.create_folder(path)
  -> Filesystem.mkdir(path)
  -> Artifact.register(path, type=Directory, created_by="agent")

Watcher.file_created(path)
  -> Event.emit(type="file.created")
  -> Artifact.register(path, created_by="user")

Watcher.file_modified(path)
  -> Event.emit(type="file.modified")
  -> Artifact.update_modified(path)

Watcher.file_deleted(path)
  -> Event.emit(type="file.deleted")
  -> Artifact.mark_deleted(path)
```

---

## 9. Environment Variables for Card Programs

| Variable | Description |
|----------|-------------|
| `LEAF_PROJECT_ROOT` | Absolute path to project folder |
| `LEAF_CARD_ID` | This card's ID |
| `LEAF_STACK_ID` | Parent stack's ID |
| `LEAF_EXECUTION_ID` | StackExecution ID |
| `LEAF_EVENT_PAYLOAD` | JSON string with trigger event details (same format as today) |
| `LEAF_STEP_INDEX` | 0-based position in the pipeline |
| `LEAF_PREV_STDOUT` | Previous card's stdout (truncated to 10KB) |
| `LEAF_PREV_OUTPUT_PATH` | Path to file containing previous card's full stdout |

For single-card stacks, `LEAF_STEP_INDEX` is `0` and `LEAF_PREV_STDOUT` / `LEAF_PREV_OUTPUT_PATH` are empty strings.

The `LEAF_PREV_OUTPUT_PATH` mechanism: after each card runs, its stdout is written to `.leaf/outputs/{execution_id}/step_{position}.out`. The next card reads this file for full output.

---

## 10. LeafEvent Variants

New and updated variants for `crates/leaf-core/src/events.rs`:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", rename_all = "snake_case")]
pub enum LeafEvent {
    // --- Project events (unchanged) ---
    ProjectOpened(Project),
    ProjectClosed { project_id: Uuid },
    ProjectUpdated(Project),

    // --- Stack events (NEW — replaces old Card* events) ---
    StackCreated(Stack),
    StackUpdated(Stack),
    StackDeleted { stack_id: Uuid },
    StackEnabled { stack_id: Uuid },
    StackDisabled { stack_id: Uuid },

    // --- Card events (NEW — cards within stacks) ---
    CardCreated(Card),
    CardUpdated(Card),
    CardDeleted { card_id: Uuid, stack_id: Uuid },
    CardReordered { card_id: Uuid, stack_id: Uuid, old_position: i32, new_position: i32 },

    // --- Stack execution events (NEW — replaces old Execution* events) ---
    StackExecutionStarted {
        stack_id: Uuid,
        execution_id: Uuid,
        card_count: i32,
    },
    StackStepCompleted {
        execution_id: Uuid,
        position: i32,
        card_id: Uuid,
        duration_ms: u64,
    },
    StackExecutionCompleted {
        stack_id: Uuid,
        execution_id: Uuid,
        total_duration_ms: u64,
    },
    StackExecutionFailed {
        stack_id: Uuid,
        execution_id: Uuid,
        failed_position: i32,
        error: String,
    },

    // --- Artifact events (NEW) ---
    ArtifactCreated {
        artifact_id: Uuid,
        path: String,
        artifact_type: ArtifactType,
        created_by: String,
    },
    ArtifactModified {
        artifact_id: Uuid,
        path: String,
    },
    ArtifactDeleted {
        artifact_id: Uuid,
        path: String,
    },

    // --- File watcher events (unchanged) ---
    FileDetected { project_id: Uuid, path: String, event_type: String },
    WatcherStarted { project_id: Uuid, paths: Vec<String> },
    WatcherStopped { project_id: Uuid },
    WatcherError { project_id: Uuid, error: String },

    // --- Event processing (updated — matched_stacks instead of matched_cards) ---
    EventCreated(Event),
    EventProcessing { event_id: Uuid, matched_stacks: Vec<Uuid> },
    EventCompleted { event_id: Uuid, status: EventStatus },

    // --- Chat events (unchanged) ---
    SessionCreated(ChatSession),
    SessionUpdated(ChatSession),
    MessageReceived(Message),
    AgentThinking { session_id: Uuid },
    AgentToolCall { session_id: Uuid, tool_name: String, arguments: serde_json::Value },

    // --- MCP events (unchanged) ---
    McpServerConnected { server_id: Uuid, server_name: String, tool_count: usize },
    McpServerDisconnected { server_id: Uuid, server_name: String },
    McpServerError { server_id: Uuid, server_name: String, error: String },
    McpToolCalled { session_id: Uuid, server_name: String, tool_name: String },
    McpToolResult { session_id: Uuid, server_name: String, tool_name: String, success: bool },

    // --- System events (unchanged) ---
    Error { context: String, message: String },
    Warning { context: String, message: String },
}
```

---

## 11. Tauri Commands

LEAF-rs uses Tauri IPC, not REST endpoints. All commands live in `crates/leaf-app/src/commands/`.

### Stack Commands (`commands/stacks.rs` — NEW)

| Command | Signature | Description |
|---------|-----------|-------------|
| `list_stacks` | `(state) -> Vec<Stack>` | List all stacks in the current project |
| `get_stack` | `(state, stack_id) -> Option<StackWithCards>` | Get stack with its ordered cards |
| `create_stack` | `(app, state, input: CreateStackInput) -> Stack` | Create a stack with initial cards |
| `update_stack` | `(app, state, stack_id, input: UpdateStackInput) -> Stack` | Update stack metadata/trigger |
| `delete_stack` | `(app, state, stack_id) -> ()` | Delete stack and all cards (CASCADE) |
| `enable_stack` | `(app, state, stack_id) -> Stack` | Enable stack |
| `disable_stack` | `(app, state, stack_id) -> Stack` | Disable stack |
| `trigger_stack` | `(app, state, stack_id) -> StackExecution` | Manually trigger stack execution |

### Card Commands (`commands/cards.rs` — REWRITTEN)

| Command | Signature | Description |
|---------|-----------|-------------|
| `add_card` | `(app, state, stack_id, input: AddCardInput) -> Card` | Add a card to a stack |
| `update_card` | `(app, state, card_id, input: UpdateCardInput) -> Card` | Update card program or metadata |
| `delete_card` | `(app, state, card_id) -> ()` | Delete card from stack |
| `reorder_cards` | `(app, state, stack_id, card_ids: Vec<Uuid>) -> Vec<Card>` | Reorder cards by providing new order |

### Execution Commands (`commands/executions.rs` — REWRITTEN)

| Command | Signature | Description |
|---------|-----------|-------------|
| `list_stack_executions` | `(state, stack_id, limit) -> Vec<StackExecution>` | List executions for a stack |
| `get_stack_execution` | `(state, execution_id) -> Option<StackExecutionDetail>` | Get execution with per-card results |

### Artifact Commands (`commands/artifacts.rs` — NEW)

| Command | Signature | Description |
|---------|-----------|-------------|
| `list_artifacts` | `(state, filters) -> Vec<Artifact>` | List artifacts (filterable) |
| `get_artifact` | `(state, artifact_id) -> Option<Artifact>` | Get artifact detail |
| `scan_artifacts` | `(state) -> Vec<Artifact>` | Scan project and register existing files |

### Input Types

```rust
#[derive(Debug, Deserialize)]
pub struct CreateStackInput {
    pub name: String,
    pub description: Option<String>,
    pub trigger: TriggerConfig,
    pub cards: Vec<CreateCardInput>,
}

#[derive(Debug, Deserialize)]
pub struct CreateCardInput {
    pub name: String,
    pub description: Option<String>,
    pub code: String,  // TypeScript source code
}

#[derive(Debug, Deserialize)]
pub struct UpdateStackInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub trigger: Option<TriggerConfig>,
}

#[derive(Debug, Deserialize)]
pub struct AddCardInput {
    pub name: String,
    pub description: Option<String>,
    pub code: String,
    pub position: Option<i32>,  // Appended to end if not specified
}

#[derive(Debug, Deserialize)]
pub struct UpdateCardInput {
    pub name: Option<String>,
    pub description: Option<String>,
    pub code: Option<String>,
}

/// Stack with its cards included for detail views
#[derive(Debug, Serialize)]
pub struct StackWithCards {
    #[serde(flatten)]
    pub stack: Stack,
    pub cards: Vec<Card>,
}

/// Stack execution with per-card results
#[derive(Debug, Serialize)]
pub struct StackExecutionDetail {
    #[serde(flatten)]
    pub execution: StackExecution,
    pub card_executions: Vec<CardExecution>,
}
```

---

## 12. Agent Tools

### Updated Tool Summary

| Tool | Status | Description |
|------|--------|-------------|
| `propose_stack` | **New** (replaces `propose_card`) | Propose a new stack with cards |
| `create_stack_now` | **New** (replaces `create_card_now`) | Create stack immediately |
| `add_card` | **New** | Add a card to an existing stack |
| `read_file` | Unchanged | Read a project file |
| `write_file` | Modified | Write file + register artifact |
| `list_directory` | Unchanged | List directory contents |
| `create_folder` | **New** | Create a folder + register artifact |
| `delete_file` | **New** | Delete a file + mark artifact deleted |
| `move_file` | **New** | Move/rename a file + update artifact |
| `list_mcp_tools` | Unchanged | List MCP tools |
| `call_mcp_tool` | Unchanged | Call MCP tool |

### propose_stack

```rust
// In crates/leaf-agent/src/tools/stack.rs

/// Tool: propose_stack
/// Proposes a new stack with cards for user review.
fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "name": {
                "type": "string",
                "description": "Stack name (e.g., 'CSV Pipeline')"
            },
            "description": {
                "type": "string",
                "description": "What this workflow does"
            },
            "trigger": {
                "type": "object",
                "properties": {
                    "type": {
                        "type": "string",
                        "enum": ["file_created", "file_modified", "manual"]
                    },
                    "watch_path": { "type": "string" },
                    "patterns": {
                        "type": "array",
                        "items": { "type": "string" }
                    }
                },
                "required": ["type"]
            },
            "cards": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "description": { "type": "string" },
                        "code": { "type": "string" }
                    },
                    "required": ["name", "code"]
                },
                "minItems": 1
            }
        },
        "required": ["name", "trigger", "cards"]
    })
}
```

### create_stack_now

Same parameters as `propose_stack`. Creates the stack immediately:
1. Creates stack row in DB
2. For each card: generates slug, creates `.leaf/programs/{card_id}/main.ts`, inserts card row
3. Registers watcher path if trigger is file-based
4. Emits `LeafEvent::StackCreated`

### add_card

```rust
/// Tool: add_card
/// Adds a card to an existing stack.
fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "stack_id": {
                "type": "string",
                "description": "The stack to add this card to"
            },
            "name": {
                "type": "string",
                "description": "Card step name"
            },
            "description": {
                "type": "string",
                "description": "What this step does"
            },
            "code": {
                "type": "string",
                "description": "TypeScript code for this step"
            },
            "position": {
                "type": "integer",
                "description": "Position in the pipeline (appended to end if omitted)"
            }
        },
        "required": ["stack_id", "name", "code"]
    })
}
```

### create_folder

```rust
/// Tool: create_folder
/// Creates a folder in the project and registers it as an artifact.
fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "Folder path relative to project root (e.g., 'inbox', 'output/reports')"
            }
        },
        "required": ["path"]
    })
}
```

### write_file (modified)

Same parameters as today, but after writing the file it also calls `Artifact.register(path, created_by="agent")`.

### delete_file

```rust
/// Tool: delete_file
/// Deletes a file from the project and marks its artifact as deleted.
fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "path": {
                "type": "string",
                "description": "File path relative to project root"
            }
        },
        "required": ["path"]
    })
}
```

Validates path is within project. Prevents deleting `.leaf/` contents.

### move_file

```rust
/// Tool: move_file
/// Moves or renames a file within the project. Updates artifact tracking.
fn parameters_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "source": {
                "type": "string",
                "description": "Current path relative to project root"
            },
            "destination": {
                "type": "string",
                "description": "New path relative to project root"
            }
        },
        "required": ["source", "destination"]
    })
}
```

---

## 13. Guided Workflow Setup

### Agent Prompt Updates

The system prompt in `crates/leaf-agent/src/prompts.rs` needs additions:

```rust
pub const WORKFLOW_SETUP_GUIDANCE: &str = r#"
## Multi-Step Workflow Creation

When a user describes a multi-step workflow:

1. **PLAN first** - Propose the folder structure, stack name, trigger, and card steps
   as a numbered list. Ask clarifying questions if the requirements are unclear.

2. **BUILD step-by-step**:
   a. Create folders first (using create_folder tool)
   b. Propose the full stack with all cards (using propose_stack)

3. **CONFIRM** - Wait for user approval before creating.
   If the user says "just do it" or similar, use create_stack_now directly.

4. **VERIFY** - Confirm everything is wired up and show how to test.

When creating cards within a stack:
- Each card should be focused on ONE task
- Use LEAF_EVENT_PAYLOAD to access the trigger event details
- Use LEAF_PREV_OUTPUT_PATH to read the previous card's output
- Write output to stdout for the next card to consume via LEAF_PREV_STDOUT
- For the final card, write result files to the output folder

For simple automations (single step), create a stack with one card.
The user never needs to think about the Stack/Card distinction for simple cases.
"#;
```

### Example Conversation

```
User: I want to process CSV files. Validate them, transform
      the data, then save results.

Agent: I'll set up a "CSV Pipeline" stack that watches inbox/
       for CSV files, with 3 cards:

       1. "Validate" - check headers and types
       2. "Transform" - normalize columns
       3. "Save" - write to output/

       First, let me create the folders.
       [creates inbox/ and output/ folders]

       Here's the stack with all 3 cards:
       [shows code for each card]

       Create this stack?

User: Yes, create it.

Agent: Done! Your CSV Pipeline is ready.

       Stack: "CSV Pipeline" (watches inbox/*.csv)
         Card 1: "Validate"  -- done
         Card 2: "Transform" -- done
         Card 3: "Save"      -- done

       Drop a CSV into inbox/ to test it.
```

---

## 14. Sandbox Changes

### Current Behavior

```rust
// crates/leaf-executor/src/deno.rs
pub fn build_args(&self, project_root: &Path, program_path: &Path) -> Vec<String> {
    vec![
        "run".to_string(),
        format!("--allow-read={}", project_root.display()),
        format!("--allow-write={}", project_root.join(".leaf").join("outputs").display()),
        "--allow-env=LEAF_EVENT_PAYLOAD,LEAF_PROJECT_ROOT,LEAF_CARD_ID,LEAF_EXECUTION_ID".to_string(),
        "--no-prompt".to_string(),
        program_path.to_string_lossy().to_string(),
    ]
}
```

### New Behavior

```rust
pub fn build_args(&self, project_root: &Path, program_path: &Path) -> Vec<String> {
    vec![
        "run".to_string(),
        // Read access: entire project directory
        format!("--allow-read={}", project_root.display()),
        // Write access: entire project directory (was .leaf/outputs only)
        format!("--allow-write={}", project_root.display()),
        // Env access: expanded for stack execution context
        "--allow-env=LEAF_EVENT_PAYLOAD,LEAF_PROJECT_ROOT,LEAF_CARD_ID,LEAF_STACK_ID,LEAF_EXECUTION_ID,LEAF_STEP_INDEX,LEAF_PREV_STDOUT,LEAF_PREV_OUTPUT_PATH".to_string(),
        "--no-prompt".to_string(),
        program_path.to_string_lossy().to_string(),
    ]
}
```

**Changes:**
- `--allow-write` expanded from `.leaf/outputs/` to entire project directory
- `--allow-env` expanded to include new stack-related variables
- Network access remains disabled
- Programs still cannot access anything outside the project directory

---

## 15. Frontend Changes

### Stack List View

The card list (`ui/src/components/CardList.tsx`) becomes a stack list. Single-card stacks display as simple items (no visible hierarchy). Multi-card stacks show the pipeline:

```
STACKS
------
+----------------------------------------------+
| CSV Pipeline                        * Active |
| Watches: inbox/*.csv                         |
| |-- Validate CSV          (Step 1)          |
| |-- Transform Data        (Step 2)          |
| +-- Save Results          (Step 3)          |
|                                              |
| Last run: 2 min ago                          |
+----------------------------------------------+

+----------------------------------------------+
| Image Resizer                       * Active |
| Watches: photos/*.jpg                        |
| Last run: 1 hour ago                         |
+----------------------------------------------+
```

### Stack Detail View

```
+----------------------------------------------------------------+
| CSV Pipeline                                   [Edit] [Delete] |
+----------------------------------------------------------------+
|                                                                |
|  TRIGGER                                                       |
|  Type: file_created  Folder: inbox/  Pattern: *.csv            |
|  Status: * Watching                                            |
|                                                                |
|  CARDS                                         [+ Add Card]   |
|  +----------------------------------------------------------+ |
|  | 1. Validate CSV                         [up][down][x]     | |
|  |    Checks headers and data types                          | |
|  |    Last run: ok 0.3s                    [View Code]       | |
|  +----------------------------------------------------------+ |
|  | 2. Transform Data                       [up][down][x]     | |
|  |    Filters rows, adds category column                     | |
|  |    Last run: ok 1.2s                    [View Code]       | |
|  +----------------------------------------------------------+ |
|  | 3. Save Results                         [up][down][x]     | |
|  |    Writes processed CSV to output/                        | |
|  |    Last run: ok 0.1s                    [View Code]       | |
|  +----------------------------------------------------------+ |
|                                                                |
|  RECENT RUNS                                                   |
|  | 14:32  data.csv     ok 1.6s  [1 ok] [2 ok] [3 ok]       | |
|  | 14:15  sales.csv    FAIL 0.5s [1 ok] [2 FAIL]            | |
|  | 13:50  orders.csv   ok 2.1s  [1 ok] [2 ok] [3 ok]       | |
|                                                                |
|  [Run Manually]  [View All Runs]                               |
+----------------------------------------------------------------+
```

### Artifacts Panel

A new sidebar panel listing tracked artifacts:

```
ARTIFACTS                                      [Filter v]
----------------------------------------------------------

  [dir] inbox/                         (user)
  [dir] output/                        (agent)

  [file] inbox/data.csv          12 KB (user, 2 min ago)
  [file] inbox/sales.csv          8 KB (user, 15 min ago)
  [file] output/data_processed.csv 14 KB (card: CSV Pipeline, 2 min ago)

  -- Deleted --
  [file] inbox/old_file.csv            (deleted 1 hour ago)
```

Filter options: All / Files / Folders, By creator: All / Agent / Cards / User, Active / Deleted

### Event Queue Updates

Stack executions appear as grouped entries:

```
EVENT QUEUE
-------------------------------------------

  14:32:15  file.created  inbox/data.csv
            -> CSV Pipeline (stack)
            |- Card 1: Validate CSV    ok 0.3s
            |- Card 2: Transform Data  ok 1.2s
            +- Card 3: Save Results    ok 0.1s
            ok Completed (1.6s total)

  14:15:22  file.created  inbox/sales.csv
            -> CSV Pipeline (stack)
            |- Card 1: Validate CSV    ok 0.2s
            +- Card 2: Transform Data  FAIL "Column not found"
            FAIL Failed at card 2
```

### Frontend Files

| File | Status | Description |
|------|--------|-------------|
| `StackList.tsx` | **New** (replaces `CardList.tsx`) | Stack list with hierarchy |
| `StackItem.tsx` | **New** (replaces `CardItem.tsx`) | Individual stack display |
| `StackDetail.tsx` | **New** (replaces `CardEditor.tsx`) | Stack detail with card pipeline |
| `CardStep.tsx` | **New** | Individual card step with reorder handles |
| `StackExecutionView.tsx` | **New** | Grouped execution display with per-card results |
| `ArtifactPanel.tsx` | **New** | Sidebar panel listing artifacts |
| `ArtifactItem.tsx` | **New** | Individual artifact display |
| `useStacks.ts` | **New** (replaces card hooks) | Tauri command integration for stacks |
| `useArtifacts.ts` | **New** | Tauri command integration for artifacts |
| `EventQueue.tsx` | Modified | Group stack execution events |
| `ChatView.tsx` | Modified | Handle stack proposals in chat |

---

## 16. Implementation Phases

### Phase A: Core Types & Database Schema

**Goal:** Define new Rust types and create database migration.

**Files modified/created:**
- `crates/leaf-core/src/types.rs` — Add `Stack`, `StackExecution`, `CardExecution`, `Artifact`, `ArtifactType`, `ArtifactStatus`. Redefine `Card` (remove `trigger`, `project_id`, `session_id`; add `stack_id`, `program_path`, `position`). Remove old `Execution`.
- `crates/leaf-core/src/events.rs` — Update `LeafEvent` enum with new variants (Stack*, Card*, Artifact*, StackExecution*). Remove old Card/Execution variants.
- `crates/leaf-db/src/migrations.rs` — Add `migration_v2()` with clean break: drop old tables, create new schema. Bump `SCHEMA_VERSION` to 2.

**Tests:** Unit tests for new type constructors. Migration test with in-memory DB. Verify schema creation.

**Dependencies:** None. This is the foundation.

---

### Phase B: Stack & Card CRUD + Tauri Commands

**Goal:** Implement database CRUD for stacks/cards and expose via Tauri commands.

**Files modified/created:**
- `crates/leaf-db/src/` — Add `stacks.rs` (Stack CRUD), update `cards.rs` (Card CRUD with stack_id, position, program_path). Add `stack_executions.rs`, `card_executions.rs` (execution CRUD). Remove old `executions.rs` if it exists.
- `crates/leaf-app/src/commands/stacks.rs` — **New.** All stack Tauri commands: `list_stacks`, `get_stack`, `create_stack`, `update_stack`, `delete_stack`, `enable_stack`, `disable_stack`, `trigger_stack`.
- `crates/leaf-app/src/commands/cards.rs` — **Rewrite.** Card commands: `add_card`, `update_card`, `delete_card`, `reorder_cards`.
- `crates/leaf-app/src/commands/executions.rs` — **Rewrite.** Execution commands: `list_stack_executions`, `get_stack_execution`.
- `crates/leaf-app/src/commands/mod.rs` — Register new command modules.
- `crates/leaf-app/src/lib.rs` — Register new Tauri commands in the builder.
- `crates/leaf-app/src/events.rs` — Update trigger matching to iterate over stacks instead of cards.

**Tests:** DB CRUD tests for stacks, cards, ordering. Integration tests for Tauri commands. Trigger evaluation against stacks.

**Dependencies:** Phase A.

---

### Phase C: Stack Execution Engine

**Goal:** Implement sequential card execution within a stack, including sandbox relaxation.

**Files modified/created:**
- `crates/leaf-executor/src/lib.rs` — Add `execute_stack()` that runs cards sequentially, passing stdout between steps. Write step outputs to `.leaf/outputs/{execution_id}/step_{position}.out`.
- `crates/leaf-executor/src/deno.rs` — Relax `--allow-write` to entire project directory. Expand `--allow-env` with new variables (`LEAF_STACK_ID`, `LEAF_STEP_INDEX`, `LEAF_PREV_STDOUT`, `LEAF_PREV_OUTPUT_PATH`).
- `crates/leaf-app/src/commands/stacks.rs` — Wire up `trigger_stack` to the execution engine. Handle retry logic at the stack level.
- `crates/leaf-app/src/events.rs` — Update event processing to create StackExecutions instead of old Executions.

**Tests:** Stack execution: all succeed, failure at step N, disabled cards skipped, env var passing, stdout chaining between cards. Single-card stack execution matches expected behavior.

**Dependencies:** Phase B.

---

### Phase D: Artifact System

**Goal:** Implement artifact tracking — DB, registration, watcher integration, API.

**Files modified/created:**
- `crates/leaf-db/src/artifacts.rs` — **New.** Artifact CRUD: `register_artifact`, `update_artifact_modified`, `mark_artifact_deleted`, `list_artifacts`, `get_artifact`, `scan_project_files`.
- `crates/leaf-app/src/commands/artifacts.rs` — **New.** Tauri commands: `list_artifacts`, `get_artifact`, `scan_artifacts`.
- `crates/leaf-app/src/commands/mod.rs` — Register artifact commands.
- `crates/leaf-app/src/events.rs` — Hook artifact registration into file event processing.
- `crates/leaf-executor/src/lib.rs` — Add project file snapshot before/after execution. Register new/modified files as artifacts created by the card.

**Tests:** Artifact CRUD. Detection from watcher events. Detection from card execution (snapshot diff). MIME type detection. Status lifecycle (Active -> Modified -> Deleted).

**Dependencies:** Phase C (needs execution engine for snapshot diff).

---

### Phase E: Agent Tools & Prompts

**Goal:** Update the agent with stack-aware tools and guided workflow prompts.

**Files modified/created:**
- `crates/leaf-agent/src/tools/stack.rs` — **New.** `ProposeStackTool`, `CreateStackNowTool`, `AddCardTool`.
- `crates/leaf-agent/src/tools/filesystem.rs` — Add `CreateFolderTool`, `DeleteFileTool`, `MoveFileTool`. Modify `WriteFileTool` to register artifacts.
- `crates/leaf-agent/src/tools/mod.rs` — Update `ToolRegistry::with_defaults()` to register new tools, remove old card tools.
- `crates/leaf-agent/src/tools/card.rs` — **Remove** (replaced by stack.rs).
- `crates/leaf-agent/src/prompts.rs` — Rewrite system prompt for stack/card model. Add workflow setup guidance. Update environment variable documentation.

**Tests:** Tool function unit tests. Agent integration tests with mock LLM. Verify stack creation flow.

**Dependencies:** Phase D (needs artifact registration in write_file).

---

### Phase F: Frontend — Stack & Card UI

**Goal:** Replace card-centric UI with stack-centric UI.

**Files created/modified (in `ui/src/`):**
- `components/StackList.tsx` — **New.** Replaces `CardList.tsx`. Shows stacks with hierarchy.
- `components/StackItem.tsx` — **New.** Replaces `CardItem.tsx`. Individual stack display.
- `components/StackDetail.tsx` — **New.** Replaces `CardEditor.tsx`. Stack detail with card pipeline, reordering, code view.
- `components/CardStep.tsx` — **New.** Individual card step display with reorder handles.
- `components/StackExecutionView.tsx` — **New.** Grouped execution display with per-card results.
- `hooks/useStacks.ts` — **New.** TanStack Query hooks for stack Tauri commands.
- `hooks/useExecutions.ts` — **Rewrite.** Hooks for stack/card execution commands.
- Remove old: `CardList.tsx`, `CardItem.tsx`, `CardEditor.tsx`.
- Update `App.tsx` / layout to use new components.

**Dependencies:** Phase B (needs Tauri commands working).

---

### Phase G: Frontend — Artifact Panel & Execution Views

**Goal:** Add artifact tracking UI and updated execution display.

**Files created/modified (in `ui/src/`):**
- `components/ArtifactPanel.tsx` — **New.** Sidebar panel listing artifacts with filters.
- `components/ArtifactItem.tsx` — **New.** Individual artifact display.
- `hooks/useArtifacts.ts` — **New.** TanStack Query hooks for artifact commands.
- `components/EventQueue.tsx` — **Modified.** Group stack execution events, show per-card progress.
- `components/EventItem.tsx` — **Modified.** Display stack execution events with card steps.
- Update layout to include artifacts panel (sidebar tab or panel).

**Dependencies:** Phase D (needs artifact API), Phase F (needs stack UI).

---

### Phase H: Frontend — Guided Setup UX

**Goal:** Polish the chat experience for multi-step workflow creation.

**Files created/modified (in `ui/src/`):**
- `components/ChatView.tsx` — **Modified.** Handle stack proposals in chat messages. Show structured preview of proposed stacks with cards.
- `components/StackProposalCard.tsx` — **New.** Renders a stack proposal within the chat, with approve/reject buttons and code preview for each card.
- `components/WorkflowProgress.tsx` — **New.** Visual progress tracker during multi-step stack creation (folder created, stack created, cards 1/3 done, etc.).

**Dependencies:** Phase E (needs agent tools), Phase F (needs stack UI components).

---

## 17. Open Questions

1. **Error handling policy** — On card failure, the pipeline stops (fail-fast). Should users be able to configure skip-on-failure or retry-step? **Recommendation:** Start with fail-fast. Add configurability later if needed.

2. **Artifact initial scan** — When a project first gets the artifact system, should LEAF scan all existing files? **Recommendation:** No automatic scan. Provide `scan_artifacts` command for on-demand scanning. Only track files going forward by default.

3. **Single-card UI simplification** — Should the UI hide the Stack/Card distinction for stacks with one card? **Recommendation:** Yes. Show it as "CSV Analyzer" not "CSV Analyzer -> Step 1: CSV Analyzer". Reveal the pipeline structure only when there are multiple cards.

4. **Stack-to-Stack chaining** — A stack's final card can write to a folder watched by another stack's trigger. This gives arbitrary pipeline depth through events, without nesting. The system already supports this — no new code needed.

5. **Card reuse** — A Card belongs to exactly one Stack (FK constraint). If users want the same logic in multiple stacks, the agent duplicates the program. Keeps the model simple.

6. **Program file organization** — With explicit `program_path`, where do files go? **Recommendation:** `.leaf/programs/{card_id}/main.ts` — same convention as today but stored explicitly. The card ID makes each program directory unique regardless of stack.

---

## 18. Affected Crates Summary

| Crate | Impact | Description |
|-------|--------|-------------|
| `leaf-core` | **Heavy** | New types (Stack, StackExecution, CardExecution, Artifact), redefined Card, new LeafEvent variants |
| `leaf-db` | **Heavy** | New migration, new CRUD modules (stacks, cards, stack_executions, card_executions, artifacts) |
| `leaf-app` | **Heavy** | New Tauri commands, rewritten event processing, updated trigger matching |
| `leaf-agent` | **Heavy** | New tools (propose_stack, create_stack_now, add_card, create_folder, delete_file, move_file), rewritten prompts |
| `leaf-executor` | **Medium** | Stack execution flow, relaxed sandbox, new env vars |
| `leaf-watcher` | **None** | Still emits file events the same way |
| `leaf-mcp` | **None** | No changes |
| `ui` | **Heavy** | Full rewrite of card UI to stack UI, new artifact panel, updated event display |
