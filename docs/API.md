# LEAF API Reference

This document describes the REST API and WebSocket endpoints for LEAF.

**Base URL:** `http://127.0.0.1:8000`

## Table of Contents

- [Health & Status](#health--status)
- [Projects](#projects)
- [Cards](#cards)
- [Chat](#chat)
- [Executions](#executions)
- [Events](#events)
- [MCP Servers](#mcp-servers)
- [WebSocket](#websocket)

---

## Health & Status

### GET /

Health check endpoint.

**Response:**
```json
{
  "name": "LEAF",
  "version": "0.1.0",
  "status": "running"
}
```

### GET /health

Simple health check.

**Response:**
```json
{
  "status": "healthy"
}
```

---

## Projects

### POST /api/projects

Create a new project.

**Request:**
```json
{
  "path": "/path/to/project",
  "name": "My Project"
}
```

**Response:**
```json
{
  "id": "proj_abc123",
  "path": "/path/to/project",
  "name": "My Project",
  "created_at": "2024-01-15T10:30:00"
}
```

### GET /api/projects

List all projects.

**Response:**
```json
[
  {
    "id": "proj_abc123",
    "path": "/path/to/project",
    "name": "My Project",
    "created_at": "2024-01-15T10:30:00"
  }
]
```

### GET /api/projects/current

Get the current project context.

**Response:**
```json
{
  "id": "proj_abc123",
  "path": "/path/to/project",
  "name": "My Project",
  "database_path": "/path/to/project/.leaf/leaf.db"
}
```

**Error (no project selected):**
```json
{
  "detail": "No project selected"
}
```

### POST /api/projects/switch

Switch to a different project.

**Request:**
```json
{
  "project_id": "proj_abc123"
}
```

**Response:**
```json
{
  "status": "switched",
  "project": {
    "id": "proj_abc123",
    "name": "My Project"
  }
}
```

### POST /api/projects/close

Close the current project.

**Response:**
```json
{
  "status": "closed"
}
```

### DELETE /api/projects/{project_id}

Delete a project from the registry (does not delete files).

**Response:**
```json
{
  "status": "deleted"
}
```

---

## Cards

**Note:** All card endpoints require an active project.

### POST /api/cards

Create a new card.

**Request:**
```json
{
  "name": "CSV Processor",
  "description": "Processes CSV files and generates reports",
  "user_prompt": "Process CSV files in inbox",
  "trigger_config": {
    "type": "file_created",
    "folder": "inbox",
    "pattern": "*.csv"
  },
  "program_config": {
    "language": "python",
    "entrypoint": "main.py",
    "dependencies": ["pandas"]
  },
  "timeout_seconds": 300,
  "retry_count": 3,
  "enabled": true
}
```

**Trigger Types:**
- `file_created` - Triggered when a file is created
- `file_modified` - Triggered when a file is modified
- `manual` - Triggered manually via API
- `schedule` - Triggered on a schedule (cron)

**Response:**
```json
{
  "id": "card_def456",
  "name": "CSV Processor",
  "description": "Processes CSV files and generates reports",
  "trigger_config": {...},
  "program_path": ".leaf/programs/csv-processor-a1b2c3",
  "enabled": true,
  "created_at": "2024-01-15T10:35:00",
  "run_count": 0
}
```

### GET /api/cards

List all cards in the current project.

**Query Parameters:**
- `enabled_only` (bool) - Only return enabled cards

**Response:**
```json
[
  {
    "id": "card_def456",
    "name": "CSV Processor",
    "enabled": true,
    "run_count": 5,
    "last_run_at": "2024-01-15T11:00:00"
  }
]
```

### GET /api/cards/{card_id}

Get a specific card.

**Response:**
```json
{
  "id": "card_def456",
  "name": "CSV Processor",
  "description": "...",
  "trigger_config": {...},
  "program_config": {...},
  "program_path": ".leaf/programs/csv-processor-a1b2c3",
  "timeout_seconds": 300,
  "retry_count": 3,
  "enabled": true,
  "created_at": "2024-01-15T10:35:00",
  "updated_at": "2024-01-15T10:35:00",
  "run_count": 5,
  "last_run_at": "2024-01-15T11:00:00"
}
```

### PATCH /api/cards/{card_id}

Update a card.

**Request:**
```json
{
  "name": "Updated Name",
  "enabled": false
}
```

Only provided fields are updated.

**Response:** Updated card object.

### DELETE /api/cards/{card_id}

Delete a card.

**Response:**
```json
{
  "status": "deleted"
}
```

### POST /api/cards/{card_id}/enable

Enable a card.

**Response:** Updated card object with `enabled: true`.

### POST /api/cards/{card_id}/disable

Disable a card.

**Response:** Updated card object with `enabled: false`.

### POST /api/cards/{card_id}/trigger

Manually trigger a card execution.

**Response:**
```json
{
  "status": "triggered",
  "execution_id": "exec_xyz789"
}
```

---

## Chat

### POST /api/chat

Send a message to the AI agent.

**Request:**
```json
{
  "content": "Create an automation that processes CSV files"
}
```

**Response:**
```json
{
  "response": "I'll create a card for you...",
  "user_message_id": "msg_abc123",
  "assistant_message_id": "msg_def456"
}
```

### GET /api/chat/history

Get chat history for the current project.

**Response:**
```json
[
  {
    "id": "msg_abc123",
    "role": "user",
    "content": "Create an automation...",
    "created_at": "2024-01-15T10:30:00"
  },
  {
    "id": "msg_def456",
    "role": "assistant",
    "content": "I'll create a card...",
    "created_at": "2024-01-15T10:30:05"
  }
]
```

### DELETE /api/chat/history

Clear chat history.

**Response:**
```json
{
  "status": "cleared"
}
```

---

## Executions

### GET /api/executions

List executions.

**Query Parameters:**
- `card_id` (string) - Filter by card
- `status` (string) - Filter by status (pending, running, completed, failed)
- `limit` (int) - Maximum results (default: 50)

**Response:**
```json
[
  {
    "id": "exec_xyz789",
    "card_id": "card_def456",
    "event_id": "evt_ghi012",
    "status": "completed",
    "started_at": "2024-01-15T11:00:00",
    "completed_at": "2024-01-15T11:00:05",
    "stdout": "Processing complete\n",
    "stderr": "",
    "error": null,
    "result": {
      "exit_code": 0,
      "duration_seconds": 5.2,
      "attempts": 1
    }
  }
]
```

### GET /api/executions/{execution_id}

Get a specific execution.

**Response:** Single execution object.

### POST /api/executions/manual

Manually execute a card.

**Request:**
```json
{
  "card_id": "card_def456"
}
```

**Response:** Execution object.

---

## Events

### GET /api/debug/events

List recent events (debug endpoint).

**Query Parameters:**
- `limit` (int) - Maximum results (default: 50)

**Response:**
```json
[
  {
    "id": "evt_ghi012",
    "type": "file.created",
    "timestamp": "2024-01-15T11:00:00",
    "payload": {
      "path": "/path/to/inbox/data.csv",
      "filename": "data.csv",
      "folder": "/path/to/inbox",
      "size": 1234
    },
    "status": "completed",
    "matched_cards": ["card_def456"],
    "execution_id": "exec_xyz789"
  }
]
```

---

## MCP Servers

### GET /api/mcp/servers

List all configured MCP servers.

**Response:**
```json
[
  {
    "id": "fetch",
    "name": "Web Fetch",
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-server-fetch"],
    "env": {},
    "enabled": false,
    "description": "Fetch content from URLs",
    "connected": false
  }
]
```

### POST /api/mcp/servers

Add a new MCP server.

**Request:**
```json
{
  "id": "my-server",
  "name": "My Custom Server",
  "command": "python",
  "args": ["-m", "my_mcp_server"],
  "env": {"API_KEY": "secret"},
  "enabled": true,
  "description": "My custom MCP server"
}
```

**Response:** Server object.

### GET /api/mcp/servers/{server_id}

Get a specific server.

**Response:** Server object.

### DELETE /api/mcp/servers/{server_id}

Remove a server configuration.

**Response:**
```json
{
  "status": "deleted"
}
```

### POST /api/mcp/servers/{server_id}/enable

Enable a server.

**Response:** Updated server object.

### POST /api/mcp/servers/{server_id}/disable

Disable a server.

**Response:** Updated server object.

### POST /api/mcp/servers/{server_id}/connect

Connect to a server.

**Response:** Server object with `connected: true`.

### POST /api/mcp/servers/{server_id}/disconnect

Disconnect from a server.

**Response:** Server object with `connected: false`.

### GET /api/mcp/tools

List all available tools from connected servers.

**Response:**
```json
[
  {
    "server_id": "fetch",
    "name": "fetch",
    "description": "Fetch content from a URL",
    "parameters": {
      "url": {
        "type": "string",
        "description": "URL to fetch"
      }
    }
  }
]
```

### POST /api/mcp/tools/call

Call an MCP tool.

**Request:**
```json
{
  "server_id": "fetch",
  "tool_name": "fetch",
  "arguments": {
    "url": "https://example.com"
  }
}
```

**Response:**
```json
{
  "success": true,
  "output": "<!DOCTYPE html>...",
  "error": null
}
```

### POST /api/mcp/connect-all

Connect to all enabled servers.

**Response:**
```json
{
  "connected": ["fetch", "filesystem"]
}
```

### POST /api/mcp/disconnect-all

Disconnect from all servers.

**Response:**
```json
{
  "status": "disconnected"
}
```

---

## WebSocket

### WS /ws

Main WebSocket for real-time events.

**Connection:**
```javascript
const ws = new WebSocket('ws://127.0.0.1:8000/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log(data.type, data.data);
};
```

**Event Types:**

```json
// File event
{
  "type": "file.created",
  "data": {
    "event_id": "evt_abc123",
    "timestamp": "2024-01-15T11:00:00",
    "payload": {
      "path": "/path/to/file.csv",
      "filename": "file.csv"
    },
    "status": "pending"
  }
}

// Event status update
{
  "type": "event.updated",
  "data": {
    "event_id": "evt_abc123",
    "status": "completed",
    "matched_cards": ["card_def456"],
    "execution_id": "exec_xyz789"
  }
}

// Execution started
{
  "type": "execution.started",
  "data": {
    "execution_id": "exec_xyz789",
    "card_id": "card_def456",
    "event_id": "evt_abc123"
  }
}

// Execution completed
{
  "type": "execution.completed",
  "data": {
    "execution_id": "exec_xyz789",
    "card_id": "card_def456",
    "duration_seconds": 5.2
  }
}

// Execution failed
{
  "type": "execution.failed",
  "data": {
    "execution_id": "exec_xyz789",
    "card_id": "card_def456",
    "error": "Exit code 1",
    "attempts": 3
  }
}

// Execution retry
{
  "type": "execution.retry",
  "data": {
    "execution_id": "exec_xyz789",
    "card_id": "card_def456",
    "attempt": 2,
    "max_attempts": 3,
    "error": "Temporary failure"
  }
}
```

### WS /ws/chat

Streaming chat WebSocket.

**Send Message:**
```json
{
  "type": "message",
  "content": "Create an automation..."
}
```

**Receive Chunks:**
```json
{
  "type": "chunk",
  "content": "I'll "
}
{
  "type": "chunk",
  "content": "create "
}
// ...
{
  "type": "done"
}
```

**Error:**
```json
{
  "type": "error",
  "message": "Error description"
}
```

---

## Error Responses

All endpoints return standard error responses:

**400 Bad Request:**
```json
{
  "detail": "Invalid request data"
}
```

**404 Not Found:**
```json
{
  "detail": "Resource not found"
}
```

**500 Internal Server Error:**
```json
{
  "detail": "Internal server error"
}
```

## Rate Limiting

Currently, LEAF does not implement rate limiting as it's designed for local use. If deploying in a shared environment, consider adding rate limiting middleware.

## Authentication

LEAF currently does not require authentication as it's designed for local desktop use. The API binds to `127.0.0.1` by default to prevent external access.
