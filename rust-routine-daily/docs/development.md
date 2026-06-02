# Development Guide

This guide covers development setup, testing, code style, and contributing guidelines for rust-leetcode-daily.

## Table of Contents

- [Project Setup](#project-setup)
- [Running Tests](#running-tests)
- [Code Style](#code-style)
- [Adding New Notification Adapters](#adding-new-notification-adapters)
- [Adding New Data Sources](#adding-new-data-sources)
- [Debugging Tips](#debugging-tips)
- [Performance Considerations](#performance-considerations)
- [Pull Request Guidelines](#pull-request-guidelines)

## Project Setup

### Prerequisites

- **Rust 1.75+** - Install via [rustup](https://rustup.rs/)
- **Git** - Version control
- **GitHub Account** - For repository access and API tokens

### Initial Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/rust-leetcode-daily.git
cd rust-leetcode-daily

# Copy environment template
cp .env.example .env

# Edit .env with your credentials
# Required: GITHUB_TOKEN, REPO_OWNER, REPO_NAME, BIRTH_YEAR, TIMEZONE
vim .env

# Build the project
cargo build

# Build with all notification features
cargo build --features telegram,discord

# Verify setup with dry run
cargo run -- --dry-run
```

### Development Dependencies

The project uses these key dependencies (see `Cargo.toml`):

| Dependency | Purpose |
|------------|---------|
| `tokio` | Async runtime |
| `reqwest` | HTTP client |
| `serde` / `serde_json` | JSON serialization |
| `chrono` / `chrono-tz` | Date/time handling |
| `clap` | CLI argument parsing |
| `anyhow` | Error handling |
| `octocrab` | GitHub API client |
| `polars` | Parquet processing |
| `teloxide` | Telegram bot (optional) |
| `serenity` | Discord bot (optional) |

### IDE Setup

#### VS Code

Recommended extensions:

```json
{
  "recommendations": [
    "rust-lang.rust-analyzer",
    "tamasfe.even-better-toml",
    "serayuzgur.crates",
    "vadimcn.vscode-lldb"
  ]
}
```

#### rust-analyzer Settings

```json
{
  "rust-analyzer.cargo.features": "all",
  "rust-analyzer.checkOnSave.command": "clippy"
}
```

## Running Tests

### Test Structure

```
tests/
├── integration_tests.rs    # End-to-end tests
└── mocks/                  # Mock servers and fixtures

src/
└── *.rs                    # Unit tests in #[cfg(test)] modules
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_fetch_quote

# Run tests with output
cargo test -- --nocapture

# Run tests with all features
cargo test --all-features

# Run integration tests only
cargo test --test integration_tests

# Run unit tests only
cargo test --lib
```

### Writing Tests

#### Unit Test Example

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_day_seed() {
        let seed = LeetCode::get_day_seed();
        assert!(seed > 0);
        assert!(seed < 1_000_000);
    }

    #[test]
    fn test_pick_seeded_random_deterministic() {
        let items = vec!["a", "b", "c"];
        let first = LeetCode::pick_seeded_random(&items, 42);
        let second = LeetCode::pick_seeded_random(&items, 42);
        assert_eq!(first, second);
    }
}
```

#### Integration Test with Mock Server

```rust
use wiremock::{MockServer, Mock, ResponseTemplate};
use wiremock::matchers::{method, path};

#[tokio::test]
async fn test_fetch_quote_fallback() {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/random"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let client = reqwest::Client::new();
    let result = fetch_quote(&client).await;
    
    assert!(result.is_ok());
    assert!(result.unwrap().contains("Steve Jobs"));
}
```

### Test Coverage

```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View report
open coverage/tarpaulin-report.html
```

## Code Style

### Formatting with rustfmt

```bash
# Check formatting
cargo fmt -- --check

# Apply formatting
cargo fmt
```

#### rustfmt.toml

```toml
max_width = 100
tab_spaces = 4
edition = "2021"
use_small_heuristics = "Default"
```

### Linting with Clippy

```bash
# Run clippy
cargo clippy

# Run with all features
cargo clippy --all-features

# Treat warnings as errors
cargo clippy -- -D warnings
```

#### Common Clippy Fixes

```rust
// Avoid: explicit type annotation when inferable
let x: i32 = 5;

// Prefer: let inference work
let x = 5;

// Avoid: unnecessary borrow
fn foo(x: &String) { }

// Prefer: take String or &str
fn foo(x: &str) { }

// Avoid: manual string formatting
format!("Hello {}", name);

// Prefer: same result, clearer intent
format!("Hello {name}");
```

### Code Conventions

#### Error Handling

```rust
// Use anyhow::Context for meaningful error chains
use anyhow::{Context, Result};

pub async fn fetch_data(url: &str) -> Result<String> {
    let response = client
        .get(url)
        .send()
        .await
        .context(format!("Failed to fetch from {}", url))?;
    
    let body = response
        .text()
        .await
        .context("Failed to read response body")?;
    
    Ok(body)
}
```

#### Async Functions

```rust
// Use async_trait for trait methods
#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send_message(&self, message: &str) -> Result<()>;
}

// Prefer tokio::fs for async file operations
let content = tokio::fs::read_to_string(path)
    .await
    .context("Failed to read file")?;
```

#### Feature-Gated Code

```rust
#[cfg(feature = "telegram")]
pub fn create_telegram_notifier(token: String, chat_id: String) -> TelegramNotifier {
    TelegramNotifier::new(token, chat_id)
}

#[cfg(not(feature = "telegram"))]
pub fn create_telegram_notifier(_token: String, _chat_id: String) -> &'static str {
    "Telegram feature not enabled"
}
```

## Adding New Notification Adapters

The notification system uses the Strategy pattern with Rust traits. Follow these steps to add a new adapter.

### Step 1: Define the Adapter Structure

Create a new file `src/notification/slack.rs`:

```rust
use crate::notification::Notifier;
use anyhow::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde_json::json;

#[cfg(feature = "slack")]
pub struct SlackNotifier {
    client: Client,
    webhook_url: String,
}

#[cfg(feature = "slack")]
impl SlackNotifier {
    pub fn new(webhook_url: String) -> Self {
        Self {
            client: Client::new(),
            webhook_url,
        }
    }
}

#[cfg(feature = "slack")]
#[async_trait]
impl Notifier for SlackNotifier {
    async fn send_message(&self, message: &str) -> Result<()> {
        let payload = json!({
            "text": message
        });
        
        self.client
            .post(&self.webhook_url)
            .json(&payload)
            .send()
            .await?;
        
        Ok(())
    }
}

// Stub implementation when feature is disabled
#[cfg(not(feature = "slack"))]
pub struct SlackNotifier;

#[cfg(not(feature = "slack"))]
impl SlackNotifier {
    pub fn new(_webhook_url: String) -> Self {
        Self
    }
}

#[cfg(not(feature = "slack"))]
#[async_trait]
impl Notifier for SlackNotifier {
    async fn send_message(&self, _message: &str) -> Result<()> {
        anyhow::bail!("Slack feature not enabled. Recompile with --features slack")
    }
}
```

### Step 2: Update Module Exports

Edit `src/notification/mod.rs`:

```rust
mod telegram;
mod discord;
mod slack;  // Add this line

pub use telegram::TelegramNotifier;
pub use discord::DiscordNotifier;
pub use slack::SlackNotifier;  // Add this line

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Notifier: Send + Sync {
    async fn send_message(&self, message: &str) -> Result<()>;
}
```

### Step 3: Add Feature Flag

Edit `Cargo.toml`:

```toml
[features]
default = []
telegram = ["dep:teloxide"]
discord = ["dep:serenity"]
slack = ["dep:reqwest"]  # Add this line (reqwest already included)

# No new dependency needed for Slack webhooks
```

If you need a new dependency:

```toml
[features]
slack = ["dep:slack-sdk"]

[dependencies]
slack-sdk = { version = "0.1", optional = true }
```

### Step 4: Add Configuration

Edit `src/config.rs`:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // ... existing fields
    pub slack_webhook_url: Option<String>,  // Add this
}

impl Config {
    pub fn load() -> Result<Self> {
        // ... existing code
        
        let slack_webhook_url = env::var("SLACK_WEBHOOK_URL").ok();  // Add this
        
        Ok(Config {
            // ... existing fields
            slack_webhook_url,
        })
    }
}
```

### Step 5: Integrate in main.rs

Edit `src/main.rs`:

```rust
if args.slack {
    #[cfg(feature = "slack")]
    let slack_notifier = config.slack_webhook_url.as_ref().map(|url| {
        notification::SlackNotifier::new(url.clone())
    });

    #[cfg(not(feature = "slack"))]
    let slack_notifier: Option<notification::SlackNotifier> = None;

    if let Some(notifier) = slack_notifier {
        notifier.send_message(&message).await?;
        println!("Sent notification via Slack");
    } else {
        println!("Slack webhook not configured or feature not enabled");
    }
}
```

### Step 6: Update CLI Arguments

Edit `src/main.rs`:

```rust
#[derive(Parser, Debug)]
struct Args {
    // ... existing args
    
    #[arg(long, help = "Send notification via Slack")]
    slack: bool,
}
```

### Step 7: Update Documentation

Add to `.env.example`:

```bash
# Slack (Optional)
SLACK_WEBHOOK_URL=https://hooks.slack.com/services/XXX/YYY/ZZZ
```

### Testing Your Adapter

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_slack_notifier_send() {
        // Use a mock server or test webhook
        let notifier = SlackNotifier::new(
            "https://hooks.slack.com/services/test".to_string()
        );
        
        // This will fail with real webhook, use mock server in tests
        let result = notifier.send_message("Test message").await;
        // Assert based on mock response
    }
}
```

## Adding New Data Sources

### Step 1: Define Data Structure

In `src/api.rs` or a new module:

```rust
#[derive(Debug, Clone)]
pub struct WeatherData {
    pub temperature: f64,
    pub conditions: String,
    pub location: String,
}
```

### Step 2: Create Fetch Function

```rust
pub async fn fetch_weather(
    client: &reqwest::Client,
    latitude: f64,
    longitude: f64,
) -> Result<WeatherData> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current_weather=true",
        latitude, longitude
    );
    
    let response = client
        .get(&url)
        .send()
        .await
        .context("Failed to fetch weather data")?;
    
    let data: WeatherResponse = response
        .json()
        .await
        .context("Failed to parse weather response")?;
    
    Ok(WeatherData {
        temperature: data.current_weather.temperature,
        conditions: format_weather_code(data.current_weather.weathercode),
        location: format!("{}, {}", latitude, longitude),
    })
}
```

### Step 3: Add Graceful Fallback

```rust
pub async fn fetch_weather(/* ... */) -> Result<WeatherData> {
    match client.get(&url).send().await {
        Ok(response) => {
            // Parse and return
        }
        Err(_) => {
            // Return default/empty data
            Ok(WeatherData {
                temperature: 0.0,
                conditions: "Unknown".to_string(),
                location: "Unknown".to_string(),
            })
        }
    }
}
```

### Step 4: Add Message Formatting

In `src/message.rs`:

```rust
pub fn format_weather(weather: &WeatherData) -> String {
    format!(
        "Weather: {:.1}C, {}",
        weather.temperature,
        weather.conditions
    )
}
```

### Step 5: Integrate into Message

```rust
pub fn build_message(
    // ... existing params
    weather: &WeatherData,
) -> String {
    format!(
        r#"...
{}
..."#,
        format_weather(weather),
    )
}
```

### Step 6: Add Configuration (if needed)

```rust
// In config.rs
pub struct Config {
    // ...
    pub weather_latitude: Option<f64>,
    pub weather_longitude: Option<f64>,
}
```

## Debugging Tips

### Enable Logging

Add `tracing` to `Cargo.toml`:

```toml
[dependencies]
tracing = "0.1"
tracing-subscriber = "0.3"
```

Initialize in `main.rs`:

```rust
fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    // ... rest of main
}
```

Add debug output:

```rust
use tracing::{debug, info, warn, error};

pub async fn fetch_quote(client: &reqwest::Client) -> Result<String> {
    debug!("Fetching quote from API");
    match client.get(QUOTE_URL).send().await {
        Ok(response) => {
            debug!(status = ?response.status(), "Received response");
            // ...
        }
        Err(e) => {
            warn!(error = ?e, "Failed to fetch quote, using default");
            Ok(DEFAULT_QUOTE.to_string())
        }
    }
}
```

### Print Environment Variables

```bash
# Debug environment loading
RUST_LOG=debug cargo run -- --dry-run
```

### Inspect HTTP Requests

Use `reqwest` middleware for request logging:

```rust
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_retry::RetryTransientMiddleware;

let client = ClientBuilder::new(reqwest::Client::new())
    .with(RetryTransientMiddleware::new_with_max_retries(3))
    .build();
```

### Debug GraphQL Queries

```rust
let request_body = serde_json::to_string_pretty(&request)?;
eprintln!("GraphQL Request:\n{}", request_body);
```

### Common Issues

| Issue | Cause | Solution |
|-------|-------|----------|
| "GITHUB_TOKEN must be set" | Missing .env file | Copy `.env.example` to `.env` |
| "Invalid timezone" | Wrong timezone format | Use IANA format: `America/New_York` |
| "Failed to fetch EASY problems" | LeetCode API blocked | Use VPN or LeetCode CN |
| "Already posted today" | Expected behavior | See [FAQ](./faq.md) |
| Rate limited | Too many requests | Add delays between requests |

### Debugging with LLDB

```bash
# Build with debug symbols
cargo build

# Debug with lldb
rust-lldb ./target/debug/routine-daily -- --dry-run

# In lldb:
(lldb) breakpoint set --file main.rs --line 50
(lldb) run
```

## Performance Considerations

### Parallel Data Fetching

Fetch independent data concurrently:

```rust
use tokio::try_join;

// Sequential (slower)
let quote = fetch_quote(&client).await?;
let history = fetch_history(&client, birth_year, month, day).await?;
let running = fetch_running_stats(file).await?;

// Parallel (faster)
let (quote, history, running) = try_join!(
    fetch_quote(&client),
    fetch_history(&client, birth_year, month, day),
    fetch_running_stats(file),
)?;
```

### Caching Strategies

```rust
use std::time::Duration;
use tokio::sync::OnceCell;

static EASY_PROBLEMS: OnceCell<Vec<Question>> = OnceCell::const_new();

async fn get_cached_easy_problems() -> &'static Vec<Question> {
    EASY_PROBLEMS
        .get_or_init(|| async {
            fetch_easy_problems().await.unwrap_or_default()
        })
        .await
}
```

### Connection Pooling

```rust
// Reuse the HTTP client across requests
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(60))
    .build()?;
```

### Memory Efficiency

```rust
// Avoid cloning large strings
pub fn build_message(/* ... */) -> String {
    // Use String::with_capacity for known sizes
    let mut message = String::with_capacity(1024);
    message.push_str(&format!("Day {}\n", day));
    // ...
    message
}
```

### Benchmarking

```bash
# Install criterion
cargo install cargo-criterion

# Add to Cargo.toml
[dev-dependencies]
criterion = "0.5"

# Run benchmarks
cargo criterion
```

## Pull Request Guidelines

### Before Submitting

1. **Format your code**
   ```bash
   cargo fmt
   ```

2. **Fix all clippy warnings**
   ```bash
   cargo clippy --all-features -- -D warnings
   ```

3. **Run all tests**
   ```bash
   cargo test --all-features
   ```

4. **Update documentation**
   - Update README.md if needed
   - Update relevant docs in `docs/`
   - Add inline comments for complex logic

### PR Checklist

```markdown
- [ ] Code formatted with `cargo fmt`
- [ ] No clippy warnings (`cargo clippy`)
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Documentation updated
- [ ] CHANGELOG.md updated (if applicable)
- [ ] Commit messages follow conventional format
```

### Commit Message Format

Use conventional commits:

```
feat: add Slack notification adapter

- Add SlackNotifier implementation
- Add SLACK_WEBHOOK_URL configuration
- Update CLI with --slack flag

Closes #123
```

Types:
- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `refactor:` - Code refactoring
- `test:` - Adding tests
- `chore:` - Maintenance tasks

### Branch Naming

```
feat/slack-notifier
fix/quote-api-timeout
docs/api-reference-update
refactor/message-builder
```

### Code Review Process

1. Create a feature branch from `main`
2. Make focused, atomic commits
3. Open a Pull Request with description
4. Address review feedback
5. Wait for CI to pass
6. Request review from maintainers

### CI Pipeline

The project uses GitHub Actions (`.github/workflows/ci.yml`):

```yaml
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: cargo fmt -- --check
      - run: cargo clippy --all-features -- -D warnings
      - run: cargo test --all-features
```

## Related Documentation

- [API Reference](./api-reference.md) - Module and function documentation
- [Architecture](./architecture.md) - System design and data flow
- [Configuration](./configuration.md) - Environment setup
- [Deployment](./deployment.md) - Production deployment
- [FAQ](./faq.md) - Common issues and solutions
