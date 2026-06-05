# Get Up

A Rust CLI that delivers a daily motivational briefing with **LeetCode** and **Deep-ML** problems, on-this-day history with your age, running stats, and an inspirational quote. Posts to **GitHub Issues**, **Telegram DM**, and **Discord DM**.

## Features

- **Multi-platform problems** — LeetCode (Easy/Medium/Hard) + Deep-ML (Easy/Medium/Hard) with scheduled difficulty
- **Difficulty scheduling** — Weekdays: 3 Easy + 2 Medium per platform, Weekends: 1 Medium + 1 Hard per platform
- **Time-aware greeting** — `Good morning` / `Good afternoon` / `Good evening` based on your local hour
- **Your age in history** — Wikipedia on-this-day events tagged with how old you were
- **Running stats** — reads parquet or CSV from Strava / OpenTracks / any app that exports run data
- **Triple notification** — GitHub Issue comment + Telegram DM + Discord DM
- **Morning & night routines** — `--night` for an evening summary variant
- **MCP server** — Model Context Protocol integration with stdio and HTTP transports for AI agent workflows
- **Structured output** — `--json` or `--xml` for machine-readable results
- **GitHub Actions cron** — fires daily via CI, zero-maintenance

## Example Output

```
☀️ Good morning — 2026-05-31 08:13:22

Day 151 / 365 (41.4%) █████████░░░░░░░░░░░

📚 Today's Problems

🟢 LeetCode Easy: 1. Two Sum
https://leetcode.com/problems/two-sum/

🟡 Deep-ML Medium: Matrix-Vector Dot Product
https://deep-ml.com/problems/1

🏃 Yesterday: 5.23 km · This month: 45.6 km · This year: 312.4 km

📜 On this day:
• 2017: Takuma Sato wins the Indy 500 (you were 17)
• 2016: Harambe incident (you were 16)

💬 Today's Quote
The only way to do great work is to love what you do.

—— Steve Jobs
```

## Quick Start

```bash
git clone https://github.com/mozayed007/get-up.git
cd get-up/rust-routine-daily

cp .env.example .env
# fill in: GITHUB_TOKEN, REPO_OWNER, REPO_NAME, BIRTH_YEAR, TIMEZONE

# Fetch LeetCode problems (Easy, Medium, Hard)
cargo run -- --fetch-leetcode

# Sync Deep-ML problems from GitHub
cargo run -- --sync-deepml

# Test with dry run
cargo run --features telegram,discord -- --dry-run
```

## CLI

| Flag | Description |
|------|-------------|
| `--fetch-leetcode` | Download all LeetCode problems (Easy, Medium, Hard) to `data/leetcode_*.txt` |
| `--fetch-easy` | Download only EASY problems (legacy) |
| `--sync-deepml` | Sync Deep-ML problems from GitHub to `data/deepml_problems.txt` |
| `--dry-run` | Print message to terminal, don't post anywhere |
| `--post` | Post the message as a comment on GitHub Issue #1 |
| `--telegram` | Send via Telegram DM |
| `--discord` | Send via Discord DM |
| `--json` | Output structured JSON instead of formatted text |
| `--xml` | Output structured XML instead of formatted text |
| `--night` | Run the night routine variant |

**Subcommand:**

| Command | Description |
|---------|-------------|
| `mcp` | Start the MCP server (`--transport stdio\|http`, `--port 3000`) |

## Agent Experience (AX)

Get Up is built for AI agents as much as humans. Every component returns structured data, the routine engine is idempotent, and the MCP server gives agents direct access to each section independently.

### How Agents Connect

Two transport modes via the [Model Context Protocol](https://modelcontextprotocol.io/):

```bash
# stdio — for local agents (Cursor, Claude Code, opencode, etc.)
cargo run --release --features mcp -- mcp

# HTTP — for remote agents or multi-client setups
cargo run --release --features mcp -- mcp --transport http --port 3000
```

In your agent's MCP config:

```json
{
  "mcpServers": {
    "get-up": {
      "command": "cargo",
      "args": ["run", "--release", "--features", "mcp", "--", "mcp"],
      "cwd": "/path/to/get-up/rust-routine-daily"
    }
  }
}
```

For HTTP transport, point your agent at `http://localhost:3000/mcp`.

### Available Tools

The server exposes six tools. Each returns JSON by default.

| Tool | Parameters | Returns |
|------|-----------|---------|
| `run_routine` | `routine_type`, `include_*` toggles, `format` | Full `RoutineResult` with all requested sections |
| `get_problems` | — | List of problems from LeetCode and Deep-ML with platform, difficulty, URL, daily challenge flag |
| `get_quote` | — | Quote text, author, source |
| `get_history` | — | List of events with year, text, Wikipedia URL, age context |
| `get_running_stats` | — | Yesterday / month / year distances and session counts |
| `get_year_progress` | — | Day of year, total days, percentage, visual bar |

### `run_routine` Parameters

The main tool gives agents full control over what they receive:

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `routine_type` | `"morning"` / `"night"` | `"morning"` | Which routine variant to run |
| `include_problems` | bool | `true` | Include daily problems (LeetCode + Deep-ML) |
| `include_running` | bool | `true` | Include running stats |
| `include_history` | bool | `true` | Include on-this-day events |
| `include_quote` | bool | `true` | Include motivational quote |
| `include_year_progress` | bool | `true` | Include year progress bar |
| `format` | `"json"` / `"xml"` / `"text"` | `"json"` | Output format |

Example: get only problems and quote in JSON:

```json
{
  "include_problems": true,
  "include_running": false,
  "include_history": false,
  "include_quote": true,
  "include_year_progress": false,
  "format": "json"
}
```

### Example Agent Workflows

**Morning briefing agent** — call `run_routine` with defaults, summarize the result in natural language, and remind the user of their problems.

**Running coach agent** — call `get_running_stats` to check recent mileage, then suggest today's distance based on weekly trends.

**History curator agent** — call `get_history` to get today's events, pick the most interesting one, and write a short story about it.

**Progress tracker agent** — call `get_year_progress` to check how far into the year we are, then compare against goals.

### Design for Agents

- **Idempotent** — calling any tool multiple times with the same config returns the same result. No side effects like posting or marking problems as used.
- **Composable** — each section is independent. Agents can mix and match what they need.
- **Structured** — all tools return JSON with consistent field names. No parsing prose required.
- **Stateless** — the server holds no per-request state beyond the shared config. Safe to call from concurrent agents.

### CLI for Agent Pipelines

For agents that prefer subprocess over MCP, use the CLI with structured output:

```bash
# JSON output for parsing
cargo run --release -- --json --dry-run

# XML output
cargo run --release -- --xml --dry-run

# Night routine
cargo run --release -- --night --json --dry-run
```

The `RoutineResult` JSON schema includes: `routine_type`, `greeting`, `timestamp`, `year_progress`, `problems`, `running`, `history`, `quote`, and `formatted_message`.

## Running Stats

Place a file at `data/running.parquet` (or `.csv`) with two columns:

| Column | Example |
|--------|---------|
| `date` | `2026-05-27` |
| `distance_km` | `5.23` |

**On Android** — install [OpenTracks](https://f-droid.org/packages/de.dennisguse.opentracks/) (free, open-source) or any app that exports CSV, and drop the file in `data/`.

## Environment Variables

```bash
GITHUB_TOKEN          # GitHub PAT (repo scope)
REPO_OWNER            # GitHub username or org
REPO_NAME             # Repository name
BIRTH_YEAR            # Your birth year (for age-tagged history)
TIMEZONE              # IANA timezone (e.g. Africa/Cairo)
LEETCODE_ENDPOINT     # https://leetcode.com/graphql/ (default)
LEETCODE_VARIANT      # com (default) or cn
TELEGRAM_TOKEN        # Telegram bot token (optional)
TELEGRAM_CHAT_ID      # Telegram chat ID (optional)
DISCORD_TOKEN         # Discord bot token (optional)
DISCORD_USER_ID       # Discord user ID (optional)
```

## GitHub Actions

Three workflows are included:

- **CI** — `cargo test --all-features` + `clippy` + `fmt` on every push
- **Daily Run** — fires at midnight UTC (3 AM Cairo), posts to GitHub Issue + Telegram + Discord
- **Deep-ML Sync** — weekly sync of Deep-ML problems from GitHub (Sundays)

Set these repo secrets:

```
GITHUB_TOKEN        (auto-injected)
TELEGRAM_TOKEN
TELEGRAM_CHAT_ID
DISCORD_TOKEN
DISCORD_USER_ID
BIRTH_YEAR
TIMEZONE            (e.g. Africa/Cairo)
```

## Project Layout

```
get-up/
├── .github/workflows/
│   ├── ci.yml               # lint + test on push
│   ├── daily.yml            # daily cron job
│   ├── sync-deepml.yml      # weekly Deep-ML sync
│   └── cleanup-caches.yml   # weekly cache cleanup
├── rust-routine-daily/
│   ├── src/
│   │   ├── main.rs          # CLI entry, orchestration
│   │   ├── config.rs        # env var loading
│   │   ├── providers/       # Problem providers
│   │   │   ├── mod.rs       # Shared selection logic
│   │   │   ├── leetcode.rs  # LeetCode provider
│   │   │   └── deepml.rs    # Deep-ML provider
│   │   ├── scheduler.rs     # Difficulty scheduling
│   │   ├── format.rs        # Message formatting
│   │   ├── serialization.rs # JSON/XML serialization
│   │   ├── api.rs           # quotes, history, running stats
│   │   ├── types.rs         # Shared types
│   │   ├── message.rs       # Greeting helper
│   │   ├── routine.rs       # Routine engine
│   │   ├── utils.rs         # Time helpers
│   │   ├── notification/    # Notification adapters
│   │   └── mcp/             # MCP server
│   ├── data/                # problem lists, used problems, running stats
│   └── Cargo.toml
├── docs/                      # Documentation
└── README.md
```

## Difficulty Scheduling

The system automatically schedules difficulty based on the day:

| Period | LeetCode | Deep-ML |
|--------|----------|---------|
| **Weekdays** (Mon-Fri) | 3 Easy + 2 Medium | 3 Easy + 2 Medium |
| **Weekends** (Sat-Sun) | 1 Medium + 1 Hard | 1 Medium + 1 Hard |

Each platform independently rolls its difficulty. LeetCode could be Easy while Deep-ML is Medium on the same day.

## Built With

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client |
| `tokio` | Async runtime |
| `chrono` / `chrono-tz` | Date/time and timezone handling |
| `polars` | Parquet/CSV data processing for running stats |
| `octocrab` | GitHub API (Issue comments) |
| `teloxide` | Telegram Bot API |
| `serenity` | Discord Bot API |
| `clap` | CLI argument parsing |
| `rmcp` | MCP server (Model Context Protocol) |
| `axum` | HTTP server for MCP streamable transport |
| `serde` / `serde_json` | JSON serialization |
| `anyhow` | Error handling |
| `rand` | Seeded random problem selection |

## License

MIT
