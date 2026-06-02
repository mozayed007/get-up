# Architecture Overview

This document describes the architecture, design patterns, and data flow of the daily routine system.

## Project Structure

```
get-up/
├── rust-leetcode-daily/
│   ├── Cargo.toml                 # Dependencies and feature flags
│   ├── .env.example               # Environment template
│   ├── src/
│   │   ├── main.rs               # Entry point, CLI, orchestration
│   │   ├── config.rs             # Configuration loading and types
│   │   ├── providers/            # Problem provider abstraction
│   │   │   ├── mod.rs            # Shared selection logic
│   │   │   ├── leetcode.rs       # LeetCode provider
│   │   │   └── deepml.rs         # Deep-ML provider
│   │   ├── scheduler.rs          # Difficulty scheduling
│   │   ├── format.rs             # Message formatting
│   │   ├── serialization.rs      # JSON/XML serialization
│   │   ├── api.rs                # External API clients
│   │   ├── types.rs              # Shared types
│   │   ├── message.rs            # Greeting helper
│   │   ├── routine.rs            # Routine engine
│   │   ├── utils.rs              # Utilities
│   │   └── notification/
│   │       ├── mod.rs            # Notifier trait
│   │       ├── telegram.rs       # Telegram adapter
│   │       └── discord.rs        # Discord adapter
│   └── data/
│       ├── leetcode_easy.txt     # Cached EASY problems
│       ├── leetcode_medium.txt   # Cached MEDIUM problems
│       ├── leetcode_hard.txt     # Cached HARD problems
│       ├── deepml_problems.txt   # Cached Deep-ML problems
│       ├── used_problems.txt     # Problem usage tracking
│       └── running.parquet       # Running statistics (optional)
```

## Module Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                main.rs                                    │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                          CLI Parsing (clap)                         │  │
│  │   --fetch-leetcode | --sync-deepml | --post | --dry-run          │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                    │                                      │
│                                    ▼                                      │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                        Config Loading                               │  │
│  │                         config.rs                                   │  │
│  │   .env → Config { github_token, repo_*, timezone, ... }            │  │
│  └────────────────────────────────────────────────────────────────────┘  │
│                                    │                                      │
└────────────────────────────────────┼──────────────────────────────────────┘
                                     │
          ┌──────────────────────────┼──────────────────────────┐
          │                          │                          │
          ▼                          ▼                          ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   providers/    │       │     api.rs      │       │    utils.rs     │
│                 │       │                 │       │                 │
│  LeetCode       │       │  fetch_quote()  │       │  get_day_of_    │
│  Deep-ML        │       │  fetch_history()│       │    year()       │
│  select_problem │       │  fetch_running()│       │  get_year_      │
│  (shared logic) │       │                 │       │    progress()   │
│                 │       │  RunningStats   │       │  read_lines()   │
│                 │       │                 │       │  append_line()  │
└────────┬────────┘       └────────┬────────┘       └─────────────────┘
         │                           │
         │                           │
         └────────────┬──────────────┘
                      │
                      ▼
         ┌────────────────────────┐
         │      scheduler.rs        │
         │                        │
         │  Schedule enum         │
         │  get_schedule()        │
         │  (weekday/weekend)     │
         │                        │
         └────────────┬───────────┘
                      │
                      │
                      ▼
         ┌────────────────────────┐
         │      format.rs           │
         │                        │
         │  build_formatted_message │
         │  get_problem_emoji       │
         │                        │
         └────────────┬───────────┘
                      │
                      │ String (message)
                      │
         ┌────────────┴───────────┐
         │                        │
         ▼                        ▼
┌─────────────────┐      ┌────────────────────┐
│   GitHub API    │      │  notification/     │
│   (octocrab)    │      │                    │
│                 │      │  ┌──────────────┐  │
│  Issue Comment  │      │  │ Notifier     │  │
│                 │      │  │   trait      │  │
└─────────────────┘      │  └──────────────┘  │
                         │         │          │
                         │    ┌────┴────┐     │
                         │    │         │     │
                         │    ▼         ▼     │
                         │ ┌──────┐ ┌──────┐  │
                         │ │telegram│ │discord│ │
                         │ └──────┘ └──────┘  │
                         └────────────────────┘
```

## Module Responsibilities

### `main.rs` - Application Entry Point

**Responsibilities:**
- Parse CLI arguments using `clap`
- Load configuration from environment
- Orchestrate data fetching
- Handle wake-up time detection
- Coordinate posting to all channels
- Handle error reporting

**Key Flow:**

```rust
#[tokio::main]
async fn main() -> Result<()> {
    // 1. Parse arguments
    let args = Args::parse();
    
    // 2. Load configuration
    let config = Config::load()?;
    
    // 3. Handle --fetch-leetcode (early exit)
    if args.fetch_leetcode { /* ... */ }
    
    // 4. Handle --sync-deepml (early exit)
    if args.sync_deepml { /* ... */ }
    
    // 5. Check wake-up time
    let is_early = current_hour >= 3 && current_hour <= 9;
    
    // 6. Run routine
    let result = run_routine(&config, &options).await?;
    
    // 7. Handle --dry-run
    if args.dry_run { /* print and exit */ }
    
    // 8. Send notifications
    if args.post { /* GitHub */ }
    if args.telegram { /* Telegram */ }
    if args.discord { /* Discord */ }
}
```

---

### `config.rs` - Configuration Module

**Responsibilities:**
- Load environment variables via `dotenvy`
- Parse and validate configuration
- Provide typed access to settings
- Handle timezone parsing

---

### `providers/` - Problem Provider Abstraction

**Responsibilities:**
- Fetch problem lists from LeetCode (Easy, Medium, Hard)
- Sync Deep-ML problems from GitHub
- Select random unused problems with deterministic seeding
- Track used problems

**Shared Selection Logic:**

```rust
async fn select_problem(
    cache_file: &str,
    used_file: &str,
    difficulty: Difficulty,
    platform: Platform,
    url_generator: impl Fn(&ProblemCache) -> String,
    seed: u64,
) -> Result<ProblemResult> {
    // Read cache, filter by difficulty, filter used, pick random
    // ...
}
```

**LeetCode Problem Selection Algorithm:**

```rust
async fn get_problem() -> Result<ProblemResult> {
    // Try 1: Get official daily challenge
    if let Some(daily) = get_daily_challenge().await? {
        if daily.difficulty == target && !is_used(&daily.slug) {
            return Ok(daily);
        }
    }
    
    // Fallback: Pick random from difficulty pool
    select_problem(cache_file, used_file, difficulty, ...)
}
```

---

### `scheduler.rs` - Difficulty Scheduling

**Responsibilities:**
- Generate weekday difficulty pattern (3 Easy + 2 Medium)
- Handle weekend schedule (Medium + Hard)
- Provide deterministic seeding per week

**Schedule Types:**

```rust
pub enum Schedule {
    Weekday { difficulty: Difficulty },
    Weekend { difficulties: [Difficulty; 2] },
}
```

---

### `api.rs` - External APIs

**Responsibilities:**
- Fetch quotes from quotable.io
- Fetch historical events from Wikimedia
- Parse running statistics from Parquet
- Handle API failures gracefully

---

### `format.rs` - Message Formatting

**Responsibilities:**
- Combine all data into final message
- Format each section with appropriate emojis
- Generate progress display

---

### `notification/` - Notification System

**Responsibilities:**
- Define `Notifier` trait
- Implement Telegram adapter
- Implement Discord adapter
- Handle feature-gated compilation

## Data Flow

```
START
  │
  ▼
┌─────────────────────────────────────┐
│  Load Configuration                 │
│  - Read .env                        │
│  - Validate timezone                │
│  - Parse tokens                     │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│  Determine Current Time             │
│  - Apply timezone                   │
│  - Check wake-up window (3-9 AM)    │
│  - Calculate day of year            │
└──────────────────┬──────────────────┘
                   │
    ┌──────────────┼──────────────┐
    │              │              │
    ▼              ▼              ▼
┌─────────┐  ┌──────────┐  ┌─────────────┐
│ LeetCode│  │  APIs    │  │   Files     │
│ Provider│  │          │  │             │
│         │  │ • Quote  │  │ • easy.txt  │
│ • Daily │  │ • History│  │ • medium.txt│
│   chall │  │          │  │ • hard.txt  │
│ • Diff  │  │          │  │ • deepml.txt│
│   pool  │  │          │  │ • used.txt  │
└────┬────┘  └────┬─────┘  └──────┬──────┘
     │            │               │
     └────────────┼───────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│  Problem Selection                  │
│  ┌─────────────────────────────┐    │
│  │ 1. Get schedule for date    │    │
│  │ 2. Try official daily       │    │
│  │ 3. Check if not used        │    │
│  │ 4. Fallback to seeded pick  │    │
│  └─────────────────────────────┘    │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│  Message Building                   │
│  - Format problems (with emojis)    │
│  - Format running stats             │
│  - Format historical events         │
│  - Combine with time/quote          │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│  Output Routing                     │
│                                      │
│  if --dry-run:  stdout only         │
│  if late:       stdout + "late"     │
│  if --post:     GitHub Issue #1     │
│  if --telegram: Telegram API        │
│  if --discord:  Discord API         │
└──────────────────┬──────────────────┘
                   │
                   ▼
END
```

## Notification Trait Pattern

The notification system uses the Strategy pattern via Rust traits:

```
┌────────────────────────────────────────────────────────────┐
│                    Notifier Trait                          │
│                                                            │
│  #[async_trait]                                            │
│  pub trait Notifier: Send + Sync {                         │
│      async fn send_message(&self, message: &str)          │
│          -> Result<()>;                                    │
│  }                                                         │
└────────────────────────────────────────────────────────────┘
                            │
            ┌───────────────┼───────────────┐
            │               │               │
            ▼               ▼               ▼
    ┌───────────────┐ ┌───────────────┐ ┌───────────────┐
    │    Telegram   │ │    Discord    │ │   Future...   │
    │   Notifier    │ │   Notifier    │ │               │
    │               │ │               │ │   Slack       │
    │ • bot token   │ │ • http client │ │   Email       │
    │ • chat_id     │ │ • channel_id  │ │   Webhook     │
    │               │ │               │ │               │
    │ send_message()│ │ send_message()│ │ send_message()│
    └───────────────┘ └───────────────┘ └───────────────┘
```

## Feature Flags

```
┌─────────────────────────────────────────────────────────────┐
│                    Feature Flags                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [features]                                                 │
│  default = []                                               │
│  telegram = ["dep:teloxide"]                                │
│  discord = ["dep:serenity"]                                 │
│  mcp = ["dep:rmcp", "dep:axum"]                             │
│                                                             │
└─────────────────────────────────────────────────────────────┘
                          │
          ┌───────────────┼───────────────┐
          │               │               │
          ▼               ▼               ▼
    ┌───────────┐   ┌───────────┐   ┌───────────┐
    │  default  │   │  telegram │   │  discord  │
    │   only    │   │           │   │           │
    │           │   │ teloxide  │   │ serenity  │
    │ octocrab  │   │           │   │           │
    │ reqwest   │   │           │   │           │
    │ polars    │   │           │   │           │
    │ chrono    │   │           │   │           │
    │   ...     │   │           │   │           │
    └───────────┘   └───────────┘   └───────────┘
          │               │               │
          └───────────────┼───────────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │  Conditional Compile  │
              │                       │
              │  #[cfg(feature = "telegram")]  │
              │  #[cfg(feature = "discord")]   │
              │  #[cfg(feature = "mcp")]       │
              └───────────────────────┘
```

## Error Handling Strategy

### Error Types

```rust
use anyhow::{Result, Context, anyhow};

// Configuration errors
Config::load()
    .context("Failed to load configuration")?;

// Network errors
client.get(url).send().await
    .context("Failed to fetch from API")?;

// File I/O errors
tokio::fs::read_to_string(path).await
    .with_context(|| format!("Failed to read file: {}", path))?;

// Business logic errors
if available.is_empty() {
    return Err(anyhow!("No available problems found"));
}
```

### Error Flow

```
┌──────────────────────────────────────────────────────────┐
│                    Error Handling                         │
├──────────────────────────────────────────────────────────┤
│                                                          │
│  ┌─────────────┐                                         │
│  │ Any Error   │                                         │
│  └──────┬──────┘                                         │
│         │                                                │
│         ▼                                                │
│  ┌─────────────────────────────────────┐                 │
│  │ .context("Human readable message")  │                 │
│  │                                     │                 │
│  │ Adds context while propagating      │                 │
│  └──────────────┬──────────────────────┘                 │
│                 │                                        │
│                 ▼                                        │
│  ┌─────────────────────────────────────┐                 │
│  │ anyhow::Error chain                 │                 │
│  │                                     │                 │
│  │ "Failed to fetch problems"          │                 │
│  │   └─ "HTTP request failed"          │                 │
│  │       └─ "Connection refused"       │                 │
│  └─────────────────────────────────────┘                 │
│                                                          │
└──────────────────────────────────────────────────────────┘
```

### Graceful Degradation

| Component | Failure Mode | Fallback |
|-----------|--------------|----------|
| Quote API | Network error | Default quote |
| History API | Network error | Empty history list |
| Running stats | File not found | Zeroed stats |
| LeetCode API | Network error | Return error (required) |
| Deep-ML API | Network error | Use existing cache |
| GitHub API | Auth error | Return error (required) |

## Related Documentation

- [API Reference](./api-reference.md) - Detailed module documentation
- [Development Guide](./development.md) - Adding new features
- [Deployment Guide](./deployment.md) - Production considerations
