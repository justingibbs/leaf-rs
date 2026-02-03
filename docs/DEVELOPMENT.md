# LEAF Development Guide

This guide covers setting up a development environment, running tests, and contributing to LEAF.

## Prerequisites

- **Python 3.11+** - Required for type hints and async features
- **UV** - Modern Python package manager ([install](https://docs.astral.sh/uv/getting-started/installation/))
- **Node.js** - Optional, for MCP servers
- **Git** - Version control

## Development Setup

### 1. Clone the Repository

```bash
git clone https://github.com/yourusername/leaf.git
cd leaf
```

### 2. Install Dependencies

```bash
# Install all dependencies including dev tools
uv sync

# Or install with extras
uv sync --all-extras
```

### 3. Configure Environment

```bash
# Copy the example environment file
cp .env.example .env

# Edit with your API key
# Get a free key from https://ai.pydantic.dev/
```

Required environment variables:

```bash
PYDANTIC_AI_API_KEY=your-key-here
LEAF_MODEL=google-gla:gemini-2.0-flash
```

### 4. Verify Installation

```bash
# Run tests to verify everything works
uv run pytest

# Start the development server
uv run python -m leaf.main
```

## Project Structure

```
leaf/
├── src/leaf/                 # Main source code
│   ├── __init__.py
│   ├── main.py              # FastAPI application entry point
│   ├── agent/               # PydanticAI agent
│   │   ├── __init__.py
│   │   ├── leaf_agent.py    # Agent configuration and tools
│   │   ├── prompts.py       # System prompts
│   │   └── tools.py         # Tool implementations
│   ├── api/                 # FastAPI routes
│   │   ├── __init__.py
│   │   ├── app_routes.py    # Application config endpoints
│   │   ├── project_routes.py # Project and chat endpoints
│   │   ├── execution_routes.py # Execution endpoints
│   │   ├── mcp_routes.py    # MCP management endpoints
│   │   ├── websocket.py     # WebSocket handlers
│   │   └── debug_routes.py  # Debug endpoints
│   ├── cards/               # Card management
│   │   ├── __init__.py
│   │   ├── registry.py      # CRUD operations
│   │   ├── models.py        # Pydantic models
│   │   └── matcher.py       # Event matching
│   ├── core/                # Core infrastructure
│   │   ├── __init__.py
│   │   ├── config.py        # Configuration
│   │   └── events.py        # Event bus
│   ├── db/                  # Database layer
│   │   ├── __init__.py
│   │   ├── models.py        # SQLModel definitions
│   │   └── session.py       # Session management
│   ├── execution/           # Execution engine
│   │   ├── __init__.py
│   │   ├── sandbox.py       # UV sandbox
│   │   └── runner.py        # Execution orchestration
│   ├── mcp/                 # MCP integration
│   │   ├── __init__.py
│   │   ├── protocol.py      # Message types
│   │   ├── client.py        # MCP client
│   │   ├── config.py        # Server configuration
│   │   ├── registry.py      # Connection registry
│   │   ├── agent_tools.py   # Agent integration
│   │   └── card_helper.py   # Helper for cards
│   ├── projects/            # Project management
│   │   ├── __init__.py
│   │   ├── manager.py       # Project CRUD
│   │   ├── context.py       # Current project context
│   │   └── models.py        # Project models
│   └── watcher/             # File watching
│       ├── __init__.py
│       └── file_watcher.py  # watchfiles integration
├── tests/                   # Test suite
│   ├── test_phase1.py       # Core infrastructure tests
│   ├── test_phase2.py       # File watcher tests
│   ├── test_phase3.py       # Cards tests
│   ├── test_phase4.py       # Agent tests
│   ├── test_phase5.py       # Execution tests
│   └── test_phase6.py       # MCP tests
├── docs/                    # Documentation
├── pyproject.toml           # Project configuration
├── .env.example             # Environment template
└── README.md
```

## Running Tests

### Run All Tests

```bash
uv run pytest
```

### Run with Verbose Output

```bash
uv run pytest -v
```

### Run Specific Test File

```bash
uv run pytest tests/test_phase3.py -v
```

### Run Specific Test Class

```bash
uv run pytest tests/test_phase3.py::TestCardRegistry -v
```

### Run with Coverage

```bash
uv run pytest --cov=leaf --cov-report=html
```

### Run Tests in Parallel

```bash
uv run pytest -n auto
```

## Development Server

### Start the Server

```bash
# With auto-reload
uv run python -m leaf.main

# Or specify a different port
LEAF_PORT=3000 uv run python -m leaf.main
```

### API Documentation

Once running, view the auto-generated API docs:

- Swagger UI: http://127.0.0.1:8000/docs
- ReDoc: http://127.0.0.1:8000/redoc

## Code Style

### Formatting

We use Ruff for formatting:

```bash
# Format code
uv run ruff format .

# Check formatting without changes
uv run ruff format --check .
```

### Linting

```bash
# Run linter
uv run ruff check .

# Auto-fix issues
uv run ruff check --fix .
```

### Type Checking

```bash
# Run type checker
uv run mypy src/leaf
```

## Database Migrations

LEAF uses SQLModel with SQLite. The database is created automatically when a project is initialized.

### Reset Project Database

```bash
# Delete the project's database to reset
rm /path/to/project/.leaf/leaf.db
```

The database will be recreated on next project access.

## Debugging

### Enable Debug Logging

```bash
# Set log level
LEAF_LOG_LEVEL=DEBUG uv run python -m leaf.main
```

### Debug Endpoints

The debug routes are available at `/api/debug/`:

- `GET /api/debug/context` - Current project context
- `GET /api/debug/watches` - Active file watchers
- `GET /api/debug/events?limit=50` - Recent events

### Using the Python Debugger

```python
# Add breakpoint in code
import pdb; pdb.set_trace()

# Or use breakpoint() in Python 3.7+
breakpoint()
```

## Adding New Features

### 1. Adding a New API Endpoint

1. Create or modify a route file in `src/leaf/api/`
2. Add the router to `src/leaf/main.py`
3. Add tests in `tests/`

Example:

```python
# src/leaf/api/my_routes.py
from fastapi import APIRouter

router = APIRouter(prefix="/api/my-feature", tags=["my-feature"])

@router.get("/")
async def my_endpoint():
    return {"message": "Hello"}
```

### 2. Adding an Agent Tool

1. Add the tool function in `src/leaf/agent/leaf_agent.py`
2. Use the `@agent.tool` decorator
3. Add tests

Example:

```python
@agent.tool
async def my_tool(ctx: RunContext[AgentContext], param: str) -> str:
    """Tool description for the AI."""
    # Implementation
    return "result"
```

### 3. Adding a Database Model

1. Add the model in `src/leaf/db/models.py`
2. The table is created automatically on first project access

Example:

```python
class MyModel(SQLModel, table=True):
    __tablename__ = "my_table"

    id: str = Field(primary_key=True)
    name: str
    created_at: datetime = Field(default_factory=datetime.now)
```

## Testing Guidelines

### Test Structure

- Each test file corresponds to a development phase
- Use pytest fixtures for common setup
- Mock external services (AI, MCP servers)

### Writing Tests

```python
import pytest
from fastapi.testclient import TestClient
from leaf.main import app

@pytest.fixture
def client():
    return TestClient(app)

def test_my_feature(client):
    response = client.get("/api/my-endpoint")
    assert response.status_code == 200
```

### Async Tests

```python
@pytest.mark.asyncio
async def test_async_function():
    result = await my_async_function()
    assert result == expected
```

## Contributing

### Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes
4. Run tests (`uv run pytest`)
5. Run linter (`uv run ruff check .`)
6. Commit with a descriptive message
7. Push and create a Pull Request

### Commit Messages

Use conventional commits:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation
- `test:` - Tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance

Example:

```
feat: add schedule trigger type for cards

- Support cron expressions for scheduled execution
- Add next_run_at field to card model
- Create scheduler background task
```

## Troubleshooting

### Common Issues

**UV not found:**
```bash
# Install UV
curl -LsSf https://astral.sh/uv/install.sh | sh
```

**Module not found errors:**
```bash
# Reinstall dependencies
uv sync --reinstall
```

**Database locked:**
```bash
# Another process may be using the database
# Check for running LEAF instances
```

**MCP server won't connect:**
```bash
# Ensure Node.js is installed for npm-based MCP servers
node --version

# Check the server command is correct
npx -y @anthropic/mcp-server-fetch
```
