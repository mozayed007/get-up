# API Reference

This document provides comprehensive reference documentation for the daily routine CLI, modules, structs, and traits.

## CLI Arguments

### Synopsis

```
routine-daily [OPTIONS]
```

### Options

| Option | Type | Description |
|--------|------|-------------|
| `--fetch-leetcode` | Flag | Fetch all LeetCode problems (Easy, Medium, Hard) to `data/leetcode_*.txt` |
| `--fetch-easy` | Flag | Fetch only EASY problems (legacy) |
| `--sync-deepml` | Flag | Sync Deep-ML problems from GitHub to `data/deepml_problems.txt` |
| `--post` | Flag | Post the generated message to GitHub Issue #1 |
| `--telegram` | Flag | Send notification via Telegram |
| `--discord` | Flag | Send notification via Discord |
| `--dry-run` | Flag | Print message to stdout without posting |
| `--json` | Flag | Output structured JSON |
| `--xml` | Flag | Output structured XML |
| `--night` | Flag | Run the night routine variant |
| `-h, --help` | Flag | Display help information |
| `-V, --version` | Flag | Display version information |

### Usage Examples

```bash
# Fetch all LeetCode difficulties
routine-daily --fetch-leetcode

# Sync Deep-ML problems
routine-daily --sync-deepml

# Preview message without sending
routine-daily --dry-run

# Post to GitHub Issue
routine-daily --post

# Post to all configured channels
routine-daily --post --telegram --discord

# JSON output for agent pipelines
routine-daily --json --dry-run
```

### Argument Interactions

| Argument | Behavior |
|----------|----------|
| `--dry-run` | Overrides `--post`, `--telegram`, `--discord`. Prints to stdout instead of sending |
| `--fetch-leetcode` | Exclusive operation. Exits after fetching, ignores other flags |
| `--sync-deepml` | Exclusive operation. Exits after syncing, ignores other flags |
| `--fetch-easy` | Exclusive operation. Exits after fetching, ignores other flags |

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 0 | Success | Operation completed successfully |
| 1 | General Error | Configuration error, API failure, or runtime error |
| 2 | Config Error | Missing required environment variable |

## Module Documentation

### Module: `config`

**Path**: `src/config.rs`

Configuration loading and management.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `Config` | Struct | Main configuration container |
| `LeetCodeVariant` | Enum | LeetCode platform variant |

#### `Config` Struct

```rust
pub struct Config {
    pub github_token: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub telegram_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub discord_token: Option<String>,
    pub discord_channel_id: Option<String>,
    pub discord_user_id: Option<String>,
    pub birth_year: i32,
    pub timezone: Tz,
    pub leetcode_endpoint: String,
    pub leetcode_variant: LeetCodeVariant,
}
```

#### `LeetCodeVariant` Enum

```rust
pub enum LeetCodeVariant {
    Com,  // leetcode.com
    Cn,   // leetcode.cn
}
```

---

### Module: `providers`

**Path**: `src/providers/`

Problem provider abstraction with shared selection logic.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `select_problem` | Function | Shared problem selection logic |
| `LeetCodeProvider` | Struct | LeetCode API client |
| `DeepMLProvider` | Struct | Deep-ML GitHub sync client |

#### `LeetCodeProvider` Methods

| Method | Description |
|--------|-------------|
| `new` | Create new provider from config |
| `fetch_easy_list` | Fetch EASY problems |
| `fetch_medium_list` | Fetch MEDIUM problems |
| `fetch_hard_list` | Fetch HARD problems |
| `get_problem` | Get today's problem with daily challenge priority |

#### `DeepMLProvider` Methods

| Method | Description |
|--------|-------------|
| `new` | Create new provider |
| `sync_problems` | Sync all problems from GitHub repo |
| `get_problem` | Get today's problem from cache |

#### `Problem` Type

```rust
pub struct Problem {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub difficulty: Difficulty,
    pub is_daily_challenge: bool,
}
```

---

### Module: `scheduler`

**Path**: `src/scheduler.rs`

Difficulty scheduling for daily problems.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `Schedule` | Enum | Day schedule (Weekday or Weekend) |
| `get_schedule` | Function | Get schedule for platform and date |

#### `Schedule` Enum

```rust
pub enum Schedule {
    Weekday { difficulty: Difficulty },
    Weekend { difficulties: [Difficulty; 2] },
}
```

---

### Module: `api`

**Path**: `src/api.rs`

External API interactions (quotes, history, running stats).

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `fetch_quote` | Function | Fetch random motivational quote |
| `fetch_history` | Function | Fetch historical events for today |
| `fetch_running_stats` | Function | Read running statistics from parquet |
| `RunningStats` | Struct | Running statistics container |

#### `RunningStats` Struct

```rust
pub struct RunningStats {
    pub yesterday_km: f64,
    pub yesterday_count: i32,
    pub month_km: f64,
    pub month_count: i32,
    pub year_km: f64,
    pub year_count: i32,
}
```

---

### Module: `format`

**Path**: `src/format.rs`

Message formatting and problem display.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `build_formatted_message` | Function | Build complete daily message |
| `get_problem_emoji` | Function | Get emoji for platform + difficulty |

---

### Module: `routine`

**Path**: `src/routine.rs`

Routine engine with idempotent execution.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `RoutineOptions` | Struct | Routine configuration |
| `RoutineResult` | Struct | Complete routine output |
| `Section` | Enum | Available sections (Problems, Running, etc.) |
| `run_routine` | Function | Main routine entry point |

#### `RoutineOptions` Struct

```rust
pub struct RoutineOptions {
    pub routine_type: RoutineType,
    pub sections: Vec<Section>,
    pub format: OutputFormat,
}
```

#### `Section` Enum

```rust
pub enum Section {
    Problems,
    Running,
    History,
    Quote,
    YearProgress,
}
```

---

### Module: `types`

**Path**: `src/types.rs`

Shared types used across the codebase.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `Difficulty` | Enum | Easy, Medium, Hard |
| `Platform` | Enum | LeetCode, DeepML |
| `ProblemResult` | Struct | Problem with platform and URL |

---

### Module: `notification`

**Path**: `src/notification/mod.rs`

Notification trait and implementations.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `Notifier` | Trait | Async notification trait |
| `TelegramNotifier` | Struct | Telegram notification implementation |
| `DiscordNotifier` | Struct | Discord notification implementation |

## Trait Reference

### `Notifier` Trait

```rust
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send_message(&self, message: &str) -> Result<()>;
}
```

#### Implementations

| Struct | Feature Flag | Description |
|--------|--------------|-------------|
| `TelegramNotifier` | `telegram` | Sends via Telegram Bot API |
| `DiscordNotifier` | `discord` | Sends via Discord HTTP API |

## Feature Flags

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `telegram` | `teloxide` | Enable Telegram notifications |
| `discord` | `serenity` | Enable Discord notifications |
| `mcp` | `rmcp`, `axum` | Enable MCP server |
| (default) | none | GitHub-only mode |

### Building with Features

```bash
# Default (no notifications)
cargo build --release

# With MCP server
cargo build --release --features mcp

# Telegram + MCP
cargo build --release --features telegram,mcp

# All features
cargo build --release --features telegram,discord,mcp
```

## Error Handling

All fallible operations return `anyhow::Result<T>`. Errors are contextualized with `anyhow::Context`.

### Common Error Types

| Error Source | Context Message |
|--------------|-----------------|
| Missing env var | `"GITHUB_TOKEN must be set"` |
| Invalid timezone | `"Invalid timezone: {tz}"` |
| API failure | `"Failed to fetch quote"` |
| File I/O | `"Failed to open file: {filename}"` |
| Problem selection | `"No available problems found"` |
| Feature not enabled | `"Telegram feature not enabled"` |

## Related Documentation

- **Architecture**: See [Architecture Overview](architecture.md)
- **Configuration**: See [Configuration Guide](configuration.md)
- **Development**: See [Development Guide](development.md)
