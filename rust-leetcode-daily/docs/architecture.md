# Architecture Overview

This document describes the architecture, design patterns, and data flow of rust-leetcode-daily.

## Project Structure

```
leetcode-daily/
├── Cargo.toml                 # Dependencies and feature flags
├── .env.example               # Environment template
├── src/
│   ├── main.rs               # Entry point, CLI, orchestration
│   ├── config.rs             # Configuration loading and types
│   ├── leetcode.rs           # LeetCode GraphQL client
│   ├── api.rs                # External API clients (quote, history, running)
│   ├── message.rs            # Message composition
│   ├── utils.rs              # Utilities (date, file I/O)
│   └── notification/
│       ├── mod.rs            # Notifier trait definition
│       ├── telegram.rs       # Telegram implementation
│       └── discord.rs        # Discord implementation
└── data/
    ├── leetcode_easy.txt     # Cached EASY problems
    ├── used_problems.txt     # Problem usage tracking
    └── running.parquet       # Running statistics (optional)
```

## Module Diagram

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                main.rs                                    │
│  ┌────────────────────────────────────────────────────────────────────┐  │
│  │                          CLI Parsing (clap)                         │  │
│  │   --fetch-easy | --post | --telegram | --discord | --dry-run        │  │
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
         ┌───────────────────────────┼───────────────────────────┐
         │                           │                           │
         ▼                           ▼                           ▼
┌─────────────────┐       ┌─────────────────┐       ┌─────────────────┐
│   leetcode.rs   │       │     api.rs      │       │    utils.rs     │
│                 │       │                 │       │                 │
│  LeetCode       │       │  fetch_quote()  │       │  get_day_of_    │
│  GraphQL API    │       │  fetch_history()│       │    year()       │
│                 │       │  fetch_running()│       │  get_year_      │
│  Question       │       │                 │       │    progress()   │
│  struct         │       │  RunningStats   │       │  read_lines()   │
│                 │       │  struct         │       │  append_line()  │
└────────┬────────┘       └────────┬────────┘       └─────────────────┘
         │                         │
         │                         │
         └────────────┬────────────┘
                      │
                      ▼
         ┌────────────────────────┐
         │      message.rs        │
         │                        │
         │  build_message()       │
         │  format_leetcode()     │
         │  format_running()      │
         │  format_history()      │
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
    
    // 3. Handle --fetch-easy (early exit)
    if args.fetch_easy { /* ... */ }
    
    // 4. Check wake-up time
    let is_early = current_hour >= 3 && current_hour <= 9;
    
    // 5. Fetch all data (parallel would be ideal)
    let quote = fetch_quote().await?;
    let history = fetch_history().await?;
    let running = fetch_running_stats().await?;
    let problem = get_today_problem().await?;
    
    // 6. Build message
    let message = build_message(/* ... */);
    
    // 7. Handle --dry-run
    if args.dry_run { /* print and exit */ }
    
    // 8. Check if late
    if !is_early { /* print "late" and exit */ }
    
    // 9. Send notifications
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

**Design:**

```rust
pub struct Config {
    // All configuration as typed fields
}

impl Config {
    pub fn load() -> Result<Self> {
        dotenv().ok();  // Load .env file
        
        // Parse and validate each field
        let github_token = env::var("GITHUB_TOKEN")?;
        // ...
        
        Ok(Config { /* fields */ })
    }
}
```

**Validation:**
- Required fields return error if missing
- Optional fields use `Option<String>`
- Timezone validated against IANA database
- Birth year parsed as integer

---

### `leetcode.rs` - LeetCode Integration

**Responsibilities:**
- Fetch EASY problem list from LeetCode
- Get official daily challenge
- Select random problem (seeded by date)
- Track used problems
- Format problem messages

**Problem Selection Algorithm:**

```rust
async fn get_today_problem() -> Result<Question> {
    // Try 1: Get official daily challenge
    if let Some(daily) = get_daily_challenge().await? {
        // Only use if EASY and not paid
        if daily.difficulty == "EASY" && !daily.paid_only {
            if !is_used(&daily.slug) {
                mark_used(&daily.slug);
                return Ok(daily);
            }
        }
    }
    
    // Fallback: Pick random from EASY pool
    pick_daily_problem().await
}

async fn pick_daily_problem() -> Result<Question> {
    let available = filter_unused(easy_problems);
    
    // Seeded random for consistent daily selection
    let seed = year * 1000 + day_of_year;
    let problem = seeded_random_pick(available, seed);
    
    mark_used(&problem.slug);
    Ok(problem)
}
```

---

### `api.rs` - External APIs

**Responsibilities:**
- Fetch quotes from quotable.io
- Fetch historical events from Wikimedia
- Parse running statistics from Parquet
- Handle API failures gracefully

**Error Handling:**

```rust
pub async fn fetch_quote(client: &Client) -> Result<String> {
    match client.get(QUOTE_URL).send().await?.json().await {
        Ok(response) => Ok(format!("{} — {}", response.content, response.author)),
        Err(_) => Ok(DEFAULT_QUOTE.to_string()),  // Fallback
    }
}
```

**Parquet Processing:**

```rust
pub async fn fetch_running_stats(file: &str) -> Result<RunningStats> {
    let df = LazyFrame::scan_parquet(file, Default::default())?;
    
    // Calculate aggregates for different periods
    let yesterday = df.clone().filter(/* yesterday */).sum();
    let month = df.clone().filter(/* this month */).sum();
    let year = df.filter(/* this year */).sum();
    
    Ok(RunningStats { /* aggregated values */ })
}
```

---

### `message.rs` - Message Composition

**Responsibilities:**
- Combine all data into final message
- Format each section appropriately
- Generate progress bars
- Apply markdown formatting

**Message Template:**

```rust
pub fn build_message(/* all data */) -> String {
    format!(
        r#"Wake up time: {}

Good morning!

Day {} of the year.

{}

{}

{}

{}

Today's Quote:
{}"#,
        get_up_time,
        day_of_year,
        year_progress,
        leetcode_info,
        running_info,
        history_today,
        quote,
    )
}
```

---

### `notification/` - Notification System

**Responsibilities:**
- Define `Notifier` trait
- Implement Telegram adapter
- Implement Discord adapter
- Handle feature-gated compilation

## Data Flow Diagram

```
┌────────────────────────────────────────────────────────────────────────────┐
│                             DATA FLOW                                       │
└────────────────────────────────────────────────────────────────────────────┘

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
│         │  │          │  │             │
│ • Daily │  │ • Quote  │  │ • easy.txt  │
│   chall │  │ • History│  │ • used.txt  │
│ • EASY  │  │          │  │ • running   │
│   list  │  │          │  │   .parquet  │
└────┬────┘  └────┬─────┘  └──────┬──────┘
     │            │               │
     └────────────┼───────────────┘
                  │
                  ▼
┌─────────────────────────────────────┐
│  Question Selection                 │
│  ┌─────────────────────────────┐    │
│  │ 1. Try official daily       │    │
│  │ 2. Check if EASY & free     │    │
│  │ 3. Check if not used        │    │
│  │ 4. Fallback to seeded pick  │    │
│  └─────────────────────────────┘    │
└──────────────────┬──────────────────┘
                   │
                   ▼
┌─────────────────────────────────────┐
│  Message Building                   │
│  - Format LeetCode problem          │
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

**Benefits:**
- Easy to add new notification channels
- Consistent interface across all adapters
- Feature-gated compilation
- Testable with mocks

**Implementation Example:**

```rust
// notification/mod.rs
pub trait Notifier: Send + Sync {
    async fn send_message(&self, message: &str) -> Result<()>;
}

// notification/telegram.rs
impl Notifier for TelegramNotifier {
    async fn send_message(&self, message: &str) -> Result<()> {
        self.bot.send_message(self.chat_id, message).await?;
        Ok(())
    }
}
```

## Feature Flags Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Feature Flags                             │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  [features]                                                 │
│  default = []                                               │
│  telegram = ["dep:teloxide"]                                │
│  discord = ["dep:serenity"]                                 │
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
    │ octocrab  │   │ added     │   │ added     │
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
              └───────────────────────┘
```

**Conditional Compilation:**

```rust
// In main.rs
#[cfg(feature = "telegram")]
let telegram_notifier = config.telegram_token.as_ref().and_then(|token| {
    config.telegram_chat_id.as_ref().map(|chat_id| {
        TelegramNotifier::new(token.clone(), chat_id.clone())
    })
});

#[cfg(not(feature = "telegram"))]
let telegram_notifier: Option<TelegramNotifier> = None;
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
    return Err(anyhow!("No available EASY problems found"));
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
│  │ "Failed to fetch EASY problems"     │                 │
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
| GitHub API | Auth error | Return error (required) |

## Related Documentation

- [API Reference](./api-reference.md) - Detailed module documentation
- [Development Guide](./development.md) - Adding new features
- [Deployment Guide](./deployment.md) - Production considerations