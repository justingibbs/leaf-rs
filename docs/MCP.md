# LEAF MCP Integration

LEAF integrates with the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/), allowing the AI agent and card programs to use external tools and services.

## What is MCP?

MCP is an open protocol developed by Anthropic that enables AI models to interact with external tools, data sources, and services. LEAF acts as an MCP client, connecting to MCP servers to extend its capabilities.

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│    LEAF     │────►│ MCP Server  │────►│  External   │
│   (Client)  │◄────│  (stdio)    │◄────│   Service   │
└─────────────┘     └─────────────┘     └─────────────┘
```

## Built-in MCP Servers

LEAF comes pre-configured with these MCP servers (disabled by default):

| Server | Description | Command |
|--------|-------------|---------|
| `filesystem` | Read/write files | `npx -y @anthropic/mcp-server-filesystem` |
| `fetch` | Fetch web content | `npx -y @anthropic/mcp-server-fetch` |
| `memory` | Persistent key-value storage | `npx -y @anthropic/mcp-server-memory` |

### Requirements

Built-in servers require **Node.js** to be installed:

```bash
# Check Node.js is installed
node --version

# If not, install via your package manager
brew install node  # macOS
```

## Managing MCP Servers

### List Servers

```bash
curl http://127.0.0.1:8000/api/mcp/servers
```

### Enable a Server

```bash
curl -X POST http://127.0.0.1:8000/api/mcp/servers/fetch/enable
```

### Connect to a Server

```bash
curl -X POST http://127.0.0.1:8000/api/mcp/servers/fetch/connect
```

### Add a Custom Server

```bash
curl -X POST http://127.0.0.1:8000/api/mcp/servers \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-server",
    "name": "My Custom Server",
    "command": "python",
    "args": ["-m", "my_mcp_server"],
    "env": {"API_KEY": "secret"},
    "enabled": true,
    "description": "My custom MCP integration"
  }'
```

### Connect All Enabled Servers

```bash
curl -X POST http://127.0.0.1:8000/api/mcp/connect-all
```

## Using MCP Tools

### In the AI Agent

Once servers are connected, the agent can use MCP tools:

```
User: "Fetch the contents of https://example.com"
Agent: [uses fetch/fetch tool] Here's what I found...
```

The agent has access to:
- `call_mcp_tool(server_id, tool_name, arguments)` - Call any MCP tool
- `list_mcp_tools()` - Discover available tools

### In Card Programs

Card programs can use the MCPClient helper:

```python
from leaf.mcp.card_helper import MCPClient

def main():
    mcp = MCPClient()

    # List available tools
    for tool in mcp.list_tools():
        print(f"{tool['server_id']}/{tool['name']}: {tool['description']}")

    # Call a tool
    result = mcp.call("fetch", "fetch", url="https://example.com")

    if result["success"]:
        content = result["output"]
        print(f"Fetched {len(content)} characters")
    else:
        print(f"Error: {result['error']}")
```

### Via API

```bash
# List available tools
curl http://127.0.0.1:8000/api/mcp/tools

# Call a tool
curl -X POST http://127.0.0.1:8000/api/mcp/tools/call \
  -H "Content-Type: application/json" \
  -d '{
    "server_id": "fetch",
    "tool_name": "fetch",
    "arguments": {"url": "https://example.com"}
  }'
```

## Server Configuration

MCP server configurations are stored globally at `~/.config/leaf/mcp.json`:

```json
{
  "servers": {
    "fetch": {
      "id": "fetch",
      "name": "Web Fetch",
      "command": "npx",
      "args": ["-y", "@anthropic/mcp-server-fetch"],
      "env": {},
      "enabled": true,
      "description": "Fetch content from URLs"
    },
    "my-server": {
      "id": "my-server",
      "name": "My Server",
      "command": "/path/to/server",
      "args": ["--config", "prod"],
      "env": {"API_KEY": "secret"},
      "enabled": true
    }
  }
}
```

## Creating Custom MCP Servers

### Python Server Example

```python
#!/usr/bin/env python3
"""Simple MCP server example."""

import json
import sys


def handle_request(request):
    """Handle a JSON-RPC request."""
    method = request.get("method")
    params = request.get("params", {})
    request_id = request.get("id")

    if method == "initialize":
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"tools": {}},
                "serverInfo": {"name": "my-server", "version": "1.0.0"}
            }
        }

    elif method == "tools/list":
        return {
            "jsonrpc": "2.0",
            "id": request_id,
            "result": {
                "tools": [
                    {
                        "name": "my_tool",
                        "description": "Does something useful",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "input": {"type": "string"}
                            },
                            "required": ["input"]
                        }
                    }
                ]
            }
        }

    elif method == "tools/call":
        tool_name = params.get("name")
        arguments = params.get("arguments", {})

        if tool_name == "my_tool":
            result = f"Processed: {arguments.get('input', '')}"
            return {
                "jsonrpc": "2.0",
                "id": request_id,
                "result": {
                    "content": [{"type": "text", "text": result}]
                }
            }

    return {
        "jsonrpc": "2.0",
        "id": request_id,
        "error": {"code": -32601, "message": "Method not found"}
    }


def main():
    """Main server loop."""
    for line in sys.stdin:
        request = json.loads(line)
        response = handle_request(request)
        print(json.dumps(response), flush=True)


if __name__ == "__main__":
    main()
```

### Register Your Server

```bash
curl -X POST http://127.0.0.1:8000/api/mcp/servers \
  -H "Content-Type: application/json" \
  -d '{
    "id": "my-server",
    "name": "My Server",
    "command": "python",
    "args": ["/path/to/my_server.py"],
    "enabled": true
  }'
```

## Common MCP Servers

### Official Anthropic Servers

| Package | Description |
|---------|-------------|
| `@anthropic/mcp-server-filesystem` | File operations |
| `@anthropic/mcp-server-fetch` | HTTP requests |
| `@anthropic/mcp-server-memory` | Key-value storage |
| `@anthropic/mcp-server-github` | GitHub API |
| `@anthropic/mcp-server-sqlite` | SQLite database |
| `@anthropic/mcp-server-postgres` | PostgreSQL database |

### Installing a Server

```bash
# Test the server works
npx -y @anthropic/mcp-server-fetch

# Add to LEAF
curl -X POST http://127.0.0.1:8000/api/mcp/servers \
  -H "Content-Type: application/json" \
  -d '{
    "id": "github",
    "name": "GitHub",
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-server-github"],
    "env": {"GITHUB_TOKEN": "your-token"},
    "enabled": true
  }'
```

## Tool Discovery

### List All Tools

```bash
curl http://127.0.0.1:8000/api/mcp/tools
```

Response:
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
      },
      "max_length": {
        "type": "integer",
        "description": "Maximum content length"
      }
    }
  },
  {
    "server_id": "filesystem",
    "name": "read_file",
    "description": "Read a file from disk",
    "parameters": {
      "path": {
        "type": "string",
        "description": "File path to read"
      }
    }
  }
]
```

### In Card Programs

```python
from leaf.mcp.card_helper import MCPClient

mcp = MCPClient()

# Check if a specific tool is available
if mcp.has_tool("fetch", "fetch"):
    result = mcp.call("fetch", "fetch", url="https://example.com")
```

## Error Handling

### Connection Errors

```python
result = mcp.call("fetch", "fetch", url="https://example.com")

if not result["success"]:
    error = result.get("error", "Unknown error")
    print(f"MCP call failed: {error}")
```

### Server Not Connected

```python
# Check if server is available
tools = mcp.list_tools()
server_ids = set(t["server_id"] for t in tools)

if "fetch" not in server_ids:
    print("Fetch server not connected")
```

## Best Practices

### 1. Check Tool Availability

```python
def fetch_url(url):
    mcp = MCPClient()

    if not mcp.has_tool("fetch", "fetch"):
        raise RuntimeError("Fetch MCP server not available")

    return mcp.call("fetch", "fetch", url=url)
```

### 2. Handle Errors Gracefully

```python
result = mcp.call("fetch", "fetch", url=url)

if result["success"]:
    return process_content(result["output"])
else:
    # Fall back to alternative method
    return fetch_with_requests(url)
```

### 3. Use Async for Performance

```python
from leaf.mcp.card_helper import MCPClient

async def main():
    mcp = MCPClient()

    # Use async version for better performance
    result = await mcp.call_async("fetch", "fetch", url=url)
```

### 4. Secure Your Credentials

```bash
# Use environment variables for sensitive data
curl -X POST http://127.0.0.1:8000/api/mcp/servers \
  -d '{
    "id": "github",
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-server-github"],
    "env": {"GITHUB_TOKEN": "'$GITHUB_TOKEN'"}
  }'
```

## Troubleshooting

### Server Won't Connect

1. Check command works manually:
   ```bash
   npx -y @anthropic/mcp-server-fetch
   ```

2. Check Node.js is installed:
   ```bash
   node --version
   ```

3. Check server logs in LEAF output

### Tool Call Fails

1. Verify server is connected:
   ```bash
   curl http://127.0.0.1:8000/api/mcp/servers/{id}
   # Look for "connected": true
   ```

2. List available tools:
   ```bash
   curl http://127.0.0.1:8000/api/mcp/tools
   ```

3. Check tool parameters match the schema

### Server Disconnects

Servers may disconnect if:
- The process crashes
- Timeout occurs
- System resources are exhausted

Reconnect with:
```bash
curl -X POST http://127.0.0.1:8000/api/mcp/servers/{id}/connect
```
