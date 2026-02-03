# LEAF Phase 7: Tauri + React Frontend Implementation Plan

## Overview

Build the desktop frontend for LEAF using Tauri 2.x + React + TypeScript. The backend (FastAPI at localhost:8000) is complete.

## Tech Stack

| Layer | Technology |
|-------|------------|
| Desktop Shell | Tauri 2.x (macOS) |
| UI Framework | React 18 + TypeScript |
| Styling | Tailwind CSS + shadcn/ui |
| Server State | React Query |
| UI State | Zustand |
| Routing | React Router |
| Code Editor | Monaco Editor |

## Project Structure

```
leaf/
├── frontend/                    # React application
│   ├── src/
│   │   ├── api/                # API client (projects.ts, cards.ts, chat.ts, etc.)
│   │   ├── hooks/              # useWebSocket, useChatWebSocket, useCards, etc.
│   │   ├── stores/             # Zustand: appStore, projectStore, eventStore
│   │   ├── components/
│   │   │   ├── layout/         # AppShell, Sidebar, Header
│   │   │   ├── projects/       # ProjectPicker, ProjectCard
│   │   │   ├── cards/          # CardList, CardDetail, CardEditor
│   │   │   ├── chat/           # ChatPanel, ChatMessage, ChatInput
│   │   │   ├── events/         # EventQueue, EventItem
│   │   │   ├── executions/     # ExecutionList, ExecutionDetail
│   │   │   ├── mcp/            # McpServerList, AddServerDialog
│   │   │   ├── settings/       # SettingsPanel
│   │   │   └── ui/             # shadcn/ui components
│   │   ├── pages/              # Route pages
│   │   └── lib/                # Utilities
│   ├── package.json
│   ├── vite.config.ts
│   └── tailwind.config.js
│
└── tauri/                       # Tauri shell
    ├── src/main.rs
    ├── Cargo.toml
    └── tauri.conf.json
```

## Views to Implement

1. **Project Picker** (`/`) - Launch screen with project list, create/open buttons
2. **Dashboard** (`/project`) - Main view with Chat Panel + Cards Sidebar + Event Queue
3. **Card Detail** (`/project/cards/:id`) - Trigger config, Monaco code editor, execution history
4. **Executions** (`/project/executions`) - Global execution history with filtering
5. **MCP Servers** (`/project/mcp`) - Server management, tools list
6. **Settings** (`/settings`) - Theme toggle, app config

## State Management Strategy

**React Query** (server state):
- `['projects']` - Project list
- `['cards']` - Cards in current project
- `['executions']` - Execution history
- `['chat-history']` - Chat messages
- `['mcp-servers']` - MCP server configs

**Zustand** (UI state):
- `appStore` - theme, sidebar state
- `projectStore` - current project ID
- `eventStore` - real-time WebSocket events

## WebSocket Integration

Two connections:
1. `/ws` - Real-time events (file.created, execution.*, card.*) → eventStore + query invalidation
2. `/ws/chat` - Streaming chat (chat.response.chunk) → ChatPanel streaming display

## Implementation Order

### Phase 1: Scaffolding
1. Initialize Tauri project in `tauri/`
2. Initialize React/Vite project in `frontend/`
3. Configure Tailwind + shadcn/ui
4. Set up React Query + Zustand + React Router

### Phase 2: Core Infrastructure
5. API client layer (`frontend/src/api/`)
6. React Query hooks (`frontend/src/hooks/useCards.ts`, etc.)
7. Zustand stores (`frontend/src/stores/`)
8. WebSocket hooks (`useWebSocket.ts`, `useChatWebSocket.ts`)

### Phase 3: Project Picker
9. ProjectPicker component with project list
10. Create project dialog
11. Open folder dialog (Tauri integration)

### Phase 4: Main Layout
12. AppShell with header, sidebar, content
13. CardList sidebar component
14. Route guards (require project)

### Phase 5: Chat Interface
15. ChatPanel with message list
16. ChatInput with streaming indicator
17. useChatWebSocket integration

### Phase 6: Event Queue
18. EventQueue component
19. useWebSocket integration with eventStore

### Phase 7: Card Detail
20. CardDetail page with trigger display
21. CardEditor with Monaco
22. CardExecutions list

### Phase 8: Remaining Views
23. ExecutionList/ExecutionDetail pages
24. McpServerList + AddServerDialog
25. SettingsPanel

### Phase 9: Tauri Integration
26. System tray support
27. Sidecar configuration for production
28. Native file dialogs

## Key Files to Create

| File | Purpose |
|------|---------|
| `frontend/src/api/client.ts` | Base fetch wrapper with error handling |
| `frontend/src/hooks/useWebSocket.ts` | Main WS connection + query invalidation |
| `frontend/src/hooks/useChatWebSocket.ts` | Streaming chat connection |
| `frontend/src/stores/projectStore.ts` | Current project context |
| `frontend/src/components/layout/AppShell.tsx` | Main layout shell |
| `frontend/src/components/chat/ChatPanel.tsx` | Streaming chat UI |
| `frontend/src/components/cards/CardEditor.tsx` | Monaco editor wrapper |
| `frontend/src/components/events/EventQueue.tsx` | Real-time event feed |
| `tauri/tauri.conf.json` | Tauri app configuration |
| `tauri/src/main.rs` | Tauri entry point with tray |

## API Endpoints to Integrate

### Projects
- `GET /api/projects` - List all projects
- `POST /api/projects` - Create new project
- `POST /api/projects/switch` - Switch to project
- `POST /api/projects/close` - Close current project
- `GET /api/projects/current` - Get current project

### Cards
- `GET /api/cards` - List cards in current project
- `POST /api/cards` - Create card
- `GET /api/cards/{id}` - Get card details
- `PATCH /api/cards/{id}` - Update card
- `DELETE /api/cards/{id}` - Delete card
- `POST /api/cards/{id}/enable` - Enable card
- `POST /api/cards/{id}/disable` - Disable card
- `POST /api/cards/{id}/trigger` - Manual trigger

### Chat
- `POST /api/chat` - Send message (non-streaming)
- `GET /api/chat/history` - Get chat history
- `WS /ws/chat` - Streaming chat

### Executions
- `GET /api/executions` - List executions
- `GET /api/executions/{id}` - Get execution details

### MCP
- `GET /api/mcp/servers` - List servers
- `POST /api/mcp/servers` - Add server
- `DELETE /api/mcp/servers/{id}` - Remove server
- `POST /api/mcp/servers/{id}/connect` - Connect
- `POST /api/mcp/servers/{id}/disconnect` - Disconnect
- `GET /api/mcp/tools` - List available tools

### WebSocket Events
- `WS /ws` - Real-time events
  - `file.created`, `file.modified`
  - `execution.started`, `execution.completed`, `execution.failed`, `execution.retry`
  - `card.created`, `card.updated`, `card.deleted`

## Component Details

### ChatPanel
- Displays message history
- Shows streaming responses with typing indicator
- Handles card proposals inline (confirm/reject)
- Input with Enter to send, Shift+Enter for newline

### EventQueue
- Real-time feed of system events
- Status indicators (pending → processing → completed/failed)
- Click to expand event details
- Auto-scroll to newest events

### CardEditor
- Monaco Editor for Python syntax highlighting
- Read file from `.leaf/programs/{card-slug}/main.py`
- Toggle between read-only and edit mode
- Save triggers API update

### ProjectPicker
- Grid/list of existing projects
- "Create New" opens folder dialog → creates `.leaf/` structure
- "Open Existing" opens folder dialog → validates `.leaf/` exists
- Clicking project switches context and navigates to dashboard

## Verification

1. **Dev mode**: Run `npm run dev` in frontend/, connect to backend at localhost:8000
2. **Create project**: Use Project Picker to create a project folder
3. **Chat**: Send message, verify streaming response
4. **Card creation**: Ask agent to create a card, verify it appears in sidebar
5. **File trigger**: Add file to watched folder, verify event in queue
6. **Execution**: Verify execution completes and shows in history
7. **Tauri build**: Run `cargo tauri build`, verify standalone app works
