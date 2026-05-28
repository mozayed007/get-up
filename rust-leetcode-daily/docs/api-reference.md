# API Reference

This document provides comprehensive reference documentation for LeetCode Daily's CLI, modules, structs, and traits.

## CLI Arguments

### Synopsis

```
leetcode-daily [OPTIONS]
```

### Options

| Option | Type | Description |
|--------|------|-------------|
| `--fetch-easy` | Flag | Fetch and save all EASY problems to `data/leetcode_easy.txt` |
| `--post` | Flag | Post the generated message to GitHub Issue #1 |
| `--telegram` | Flag | Send notification via Telegram |
| `--discord` | Flag | Send notification via Discord |
| `--dry-run` | Flag | Print message to stdout without posting |
| `-h, --help` | Flag | Display help information |
| `-V, --version` | Flag | Display version information |

### Usage Examples

```bash
# Fetch EASY problems from LeetCode
leetcode-daily --fetch-easy

# Preview message without sending
leetcode-daily --dry-run

# Post to GitHub Issue
leetcode-daily --post

# Post to all configured channels
leetcode-daily --post --telegram --discord

# Combined workflow
leetcode-daily --post --telegram --discord --dry-run  # dry-run overrides
```

### Argument Interactions

```
┌────────────────────────────────────────────────────────────────────┐
│ Argument Precedence (when combined)                                 │
├────────────────────────────────────────────────────────────────────┤
│ --dry-run     │ Overrides --post, --telegram, --discord            │
│               │ Prints to stdout instead of sending                │
├───────────────┼────────────────────────────────────────────────────┤
│ --fetch-easy  │ Exclusive operation                                │
│               │ Exits after fetching, ignores other flags          │
└───────────────┴────────────────────────────────────────────────────┘
```

## Exit Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 0 | Success | Operation completed successfully |
| 1 | General Error | Configuration error, API failure, or runtime error |
| 2 | Config Error | Missing required environment variable |

### Exit Code Examples

```bash
# Check exit code (bash)
leetcode-daily --dry-run
echo $?  # 0 for success

# Use in scripts
if leetcode-daily --post; then
    echo "Posted successfully"
else
    echo "Failed to post"
fi
```

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
    pub birth_year: i32,
    pub timezone: Tz,
    pub leetcode_endpoint: String,
    pub leetcode_variant: LeetCodeVariant,
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `github_token` | `String` | Yes | GitHub Personal Access Token |
| `repo_owner` | `String` | Yes | Repository owner username |
| `repo_name` | `String` | Yes | Repository name |
| `telegram_token` | `Option<String>` | No | Telegram bot token |
| `telegram_chat_id` | `Option<String>` | No | Telegram chat ID |
| `discord_token` | `Option<String>` | No | Discord bot token |
| `discord_channel_id` | `Option<String>` | No | Discord channel ID |
| `birth_year` | `i32` | Yes | Birth year for age calculations |
| `timezone` | `Tz` | Yes | IANA timezone identifier |
| `leetcode_endpoint` | `String` | No | LeetCode GraphQL endpoint |
| `leetcode_variant` | `LeetCodeVariant` | No | Platform variant (Com/Cn) |

#### `LeetCodeVariant` Enum

```rust
pub enum LeetCodeVariant {
    Com,  // leetcode.com
    Cn,   // leetcode.cn
}
```

---

### Module: `leetcode`

**Path**: `src/leetcode.rs`

LeetCode API interaction and problem management.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `LeetCode` | Struct | LeetCode API client |
| `Question` | Struct | Problem representation |

#### `LeetCode` Struct

```rust
pub struct LeetCode {
    client: Client,
    endpoint: String,
    variant: LeetCodeVariant,
}
```

##### Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `new` | `fn new(config: &Config) -> Self` | Create new LeetCode client |
| `fetch_easy_list` | `async fn fetch_easy_list(&self, output_file: &str) -> Result<()>` | Fetch all EASY problems |
| `get_daily_challenge` | `async fn get_daily_challenge(&self) -> Result<Option<Question>>` | Get official daily challenge |
| `get_today_problem` | `async fn get_today_problem(&self, easy_file: &str, used_file: &str) -> Result<Question>` | Get today's problem |
| `pick_daily_problem` | `async fn pick_daily_problem(&self, easy_file: &str, used_file: &str) -> Result<Question>` | Pick random unused problem |

#### `Question` Struct

```rust
pub struct Question {
    pub id: String,
    pub title: String,
    pub slug: String,
    pub difficulty: String,
    pub paid_only: bool,
    pub is_daily_challenge: bool,
}
```

| Field | Type | Description |
|-------|------|-------------|
| `id` | `String` | Frontend question ID (e.g., "1") |
| `title` | `String` | Problem title (e.g., "Two Sum") |
| `slug` | `String` | URL slug (e.g., "two-sum") |
| `difficulty` | `String` | Difficulty level |
| `paid_only` | `bool` | Whether it's a premium problem |
| `is_daily_challenge` | `bool` | Whether it's today's daily challenge |

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

| Field | Type | Description |
|-------|------|-------------|
| `yesterday_km` | `f64` | Total km run yesterday |
| `yesterday_count` | `i32` | Number of sessions yesterday |
| `month_km` | `f64` | Total km this month |
| `month_count` | `i32` | Number of sessions this month |
| `year_km` | `f64` | Total km this year |
| `year_count` | `i32` | Number of sessions this year |

#### Function Signatures

```rust
pub async fn fetch_quote(client: &reqwest::Client) -> Result<String>

pub async fn fetch_history(
    client: &reqwest::Client,
    birth_year: i32,
    month: u32,
    day: u32,
) -> Result<Vec<String>>

pub async fn fetch_running_stats(parquet_file: &str) -> Result<RunningStats>
```

---

### Module: `message`

**Path**: `src/message.rs`

Message formatting and building.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `build_message` | Function | Build complete daily message |
| `format_leetcode` | Function | Format LeetCode problem |
| `format_running` | Function | Format running statistics |
| `format_history` | Function | Format historical events |

#### Function Signatures

```rust
pub fn build_message(
    get_up_time: &str,
    day_of_year: u32,
    year_progress: &str,
    leetcode: &str,
    running_info: &str,
    history_today: &str,
    quote: &str,
    _config: &Config,
) -> String

pub fn format_leetcode(problem: &Question, variant: &LeetCodeVariant) -> String

pub fn format_running(stats: &RunningStats) -> String

pub fn format_history(history: &[String]) -> String
```

---

### Module: `utils`

**Path**: `src/utils.rs`

Utility functions for date/time and file operations.

#### Exports

| Item | Type | Description |
|------|------|-------------|
| `get_day_of_year` | Function | Calculate day of year |
| `get_year_progress` | Function | Calculate year progress with bar |
| `get_local_time` | Function | Get current time in configured timezone |
| `read_lines` | Function | Read non-empty lines from file |
| `append_line` | Function | Append line to file |
| `pick_random` | Function | Pick random element from slice |

#### Function Signatures

```rust
pub fn get_day_of_year(now: &DateTime<Tz>) -> u32

pub fn get_year_progress(now: &DateTime<Tz>) -> String

pub fn get_local_time(config: &Config) -> DateTime<Tz>

pub async fn read_lines(filename: &str) -> Result<Vec<String>>

pub async fn append_line(filename: &str, line: &str) -> Result<()>

pub fn pick_random<T: Clone>(items: &[T]) -> Option<T>
```

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

**Path**: `src/notification/mod.rs`

Async trait for sending notifications.

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

### `TelegramNotifier`

```rust
pub struct TelegramNotifier {
    bot: teloxide::Bot,
    chat_id: teloxide::types::ChatId,
}

impl TelegramNotifier {
    pub fn new(bot_token: String, chat_id: String) -> Self
}

#[async_trait]
impl Notifier for TelegramNotifier {
    async fn send_message(&self, message: &str) -> Result<()>
}
```

### `DiscordNotifier`

```rust
pub struct DiscordNotifier {
    http: serenity::http::Http,
    channel_id: serenity::model::id::ChannelId,
}

impl DiscordNotifier {
    pub fn new(token: String, channel_id: String) -> Self
}

#[async_trait]
impl Notifier for DiscordNotifier {
    async fn send_message(&self, message: &str) -> Result<()>
}
```

## Feature Flags

| Feature | Dependencies | Description |
|---------|--------------|-------------|
| `telegram` | `teloxide` | Enable Telegram notifications |
| `discord` | `serenity` | Enable Discord notifications |
| (default) | none | GitHub-only mode |

### Building with Features

```bash
# Default (no notifications)
cargo build --release

# Telegram only
cargo build --release --features telegram

# Discord only
cargo build --release --features discord

# All features
cargo build --release --features telegram,discord
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
| Problem selection | `"No available EASY problems found"` |
| Feature not enabled | `"Telegram feature not enabled. Recompile with --features telegram"` |

## Related Documentation

- **Architecture**: See [Architecture Overview](architecture.md)
- **Configuration**: See [Configuration Guide](configuration.md)
- **Development**: See [Development Guide](development.md)
