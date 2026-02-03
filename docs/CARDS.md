# LEAF Cards Guide

Cards are the core automation units in LEAF. Each card represents a single automation that responds to triggers and executes Python code.

## What is a Card?

A Card consists of:

1. **Trigger** - What starts the automation (file change, schedule, manual)
2. **Program** - Python code that runs when triggered
3. **Configuration** - Settings like timeout, retries, and dependencies

```
┌─────────────────────────────────────────┐
│                  Card                    │
├─────────────────────────────────────────┤
│  Trigger: file_created in inbox/*.csv   │
│  Program: .leaf/programs/csv-proc-xyz/  │
│  Timeout: 300 seconds                   │
│  Retries: 3                             │
│  Dependencies: [pandas, numpy]          │
└─────────────────────────────────────────┘
```

## Creating Cards

### Via the AI Agent

The easiest way to create cards is through natural language:

```bash
curl -X POST http://127.0.0.1:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"content": "Create an automation that processes CSV files in inbox and generates a summary report in reports/"}'
```

The agent will:
1. Understand your requirements
2. Propose a card with appropriate triggers
3. Generate the Python code
4. Create the card upon confirmation

### Via the API

```bash
curl -X POST http://127.0.0.1:8000/api/cards \
  -H "Content-Type: application/json" \
  -d '{
    "name": "CSV Processor",
    "description": "Processes CSV files and generates reports",
    "user_prompt": "Process CSV files",
    "trigger_config": {
      "type": "file_created",
      "folder": "inbox",
      "pattern": "*.csv"
    },
    "program_config": {
      "language": "python",
      "entrypoint": "main.py",
      "dependencies": ["pandas"]
    }
  }'
```

## Trigger Types

### file_created

Triggers when a new file is created in a watched folder.

```json
{
  "type": "file_created",
  "folder": "inbox",
  "pattern": "*.csv"
}
```

**Pattern Syntax:**
- `*` - Match any characters
- `*.csv` - Match files ending in .csv
- `report_*.xlsx` - Match files starting with "report_" and ending in .xlsx

### file_modified

Triggers when an existing file is modified.

```json
{
  "type": "file_modified",
  "folder": "data",
  "pattern": "config.json"
}
```

### manual

Triggers only when explicitly called via API.

```json
{
  "type": "manual"
}
```

Trigger manually:
```bash
curl -X POST http://127.0.0.1:8000/api/cards/{card_id}/trigger
```

### schedule (Future)

Triggers on a cron schedule.

```json
{
  "type": "schedule",
  "cron": "0 9 * * *"
}
```

## Card Program Structure

Each card's program lives in `.leaf/programs/{slug}/`:

```
.leaf/programs/csv-processor-abc123/
├── main.py          # Entry point (required)
├── pyproject.toml   # Dependencies (auto-generated)
└── .venv/           # Virtual environment (auto-created)
```

### main.py

The entry point for your automation:

```python
"""CSV Processor Card

Processes CSV files from inbox and generates reports.
"""

import sys
from pathlib import Path


def main():
    """Main entry point."""
    # Get the trigger file path from command line args
    if len(sys.argv) > 1:
        trigger_file = Path(sys.argv[1])
        print(f"Processing: {trigger_file}")

        # Your automation logic here
        process_csv(trigger_file)
    else:
        print("No trigger file provided")


def process_csv(file_path: Path):
    """Process a CSV file."""
    import pandas as pd

    # Read the CSV
    df = pd.read_csv(file_path)

    # Generate summary
    summary = df.describe()

    # Save report
    report_path = Path("reports") / f"{file_path.stem}_report.csv"
    report_path.parent.mkdir(exist_ok=True)
    summary.to_csv(report_path)

    print(f"Report saved: {report_path}")


if __name__ == "__main__":
    main()
```

## Environment Variables

Card programs have access to these environment variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `LEAF_PROJECT_ROOT` | Project root directory | `/path/to/project` |
| `LEAF_CARD_ID` | Current card's ID | `card_abc123` |

```python
import os

project_root = os.environ.get("LEAF_PROJECT_ROOT")
card_id = os.environ.get("LEAF_CARD_ID")
```

## Command Line Arguments

When a card is triggered by a file event, the trigger file path is passed as the first argument:

```python
import sys
from pathlib import Path

if len(sys.argv) > 1:
    trigger_file = Path(sys.argv[1])
    # Process the file
```

## Dependencies

Specify Python packages in the card's program config:

```json
{
  "program_config": {
    "dependencies": ["pandas", "numpy", "requests"]
  }
}
```

Dependencies are automatically installed in the card's isolated virtual environment.

### Adding Dependencies After Creation

Edit the card's `pyproject.toml`:

```toml
[project]
name = "card-csv-processor-abc123"
version = "0.1.0"
requires-python = ">=3.11"
dependencies = [
    "pandas",
    "numpy",
    "openpyxl"  # Added
]
```

The environment will sync on next execution.

## Execution

### How Execution Works

1. **Trigger fires** - File created, schedule, or manual
2. **Event created** - Recorded in database
3. **Cards matched** - Find cards matching the trigger
4. **Sandbox setup** - Create/sync UV environment
5. **Program runs** - Execute main.py with args
6. **Results stored** - Capture stdout, stderr, exit code
7. **Retries** - If failed, retry up to configured count

### Execution Settings

```json
{
  "timeout_seconds": 300,  // Max runtime (default: 5 min)
  "retry_count": 3         // Retry attempts (default: 3)
}
```

### Viewing Executions

```bash
# List recent executions
curl http://127.0.0.1:8000/api/executions

# Get specific execution
curl http://127.0.0.1:8000/api/executions/{execution_id}

# Filter by card
curl http://127.0.0.1:8000/api/executions?card_id=card_abc123

# Filter by status
curl http://127.0.0.1:8000/api/executions?status=failed
```

### Execution Output

Programs can output:
- **stdout** - Captured and stored
- **stderr** - Captured and stored
- **Files** - Write anywhere in project (except .leaf/)

```python
# These are captured
print("Processing started")
print("Rows processed: 1000")

# Errors go to stderr
import sys
print("Warning: missing values", file=sys.stderr)
```

## Best Practices

### 1. Handle Missing Files Gracefully

```python
from pathlib import Path

def main():
    if len(sys.argv) < 2:
        print("No trigger file provided")
        return

    trigger_file = Path(sys.argv[1])
    if not trigger_file.exists():
        print(f"File not found: {trigger_file}")
        return

    process_file(trigger_file)
```

### 2. Use Relative Paths

```python
import os
from pathlib import Path

# Get project root
project_root = Path(os.environ.get("LEAF_PROJECT_ROOT", "."))

# Use relative paths from project root
output_dir = project_root / "reports"
output_dir.mkdir(exist_ok=True)
```

### 3. Log Progress

```python
def process_large_file(file_path):
    print(f"Starting: {file_path}")

    with open(file_path) as f:
        for i, line in enumerate(f):
            if i % 1000 == 0:
                print(f"Processed {i} lines...")
            process_line(line)

    print(f"Complete: {i + 1} lines processed")
```

### 4. Handle Errors

```python
def main():
    try:
        process_file(trigger_file)
    except FileNotFoundError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        sys.exit(1)
    except Exception as e:
        print(f"UNEXPECTED ERROR: {e}", file=sys.stderr)
        sys.exit(1)
```

### 5. Use MCP Tools

```python
from leaf.mcp.card_helper import MCPClient

def main():
    mcp = MCPClient()

    # Fetch web content
    result = mcp.call("fetch", "fetch", url="https://api.example.com/data")
    if result["success"]:
        data = result["output"]
        # Process data
```

## Card Lifecycle

### Creating

```bash
POST /api/cards
```

- Creates database record
- Creates program directory
- Creates placeholder main.py
- Sets up file watcher (if file trigger)

### Enabling/Disabling

```bash
POST /api/cards/{id}/enable
POST /api/cards/{id}/disable
```

- Enables/disables the card
- Adds/removes file watcher

### Updating

```bash
PATCH /api/cards/{id}
```

- Updates configuration
- Recreates watcher if trigger changed

### Deleting

```bash
DELETE /api/cards/{id}
```

- Removes database record
- Removes file watcher
- **Does not delete program files** (for safety)

## Troubleshooting

### Card Not Triggering

1. Check card is enabled:
   ```bash
   curl http://127.0.0.1:8000/api/cards/{id}
   # Look for "enabled": true
   ```

2. Check file watcher is active:
   ```bash
   curl http://127.0.0.1:8000/api/debug/watches
   ```

3. Verify trigger pattern matches your file

### Execution Failing

1. Check execution details:
   ```bash
   curl http://127.0.0.1:8000/api/executions?card_id={id}&status=failed
   ```

2. Look at stderr output for errors

3. Test program manually:
   ```bash
   cd /path/to/project/.leaf/programs/your-card/
   uv run python main.py /path/to/test/file.csv
   ```

### Dependencies Not Installing

1. Check pyproject.toml syntax
2. Try syncing manually:
   ```bash
   cd /path/to/project/.leaf/programs/your-card/
   uv sync
   ```
