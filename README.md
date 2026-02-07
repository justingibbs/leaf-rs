# LEAF

**Local Event-Driven Automation Framework**

LEAF is a desktop application that lets you create automations through natural language. Describe what you want to automate, an LLM generates TypeScript code, and that code runs automatically when files are added to watched folders.

## Features

- **Natural Language Automation**: Describe your automation in plain English, and LEAF generates the code
- **File Watching**: Monitor folders for new or modified files and trigger automations automatically
- **Sandboxed Execution**: All code runs in a secure Deno sandbox with limited permissions
- **Multiple LLM Providers**: Works with Anthropic Claude, OpenAI, Google Gemini, or local Ollama
- **MCP Integration**: Extend the AI agent's capabilities with Model Context Protocol servers
- **Dark Mode**: Full dark mode support

## Installation

There are three ways to install and run LEAF:

### Option 1: Download a Release (Recommended)

The simplest way to get started — no build tools required.

1. Download the latest `.dmg` from [Releases](https://github.com/your-org/leaf-rs/releases)
2. Open the DMG and drag LEAF to your Applications folder
3. **First launch**: Right-click the app and select "Open"

> **Note**: This app is not code-signed. On first launch, macOS will warn about an unidentified developer. Right-click and select "Open" to bypass this warning.

### Option 2: Development Mode (Recommended for Contributors)

Run from source with hot reload — frontend changes update instantly.

**Prerequisites:**
- Rust (stable toolchain)
- Node.js 18+ and pnpm
- Tauri CLI (`cargo install tauri-cli`)

```bash
# Clone the repository
git clone https://github.com/your-org/leaf-rs.git
cd leaf-rs

# Install frontend dependencies
cd ui && pnpm install && cd ..

# Start the dev server
cargo tauri dev
```

To stop the dev server, press `Ctrl+C` in the terminal where it's running. This shuts down both the Rust backend and the Vite frontend dev server.

### Option 3: Build & Install from Source

Build a distributable `.dmg` locally. Use this for final testing or sharing with others.

**Prerequisites:** Same as Option 2.

```bash
# Clone and install (if you haven't already)
git clone https://github.com/your-org/leaf-rs.git
cd leaf-rs
cd ui && pnpm install && cd ..

# Build the app
cargo tauri build
```

Then:
1. Open `target/release/bundle/dmg/LEAF_0.1.0_aarch64.dmg`
2. Drag LEAF to Applications
3. **First launch**: Right-click → "Open" (required since unsigned)

## Quick Start

1. **Open or Create a Project**: Select a folder to use as your LEAF project
2. **Configure API Key**: Go to Settings (Cmd+,) and enter your LLM provider API key
3. **Start Chatting**: Describe what you want to automate
4. **Accept the Card**: When the AI proposes an automation, accept it to create a card
5. **Watch It Work**: Add files to your watched folder and watch the automation run

## Usage

### Chat Interface

The chat interface is the primary way to create automations. You can:
- Describe what you want to automate in natural language
- Ask the AI to modify existing cards
- Get help understanding how your automations work

### Cards

Cards are the individual automation units. Each card has:
- **Trigger**: When the automation runs (file created, file modified, manual, or scheduled)
- **Program**: The TypeScript code that executes
- **Settings**: Timeout, retry behavior, and other configuration

### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| Cmd+1 | Switch to Chat view |
| Cmd+2 | Switch to Cards view |
| Cmd+N | Create new chat session |
| Cmd+, | Open Settings |
| Escape | Close modal |

## Configuration

### LLM Providers

LEAF supports multiple LLM providers:

| Provider | Models | API Key Required |
|----------|--------|------------------|
| Anthropic | Claude 3.5 Sonnet, Claude 3 Opus | Yes |
| OpenAI | GPT-4o, GPT-4 Turbo | Yes |
| Google | Gemini 1.5 Pro/Flash | Yes |
| Ollama | Llama 3.1, CodeLlama, Mistral | No (local) |

### MCP Servers

Model Context Protocol (MCP) servers extend the AI agent's capabilities. Configure them in Settings > MCP Servers.

## Project Structure

```
leaf-rs/
├── crates/
│   ├── leaf-core/      # Core types and traits
│   ├── leaf-db/        # SQLite database layer
│   ├── leaf-watcher/   # File system monitoring
│   ├── leaf-agent/     # LLM agent and code generation
│   ├── leaf-executor/  # Deno sandbox execution
│   ├── leaf-mcp/       # MCP client
│   └── leaf-app/       # Tauri application
├── ui/                 # React + TypeScript frontend
└── docs/               # Documentation
```

## Development

```bash
# Run tests
cargo test --all

# Run linter
cargo clippy --all

# Type check frontend
cd ui && pnpm tsc --noEmit

# Development mode with hot reload
cargo tauri dev
```

## Architecture

LEAF follows the [Concepts & Synchronizations](https://essenceofsoftware.com/) model:

- **Project**: A folder containing `.leaf/` with database and configuration
- **Card**: An automation unit with trigger, program, and settings
- **Event**: A file system event that can trigger cards
- **Execution**: A single run of a card's program

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for more details.

## License

MIT
