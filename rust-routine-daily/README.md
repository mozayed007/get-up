# get-up

A Rust CLI tool and MCP server for daily motivational messages with LeetCode problems. Built for both humans (UX) and agents (AX).

## Features

**For Humans (UX)**
- Fetches daily LeetCode EASY problems
- Generates motivational messages with quotes, history events, running stats
- Posts to GitHub Issues
- Telegram and Discord notification support (feature flags)
- Year progress bar visualization

**For Agents (AX)**
- MCP server (stdio + HTTP/SSE transports) for agent integration
- Structured JSON and XML output formats
- Composable tools: call individual pieces or the full routine
- Customizable routine: agents choose which sections to include
- Morning and night routine variants

## Quick Start

```bash
# Build
cargo build --release

# Preview message
routine-daily --dry-run

# JSON output for agents
routine-daily --json

# XML output
routine-daily --xml

# Night routine
routine-daily --night --json
```

## MCP Server

The MCP server lets AI agents (Claude Code, OpenCode, Cursor, Pi, Codex, Kilo CLI, etc.) use get-up as a background engine.

### Build with MCP support

```bash
cargo build --release --features mcp
```

### Start the MCP server

```bash
# stdio mode (for Claude Code, OpenCode, Cursor, etc.)
routine-daily mcp --transport stdio

# HTTP/SSE mode (for remote agents like Pi)
routine-daily mcp --transport http --port 3000
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `run_routine` | Full morning/night routine with customizable sections |
| `get_leetcode_problem` | Today's LeetCode problem |
| `get_quote` | Motivational quote |
| `get_history` | On-this-day historical events |
| `get_running_stats` | Running statistics |
| `get_year_progress` | Year progress bar |

### Agent Configuration Examples

**Claude Code / OpenCode (stdio)**
```json
{
  "mcpServers": {
    "get-up": {
      "command": "routine-daily",
      "args": ["mcp", "--transport", "stdio"]
    }
  }
}
```

**Remote agents (HTTP/SSE)**
```bash
# Start server
routine-daily mcp --transport http --port 3000

# Agents connect to http://localhost:3000/mcp
```

### Customizing the Routine

Agents can pass parameters to `run_routine`:

```json
{
  "routine_type": "night",
  "include_leetcode": true,
  "include_running": false,
  "include_history": true,
  "include_quote": true,
  "include_year_progress": false,
  "format": "json"
}
```

## CLI Reference

```
USAGE:
    routine-daily [OPTIONS] [SUBCOMMAND]

OPTIONS:
    --fetch-easy    Fetch all EASY problems from LeetCode
    --dry-run       Print message without posting
    --post          Post to GitHub Issue #1
    --telegram      Send via Telegram
    --discord       Send via Discord
    --json          Output structured JSON
    --xml           Output structured XML
    --night         Run night routine instead of morning
    -h, --help      Print help
    -V, --version   Print version

SUBCOMMANDS:
    mcp             Start MCP server for agent integration
```

## Example Output

### Text format (default)
```
☀️ Good morning — 2026-02-16 08:30:00

Day 47 · ████████░░░░░░░░░░░░

🟢 LeetCode EASY: 1. Two Sum
https://leetcode.com/problems/two-sum/

🏃 Yesterday: 5.23 km · This month: 45.67 km · This year: 123.45 km

📜 On this day:
• 2020: [Some historical event](https://en.wikipedia.org/wiki/...) (you were 30)

💬 Today's Quote
The only way to do great work is to love what you do.

—— Steve Jobs
```

### JSON format (`--json`)
```json
{
  "routine_type": "morning",
  "greeting": "☀️ Good morning",
  "timestamp": "2026-02-16 08:30:00",
  "year_progress": {
    "day_of_year": 47,
    "total_days": 365,
    "percentage": 12.88,
    "bar": "████████░░░░░░░░░░░░"
  },
  "leetcode": {
    "question": {
      "id": "1",
      "title": "Two Sum",
      "slug": "two-sum",
      "difficulty": "EASY",
      "is_daily_challenge": false
    },
    "url": "https://leetcode.com/problems/two-sum/",
    "is_daily_challenge": false
  },
  "running": {
    "yesterday_km": 5.23,
    "yesterday_count": 1,
    "month_km": 45.67,
    "month_count": 8,
    "year_km": 123.45,
    "year_count": 22
  },
  "history": [
    {
      "year": 2020,
      "text": "Some historical event",
      "url": "https://en.wikipedia.org/wiki/...",
      "age_context": "(you were 30)"
    }
  ],
  "quote": {
    "text": "The only way to do great work is to love what you do.",
    "author": "Steve Jobs",
    "source": "api"
  },
  "formatted_message": "..."
}
```

## Project Structure

```
rust-leetcode-daily/
├── Cargo.toml
├── src/
│   ├── lib.rs              # Library crate root
│   ├── main.rs             # CLI entry point
│   ├── config.rs           # Configuration from env vars
│   ├── utils.rs            # Date helpers, file I/O
│   ├── leetcode.rs         # LeetCode API client
│   ├── api.rs              # Quote, history, running stats
│   ├── message.rs          # Text formatting
│   ├── routine.rs          # Routine orchestration + structured types
│   ├── mcp/
│   │   └── mod.rs          # MCP server (feature-gated)
│   └── notification/
│       ├── mod.rs           # Notifier trait
│       ├── telegram.rs      # Telegram adapter
│       └── discord.rs       # Discord adapter
└── data/
    ├── leetcode_easy.txt    # EASY problems list
    └── used_problems.txt    # Previously used problems
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `telegram` | No | Telegram bot notifications |
| `discord` | No | Discord notifications |
| `mcp` | No | MCP server (adds rmcp, axum, schemars) |

## Building

```bash
# Minimal (CLI only)
cargo build --release

# With notifications
cargo build --release --features telegram,discord

# With MCP server
cargo build --release --features mcp

# Everything
cargo build --release --features telegram,discord,mcp
```

## Environment Variables

See `.env.example` for all supported variables.

| Variable | Required | Description |
|----------|----------|-------------|
| `GITHUB_TOKEN` | Yes | GitHub Personal Access Token |
| `REPO_OWNER` | Yes | GitHub repo owner |
| `REPO_NAME` | Yes | GitHub repo name |
| `BIRTH_YEAR` | Yes | Birth year for age context |
| `TIMEZONE` | Yes | Timezone (e.g., `Africa/Cairo`) |
| `TELEGRAM_TOKEN` | No | Telegram bot token |
| `TELEGRAM_CHAT_ID` | No | Telegram chat ID |
| `DISCORD_TOKEN` | No | Discord bot token |
| `DISCORD_CHANNEL_ID` | No | Discord channel ID |
| `DISCORD_USER_ID` | No | Discord user ID for DMs |
| `LEETCODE_ENDPOINT` | No | LeetCode GraphQL endpoint |
| `LEETCODE_VARIANT` | No | `com` or `cn` |

## License

MIT
