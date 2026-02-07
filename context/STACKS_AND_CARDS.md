# Stacks & Cards — Multi-Step Workflow Design

## Overview

This document redefines LEAF's workflow model using an explicit **Stack & Cards** metaphor inspired by Apple HyperCard. It replaces the `parent_id`/subcard approach from the Hybrid Stacks spec with two first-class concepts: **Stack** (a trigger + ordered pipeline) and **Card** (a single program step).

This also covers Artifacts and Guided Workflow Setup, carried forward from the original spec.

### Why the change?

The Hybrid Stacks spec overloaded `Card` to mean both "a single automation" and "a container for child automations." A parent card had a trigger and a `program_path`, but if it had subcards the program was silently ignored. This created:

- Ambiguous identity — is a card a step or a pipeline?
- Special-case execution — "if has children, ignore own program"
- A nesting question that had to be answered with a policy check

Making Stack a first-class concept eliminates all three problems.

### Design Principles

- **HyperCard honest** — A Stack contains Cards. Cards don't contain Cards.
- **One execution path** — The runner always executes Stack → Cards in order. No branching.
- **Nesting is structurally impossible** — No FK exists for Stack-in-Stack. The schema enforces the depth limit.
- **Backward compatible** — Today's standalone card becomes a Stack with one Card. No migration complexity.
- **File-first** — Artifacts are real files on disk. The DB is an index, not the source of truth.
- **Guided, not magic** — The agent proposes each step; the user confirms.

---

## Concept Map

```
┌────────────────────────────────────────────────────────────────────────────┐
│                         LEAF CONCEPT MAP (updated)                        │
├────────────────────────────────────────────────────────────────────────────┤
│                                                                            │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐       ┌─────────┐   │
│  │ Project │────────▶│ Watcher │────────▶│  Event  │──────▶│  Queue  │   │
│  └─────────┘  opens   └─────────┘ emits   └─────────┘pushes └─────────┘   │
│       │                                        │                   │       │
│       │ has many                               │ matches           │       │
│       ▼                                        ▼                   ▼       │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐       ┌─────────┐   │
│  │  Stack  │◀────────│ Trigger │◀────────│  Event  │       │   UI    │   │
│  └─────────┘  has     └─────────┘evaluates└─────────┘       └─────────┘   │
│       │                                                                    │
│       │ contains (ordered)                                                 │
│       ▼                                                                    │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐                      │
│  │  Card   │────────▶│ Program │────────▶│ Sandbox │                      │
│  └─────────┘  has     └─────────┘  runs   └─────────┘                      │
│       │                                                                    │
│       │ produces                                                           │
│       ▼                                                                    │
│  ┌─────────┐                                                               │
│  │Artifact │                                                               │
│  └─────────┘                                                               │
│                                                                            │
│  ┌─────────┐         ┌─────────┐         ┌─────────┐                      │
│  │   MCP   │◀────────│  Agent  │◀────────│  Chat   │                      │
│  └─────────┘  tools   └─────────┘responds │ Session │                      │
│                            │               └─────────┘                      │
│                            │ creates                                       │
│                            ▼                                               │
│                       ┌─────────┐                                          │
│                       │  Stack  │ (with Cards)                             │
│                       └─────────┘                                          │
│                                                                            │
│  Stack.source_session_id ─────────────────▶ ChatSession                   │
│  (provenance: which conversation created this stack)                      │
│                                                                            │
└────────────────────────────────────────────────────────────────────────────┘
```

---

## New Concepts

### Stack

**Purpose**: A trigger + an ordered pipeline of Cards. The unit users create, name, enable, and disable.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| project_id | Which project this belongs to |
| name | Human-readable name (e.g., "CSV Pipeline") |
| description | What this workflow does |
| trigger | TriggerConfig (FileCreated, FileModified, Schedule, Manual) |
| enabled | Whether this stack is active |
| source_session_id | ChatSession that created this stack (provenance) |
| created_at | When created |
| updated_at | Last modified |

| Action | Description |
|--------|-------------|
| create(project_id, name, trigger) | Create a new stack |
| update(changes) | Modify stack metadata or trigger |
| enable() | Activate the stack (trigger starts matching) |
| disable() | Deactivate the stack (trigger stops matching) |
| delete() | Remove stack and all its cards (CASCADE) |

**Key point:** A Stack always has at least one Card. There is no "empty stack." When the agent creates a stack, it creates the first card simultaneously.

---

### Card (redefined)

**Purpose**: A single program step within a Stack. Has no trigger of its own — it runs when its Stack fires.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| stack_id | Which stack this belongs to (NOT NULL) |
| name | Human-readable step name (e.g., "Validate Input") |
| description | What this step does |
| program | ProgramConfig (language, entrypoint, timeout, retries) |
| position | 0-indexed order within the stack |
| enabled | Whether this card runs (disabled cards are skipped) |
| created_at | When created |
| updated_at | Last modified |

| Action | Description |
|--------|-------------|
| create(stack_id, name, program, position) | Add a card to a stack |
| update(changes) | Modify card program or metadata |
| reorder(new_position) | Move within the stack |
| enable() | Include in execution |
| disable() | Skip during execution |
| delete() | Remove card from stack |

**What changed from the old Card:**
- `trigger` moved to Stack — Cards don't have triggers
- `stack_id` replaces the old implicit standalone identity — every Card belongs to a Stack
- `session_id` (provenance) moved to Stack — the conversation creates the Stack, not individual cards
- `position` is always meaningful, not just for subcards

---

### StackExecution

**Purpose**: A single run of a Stack's card pipeline.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| stack_id | Which stack ran |
| event_id | What triggered it |
| status | pending, running, completed, failed |
| card_count | Total enabled cards at start |
| completed_cards | How many finished successfully |
| failed_at_position | Which position failed (null if success) |
| error | Error message from the failing card |
| started_at | When execution began |
| completed_at | When execution finished |

| Action | Description |
|--------|-------------|
| start(stack, event) | Begin pipeline execution |
| step_completed(position) | Mark a card step as done |
| fail(position, error) | Stop pipeline on failure |
| complete() | All cards finished successfully |

---

### CardExecution

**Purpose**: A single run of one Card within a StackExecution.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| stack_execution_id | Parent pipeline run |
| card_id | Which card ran |
| position | Position at time of execution |
| status | pending, running, completed, failed |
| stdout | Program stdout |
| stderr | Program stderr |
| exit_code | Process exit code |
| started_at | When started |
| completed_at | When finished |

---

### Artifact

**Purpose**: Lightweight tracking of files created by stacks, cards, the agent, or the user.

| State | Description |
|-------|-------------|
| id | Unique identifier |
| project_id | Which project |
| relative_path | Path relative to project root |
| created_by | Origin: "agent", "card:{card_id}", "user" |
| created_by_execution_id | StackExecution that created it (nullable) |
| file_size | Size in bytes |
| mime_type | Detected MIME type |
| created_at | When first tracked |

| Action | Description |
|--------|-------------|
| register(path, created_by) | Track a new file |
| unregister(path) | Stop tracking (file deleted) |
| scan(project_path) | Detect and register existing files |

---

## Schema

```sql
-- Stacks: the trigger + pipeline container
CREATE TABLE stacks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    trigger_type TEXT NOT NULL,         -- 'file_created', 'file_modified', 'schedule', 'manual'
    trigger_config TEXT NOT NULL,       -- JSON (TriggerConfig)
    enabled INTEGER NOT NULL DEFAULT 1,
    source_session_id TEXT,            -- ChatSession provenance
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE INDEX idx_stacks_project ON stacks(project_id);

-- Cards: individual program steps within a stack
CREATE TABLE cards (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    program_path TEXT NOT NULL,         -- Relative to .leaf/programs/
    program_config TEXT NOT NULL,       -- JSON (ProgramConfig)
    position INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

CREATE INDEX idx_cards_stack ON cards(stack_id);

-- Stack executions: one run of a full pipeline
CREATE TABLE stack_executions (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id),
    event_id TEXT,
    status TEXT NOT NULL,               -- pending, running, completed, failed
    card_count INTEGER NOT NULL,
    completed_cards INTEGER NOT NULL DEFAULT 0,
    failed_at_position INTEGER,
    error TEXT,
    started_at DATETIME,
    completed_at DATETIME
);

CREATE INDEX idx_stack_exec_stack ON stack_executions(stack_id);

-- Card executions: one card's run within a stack execution
CREATE TABLE card_executions (
    id TEXT PRIMARY KEY,
    stack_execution_id TEXT NOT NULL REFERENCES stack_executions(id) ON DELETE CASCADE,
    card_id TEXT NOT NULL REFERENCES cards(id),
    position INTEGER NOT NULL,
    status TEXT NOT NULL,               -- pending, running, completed, failed
    stdout TEXT,
    stderr TEXT,
    exit_code INTEGER,
    started_at DATETIME,
    completed_at DATETIME
);

CREATE INDEX idx_card_exec_stack_exec ON card_executions(stack_execution_id);

-- Artifacts: file tracking
CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    relative_path TEXT NOT NULL,
    created_by TEXT NOT NULL,           -- 'agent', 'card:<card_id>', 'user'
    created_by_execution_id TEXT,       -- FK to stack_executions
    file_size INTEGER,
    mime_type TEXT,
    created_at DATETIME NOT NULL,
    UNIQUE(project_id, relative_path)
);

CREATE INDEX idx_artifacts_project ON artifacts(project_id);
```

**Note on CASCADE:** Deleting a stack deletes its cards and their executions. This matches HyperCard — deleting a stack deletes the cards in it.

---

## Synchronizations

### Trigger Matching (updated)

The existing trigger evaluation logic stays the same, but now it lives on Stack instead of Card:

```
Event.emit(type="file.*")
  -> for each Stack.enabled where Stack.trigger.evaluate(event) == true:
       StackExecution.start(stack, event)
```

**Key change:** We iterate over Stacks, not Cards. The self-evaluating trigger pattern (`trigger.evaluate(event)`) is preserved — it just lives on Stack.trigger instead of Card.trigger.

---

### Stack Execution (new)

```
StackExecution.start(stack, event)
  -> cards = Card.list(stack_id, enabled=true, order_by=position)
  -> stack_exec = StackExecution.create(stack, event, card_count=len(cards))
  -> Event.emit(type="stack.started", payload={stack_id, stack_execution_id, card_count})
  -> for each card in cards:
       CardExecution.start(card, stack_exec, event)

CardExecution.start(card, stack_exec, event)
  -> Sandbox.setup(card)
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
  -> StackExecution.step_completed(position)
  -> Event.emit(type="stack.step_completed", payload={stack_execution_id, position, card_id})
  -> continue to next card

CardExecution.fail(error)
  -> StackExecution.fail(position, error)
  -> Event.emit(type="stack.failed", payload={stack_id, stack_execution_id, failed_position, error})
  -> STOP (do not execute remaining cards)

StackExecution.all_steps_complete()
  -> StackExecution.status = completed
  -> Event.emit(type="stack.completed", payload={stack_id, stack_execution_id, duration})
```

**Single-card stacks** follow the exact same path. There's no special case — it's just a pipeline of length 1.

---

### Stack Management

```
Stack.create(project_id, name, trigger, cards)
  -> Trigger.register(stack.trigger)
  -> if Trigger.type == "file.*":
       Watcher.add_path(trigger.folder, trigger.pattern)
  -> Event.emit(type="stack.created", payload={stack_id})

Stack.enable()
  -> Trigger.arm(stack.trigger)
  -> Event.emit(type="stack.enabled", payload={stack_id})

Stack.disable()
  -> Trigger.disarm(stack.trigger)
  -> Event.emit(type="stack.disabled", payload={stack_id})

Stack.delete()
  -> CASCADE deletes all Cards
  -> Trigger.unregister(stack.trigger)
  -> Program.delete_all(stack.cards.program_paths)
  -> Event.emit(type="stack.deleted", payload={stack_id})
```

---

### Card Management (within a Stack)

```
Card.create(stack_id, name, program, position)
  -> Event.emit(type="card.created", payload={card_id, stack_id, position})

Card.reorder(new_position)
  -> shift other cards' positions
  -> Event.emit(type="card.reordered", payload={card_id, stack_id, old_position, new_position})

Card.delete()
  -> shift remaining cards' positions to close gap
  -> Program.delete(card.program_path)
  -> Event.emit(type="card.deleted", payload={card_id, stack_id})
```

---

### Agent → Stack/Card Creation

```
ChatSession.send(message)
  -> Agent.respond(session, message, context={project, stacks, recent_events})

Agent.propose_stack(spec)
  -> Message.create(session_id, role="assistant", content=stack_preview)
  -> await User.confirm or User.reject

User.confirm(stack_spec)
  -> Agent.generate_code(stack_spec.cards)
  -> for each card in stack_spec.cards:
       Program.generate(card.code)
  -> Stack.create(stack_with_cards, source_session_id=session.id)
  -> Event.emit(type="stack.created", payload={stack_id, session_id})
```

---

### Artifact Tracking

```
CardExecution.complete(result)
  -> snapshot_after = scan_project_files(project_path)
  -> new_files = snapshot_after - snapshot_before
  -> for each new_file:
       Artifact.register(path=new_file, created_by="card:{card_id}",
                         created_by_execution_id=stack_exec.id)

Agent.write_file(path, content)
  -> Artifact.register(path=path, created_by="agent")
```

---

## Environment Variables for Card Programs

| Variable | Description |
|----------|-------------|
| `LEAF_PROJECT_ROOT` | Absolute path to project folder |
| `LEAF_CARD_ID` | This card's ID |
| `LEAF_STACK_ID` | Parent stack's ID |
| `LEAF_EXECUTION_ID` | StackExecution ID |
| `LEAF_EVENT_PAYLOAD` | JSON string with trigger event details |
| `LEAF_STEP_INDEX` | 0-based position in the pipeline |
| `LEAF_PREV_STDOUT` | Previous card's stdout (truncated to 10KB) |
| `LEAF_PREV_OUTPUT_PATH` | Path to file containing previous card's full stdout |

For single-card stacks, `LEAF_STEP_INDEX` is `0` and `LEAF_PREV_STDOUT` / `LEAF_PREV_OUTPUT_PATH` are empty/unset.

The `LEAF_PREV_OUTPUT_PATH` mechanism: after each card runs, its stdout is written to `.leaf/outputs/{execution_id}/step_{position}.out`. The next card reads this file for full output.

---

## Event Types

| Type | Payload | Description |
|------|---------|-------------|
| `stack.created` | `{stack_id}` | New stack created |
| `stack.enabled` | `{stack_id}` | Stack activated |
| `stack.disabled` | `{stack_id}` | Stack deactivated |
| `stack.deleted` | `{stack_id}` | Stack removed |
| `stack.started` | `{stack_id, stack_execution_id, card_count}` | Pipeline began |
| `stack.step_completed` | `{stack_execution_id, position, card_id, duration}` | One card finished |
| `stack.completed` | `{stack_id, stack_execution_id, total_duration}` | All cards finished |
| `stack.failed` | `{stack_id, stack_execution_id, failed_position, error}` | Pipeline stopped on failure |
| `card.created` | `{card_id, stack_id, position}` | Card added to stack |
| `card.reordered` | `{card_id, stack_id, old_position, new_position}` | Card moved |
| `card.deleted` | `{card_id, stack_id}` | Card removed |

---

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| `GET` | `/stacks` | List all stacks in project |
| `POST` | `/stacks` | Create a stack (with initial cards) |
| `GET` | `/stacks/{id}` | Get stack with its cards |
| `PUT` | `/stacks/{id}` | Update stack metadata/trigger |
| `DELETE` | `/stacks/{id}` | Delete stack and all cards |
| `POST` | `/stacks/{id}/enable` | Enable stack |
| `POST` | `/stacks/{id}/disable` | Disable stack |
| `POST` | `/stacks/{id}/trigger` | Manually trigger stack |
| `POST` | `/stacks/{id}/cards` | Add a card to a stack |
| `PUT` | `/stacks/{id}/cards/reorder` | Reorder cards |
| `GET` | `/stacks/{id}/executions` | List stack executions |
| `GET` | `/stack-executions/{id}` | Get execution with per-card results |
| `PUT` | `/cards/{id}` | Update a card |
| `DELETE` | `/cards/{id}` | Delete a card |

---

## Agent Tools (updated)

| Tool | Description |
|------|-------------|
| `propose_stack` | Propose a new stack with cards |
| `create_stack_now` | Create stack immediately (for "just do it" mode) |
| `add_card` | Add a card to an existing stack |
| `read_file` | Read a project file |
| `write_file` | Write file + register artifact |
| `list_directory` | List directory contents |

The `propose_stack` tool replaces both the old `propose_card` and `propose_subcard` tools. It takes:

```
propose_stack({
  name: "CSV Pipeline",
  description: "Process incoming CSV files",
  trigger: { type: "file_created", watch_path: "inbox", patterns: ["*.csv"] },
  cards: [
    { name: "Validate", description: "Check headers and types", code: "..." },
    { name: "Transform", description: "Normalize columns", code: "..." },
    { name: "Save", description: "Write to output/", code: "..." }
  ]
})
```

For simple automations, the agent creates a stack with one card — the user never needs to think about it.

---

## Guided Workflow Setup

When a user describes a multi-step workflow, the agent follows this flow:

1. **Plan** — Propose the folder structure, stack name, trigger, and card steps as a numbered list
2. **Build step-by-step** — Create folders, then propose the stack with all cards
3. **Confirm** — Wait for user approval (or create all at once if user says "just do it")
4. **Verify** — Confirm everything is wired up

### Example Conversation

```
User: I want to process CSV files. Validate them, transform
      the data, then save results.

Agent: I'll set up a "CSV Pipeline" stack that watches inbox/
       for CSV files, with 3 cards:

       1. "Validate" — check headers and types
       2. "Transform" — normalize columns
       3. "Save" — write to output/

       First, let me create the folders.
       [creates inbox/ and output/ folders]

       Here's the stack with all 3 cards:
       [shows code for each card]

       Create this stack?

User: Yes, create it.

Agent: Done! Your CSV Pipeline is ready.

       Stack: "CSV Pipeline" (watches inbox/*.csv)
         Card 1: "Validate"  ✅
         Card 2: "Transform" ✅
         Card 3: "Save"      ✅

       Drop a CSV into inbox/ to test it.
```

---

## Migration from Current Code

The current codebase has `Card` with a `trigger` field and no Stack concept. Here's what changes:

### Rust Types (leaf-core)

**Before:**
```rust
pub struct Card {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger: TriggerConfig,   // trigger on Card
    pub program: ProgramConfig,
    pub enabled: bool,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**After:**
```rust
pub struct Stack {
    pub id: Uuid,
    pub project_id: Uuid,
    pub name: String,
    pub description: String,
    pub trigger: TriggerConfig,       // trigger moves here
    pub enabled: bool,
    pub source_session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct Card {
    pub id: Uuid,
    pub stack_id: Uuid,               // every card belongs to a stack
    pub name: String,
    pub description: String,
    pub program: ProgramConfig,        // card is just a program step
    pub position: i32,
    pub enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Database Migration

```sql
-- Create stacks table from existing cards
CREATE TABLE stacks (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    trigger_type TEXT NOT NULL,
    trigger_config TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    source_session_id TEXT,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Migrate: each existing card becomes a stack with one card
INSERT INTO stacks (id, project_id, name, description, trigger_type,
                    trigger_config, enabled, source_session_id, created_at, updated_at)
SELECT id, project_id, name, description,
       json_extract(trigger, '$.type'),
       trigger,
       enabled, session_id, created_at, updated_at
FROM cards_old;

-- Recreate cards table with new schema
CREATE TABLE cards_new (
    id TEXT PRIMARY KEY,
    stack_id TEXT NOT NULL REFERENCES stacks(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    program_path TEXT NOT NULL,
    program_config TEXT NOT NULL,
    position INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL,
    updated_at DATETIME NOT NULL
);

-- Migrate: each old card becomes a card in its corresponding stack
INSERT INTO cards_new (id, stack_id, name, description, program_path,
                       program_config, position, enabled, created_at, updated_at)
SELECT
    ('card_' || id),    -- new card ID (stack took the old card's ID)
    id,                 -- stack_id = old card's id
    name, description,
    json_extract(program, '$.entrypoint'),
    program,
    0,                  -- position 0 (only card in the stack)
    1,                  -- enabled
    created_at, updated_at
FROM cards_old;
```

### Affected Crates

| Crate | Changes |
|-------|---------|
| `leaf-core` | Add `Stack` struct, update `Card` struct (remove trigger, add stack_id + position) |
| `leaf-db` | New `stacks` table, update `cards` table, migration, CRUD queries |
| `leaf-app` | Update Tauri commands: `list_stacks`, `create_stack`, etc. Update trigger matching to iterate stacks. Update event emission. |
| `leaf-agent` | Replace `propose_card` / `create_card` tools with `propose_stack` / `add_card`. Update prompts. |
| `leaf-executor` | Add `LEAF_STACK_ID`, `LEAF_STEP_INDEX`, `LEAF_PREV_*` env vars. Add sequential execution logic. |
| `leaf-watcher` | No changes (still emits file events the same way) |
| `leaf-mcp` | No changes |
| `ui` | Update card list → stack list. Stack detail shows ordered cards. Execution view shows per-card results. |

---

## Open Questions

1. **Sandbox write permissions** — The current Deno sandbox only allows writing to `.leaf/outputs/`. The spec assumes cards can write anywhere in the project. This affects subcard chaining if cards need to write to watched folders. See separate discussion on sandbox permissions.

2. **Single-card UI simplification** — Should the UI hide the Stack/Card distinction for stacks with one card? Recommendation: Yes. Show it as "CSV Analyzer" not "CSV Analyzer → Step 1: CSV Analyzer". Only reveal the stack structure when there are multiple cards.

3. **Card reuse** — A Card belongs to exactly one Stack (FK constraint). If users want the same logic in multiple stacks, the agent duplicates the program. This keeps the model simple. Re-evaluate if duplication becomes a pain point.

4. **Stack-to-Stack chaining** — A stack's output card can write to a folder watched by another stack's trigger. This gives arbitrary pipeline depth through events, without any nesting machinery. The system already supports this — no new code needed.
