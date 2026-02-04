# LEAF User Guide

This guide will help you get started with LEAF and create your first automation.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Creating Your First Automation](#creating-your-first-automation)
3. [The Chat Interface](#the-chat-interface)
4. [Understanding Cards](#understanding-cards)
5. [MCP Integration](#mcp-integration)
6. [Troubleshooting](#troubleshooting)

---

## Getting Started

### Installation

1. Download LEAF from the [releases page](https://github.com/your-org/leaf-rs/releases)
2. Open the DMG file and drag LEAF to your Applications folder
3. **Important**: Since LEAF is not code-signed, you need to right-click and select "Open" the first time

### Setting Up Your API Key

Before you can use LEAF's AI features, you need to configure an LLM provider:

1. Open LEAF
2. Press **Cmd+,** to open Settings
3. Go to the **LLM Provider** tab
4. Select your provider (Anthropic, OpenAI, Google, or Ollama)
5. Enter your API key
6. Click **Save Settings**

> **Tip**: If you want to use LEAF without an internet connection, configure Ollama with a local model like Llama 3.1.

### Creating a Project

A LEAF project is simply a folder that LEAF monitors:

1. Click **Open Project** on the welcome screen
2. Select or create a folder for your project
3. LEAF will create a `.leaf` folder to store its configuration and database

---

## Creating Your First Automation

Let's create a simple automation that processes text files.

### Step 1: Start a Chat

1. Make sure you're on the **Chat** tab (Cmd+1)
2. Click **New Chat** or press Cmd+N
3. You'll see an empty conversation

### Step 2: Describe Your Automation

Type a message describing what you want to automate. For example:

> "Create a card that watches the 'inbox' folder for text files and counts the number of words in each file, saving the result to an 'output' folder."

### Step 3: Review the Proposed Card

The AI will:
1. Understand your request
2. Generate TypeScript code
3. Propose a card with the appropriate trigger and settings

Review the proposed card carefully. You can ask questions or request modifications.

### Step 4: Accept the Card

When you're happy with the card, accept it. The card will appear in your Cards list (Cmd+2).

### Step 5: Test Your Automation

1. Create the `inbox` folder in your project directory if it doesn't exist
2. Drop a `.txt` file into the folder
3. Watch the **Events** panel to see the file detected
4. Check the **Executions** to see the card run
5. Find the output in your `output` folder

---

## The Chat Interface

### Session Management

LEAF organizes conversations into sessions:

- **New Session** (Cmd+N): Start a fresh conversation
- **Session List**: Click on previous sessions to continue them
- **Archive**: Remove sessions you no longer need

### Effective Prompting

Here are some tips for getting good results:

**Be Specific**
> "Create a card that converts PDF files to text using OCR"

is better than:

> "Process PDFs"

**Mention File Types**
> "Watch for `.csv` files and extract the email column"

**Describe the Output**
> "Save the results as JSON in the 'processed' folder"

### Example Prompts

- "Create a card that resizes images larger than 1000px to 800px width"
- "Watch for CSV files and convert them to JSON format"
- "When a markdown file is added, convert it to HTML"
- "Create a card that extracts text from PDF files and saves them as .txt"

---

## Understanding Cards

### Card Components

Each card has four main parts:

1. **Name**: A descriptive name for the automation
2. **Trigger**: What causes the card to run
3. **Program**: The TypeScript code that executes
4. **Settings**: Timeout, retries, and other configuration

### Trigger Types

| Trigger | Description | Example Use |
|---------|-------------|-------------|
| File Created | Runs when a new file matches patterns | Process new uploads |
| File Modified | Runs when an existing file changes | Re-process updated documents |
| Manual | Only runs when you click "Trigger" | On-demand processing |
| Schedule | Runs on a cron schedule | Daily reports |

### The Sandbox

Card programs run in a secure Deno sandbox:

- **Limited File Access**: Only your project folder is accessible
- **No Network by Default**: Enable in Settings if needed
- **Timeout Protection**: Long-running programs are stopped
- **Automatic Retries**: Failed executions can retry

### Managing Cards

- **Enable/Disable**: Toggle the switch to pause a card
- **Edit**: Click on a card to modify it
- **Delete**: Remove cards you no longer need
- **Trigger**: Manually run a card for testing

---

## MCP Integration

Model Context Protocol (MCP) servers extend what the AI agent can do.

### What MCP Provides

MCP servers can give the AI access to:

- External databases
- APIs and web services
- File systems beyond your project
- Custom tools you create

### Configuring MCP Servers

1. Open Settings (Cmd+,)
2. Go to the **MCP Servers** tab
3. Click **Add Server**
4. Enter the server details:
   - **Name**: A friendly name
   - **Command**: The executable path (e.g., `npx`)
   - **Arguments**: Command arguments (e.g., `-y @modelcontextprotocol/server-filesystem /path`)
   - **Environment**: Any required environment variables
5. Click **Add Server**

### Example MCP Servers

**File System Server**
```
Name: File System
Command: npx
Args: -y @modelcontextprotocol/server-filesystem /Users/me/Documents
```

**SQLite Server**
```
Name: SQLite
Command: npx
Args: -y @modelcontextprotocol/server-sqlite /path/to/database.db
```

---

## Troubleshooting

### "No API Key Configured"

Make sure you've:
1. Opened Settings (Cmd+,)
2. Selected a provider
3. Entered a valid API key
4. Clicked Save

### Card Execution Fails

Check the execution details:
1. Click on the failed execution in the Events panel
2. Look at stdout and stderr for error messages
3. Common issues:
   - Missing permissions (enable network access if needed)
   - Timeout (increase timeout in card settings)
   - Syntax errors in generated code

### Files Not Being Detected

Make sure:
1. The watcher is running (check the Events panel)
2. Your file matches the trigger patterns
3. The watch path is correct (relative to project root)

### MCP Server Won't Connect

Check:
1. The command path is correct
2. Required dependencies are installed (e.g., `npx` available)
3. Environment variables are set correctly
4. Click "Test Connection" to see specific errors

### Performance Issues

If LEAF feels slow:
1. Reduce the number of watched paths
2. Use more specific file patterns
3. Archive old chat sessions
4. Close and reopen the project

---

## Getting Help

- **Documentation**: Check the `docs/` folder for more detailed information
- **Issues**: Report bugs on [GitHub Issues](https://github.com/your-org/leaf-rs/issues)
- **Discussions**: Ask questions in [GitHub Discussions](https://github.com/your-org/leaf-rs/discussions)
