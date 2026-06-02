# Rust LeetCode Daily Motivator Project Plan

This document outlines a comprehensive plan for implementing a Rust-based version of the [2026 GitHub repo](https://github.com/yihong0618/2026). The original project automates daily motivational "get-up" messages posted as comments on GitHub Issue #1, including elements like time, year progress, random poems, historical events, running stats, and a LeetCode EASY problem suggestion. It fetches EASY problems from LeetCode CN (with notes on adapting to LeetCode.com) and prioritizes the daily challenge if it's EASY and unused.

The Rust version will be a CLI tool (`routine-daily`), runnable via cron for daily execution. It supports both LeetCode CN and .com via config, handles authentication-free GraphQL queries, and includes posting to GitHub and optional Telegram notifications. Focus on modularity, async I/O, error handling, and testability.

**Goals:**
- Replicate core functionality with Rust's safety and performance.
- Make it extensible (e.g., add more APIs).
- Ensure it works cross-platform (Linux/macOS/Windows).
- Initial setup time: ~2-4 hours; full implementation: ~1-2 days.

**Assumptions:**
- Rust 1.75+ installed (`rustup update stable`).
- GitHub Personal Access Token (PAT) for posting.
- Optional: Telegram Bot Token for notifications.
- Running stats from a Parquet file (e.g., from Strava export; fetch via separate script if needed).
- No paid LeetCode problems; filter for EASY difficulty.

---

## 1. Project Initialization

### Step 1: Create the Project
```bash
cargo new rust-routine-daily
cd rust-routine-daily
```

### Step 2: Update Cargo.toml
Add dependencies for HTTP, async, JSON, GitHub/Telegram clients, data querying, and utils. Use `cargo add` for each or edit manually:

```toml
[package]
name = "routine-daily"
version = "0.1.0"
edition = "2021"

[dependencies]
# HTTP & JSON
reqwest = { version = "0.12", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Async runtime
tokio = { version = "1.0", features = ["full"] }

# Dates & Timezones
chrono = { version = "0.4", features = ["serde"] }
chrono-tz = "0.9"

# Random & Utils
rand = "0.8"
anyhow = "1.0"
dotenvy = "0.15"

# GitHub API
octocrab = "0.39"

# Telegram Bot (optional; enable via feature flag)
telegram-bot = { version = "0.7", optional = true }

# Parquet for Running Stats
polars = { version = "0.42", features = ["parquet", "lazy"] }

# Chinese Text Utils (for history/poems if needed)
zhconv = "0.1"  # Optional: Simplified/Traditional conversion

[features]
default = []
telegram = ["dep:telegram-bot"]

[[bin]]
name = "routine-daily"
path = "src/main.rs"
```

Run `cargo build` to verify.

### Step 3: Environment Setup
Create `.env` in project root (add to `.gitignore`):
```
# GitHub
GITHUB_TOKEN=ghp_your_personal_access_token_here  # scopes: repo (for issues/comments)

# Repo Details
REPO_OWNER=yourusername
REPO_NAME=2026-rust  # Create this repo on GitHub

# Telegram (optional)
TELEGRAM_TOKEN=your_bot_token
TELEGRAM_CHAT_ID=your_chat_id

# Personal
BIRTH_YEAR=1989  # For historical events age calc
TIMEZONE=Africa/Cairo  # EET; adjust as needed

# LeetCode
LEETCODE_ENDPOINT=https://leetcode.cn/graphql/  # Or https://leetcode.com/graphql/
LEETCODE_VARIANT=cn  # 'cn' or 'com' for query adjustments
```

Create `.gitignore`:
```
target/
.env
leetcode_easy.txt
used_problems.txt
*.parquet  # Temp running data
```

### Step 4: Data Files
- `leetcode_easy.txt`: List of EASY problem slugs/IDs (one per line, e.g., "1-Two Sum").
- `used_problems.txt`: Used slugs (append daily).
- `running.parquet`: Download your Strava export Parquet (or generate via Polars script).

---

## 2. Project Structure
```
rust-routine-daily/
├── Cargo.toml
├── .env.example          # Template for .env
├── README.md             # Usage, cron setup
├── src/
│   ├── main.rs           # CLI entry: Parse args, orchestrate, post message
│   ├── config.rs         # Load .env, struct for settings
│   ├── leetcode.rs       # GraphQL fetches, problem selection
│   ├── api.rs            # External: Poem, history, running stats
│   ├── utils.rs          # Date progress, file I/O, random, formatting
│   └── message.rs        # Template rendering, validation
├── data/                 # Static files (gitignored outputs)
│   ├── leetcode_easy.txt
│   ├── used_problems.txt
│   └── running.parquet   # Symlink or fetch script
└── tests/                # Unit/integration tests
    ├── leetcode.rs
    └── api.rs
```

---

## 3. Module Breakdown & Code Snippets

### 3.1 config.rs
Handles loading from `.env`. Use `anyhow` for errors.

```rust
use dotenvy::dotenv;
use std::env;
use chrono_tz::Tz;

#[derive(Debug, Clone)]
pub struct Config {
    pub github_token: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub telegram_token: Option<String>,
    pub telegram_chat_id: Option<String>,
    pub birth_year: i32,
    pub timezone: Tz,
    pub leetcode_endpoint: String,
    pub leetcode_variant: String,  // "cn" or "com"
}

impl Config {
    pub fn load() -> anyhow::Result<Self> {
        dotenv().ok();
        Ok(Self {
            github_token: env::var("GITHUB_TOKEN")?,
            repo_owner: env::var("REPO_OWNER")?,
            repo_name: env::var("REPO_NAME")?,
            telegram_token: env::var("TELEGRAM_TOKEN").ok(),
            telegram_chat_id: env::var("TELEGRAM_CHAT_ID").ok(),
            birth_year: env::var("BIRTH_YEAR")?.parse()?,
            timezone: env::var("TIMEZONE")?.parse()?,
            leetcode_endpoint: env::var("LEETCODE_ENDPOINT")?,
            leetcode_variant: env::var("LEETCODE_VARIANT").unwrap_or_else(|_| "cn".to_string()),
        })
    }
}
```

**ToDo:** Add validation (e.g., valid TZ).

### 3.2 utils.rs
Helpers for dates, files, random.

```rust
use chrono::{DateTime, Utc, LocalResult};
use chrono_tz::Tz;
use rand::seq::SliceRandom;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use anyhow::Result;

pub fn get_year_progress(now: &DateTime<Utc>) -> String {
    let day_of_year = now.ordinal();
    let total_days = if now.is_leap() { 366 } else { 365 };
    let filled = (day_of_year as f32 / total_days as f32 * 20.0) as usize;  // 20-char bar
    let bar = "█".repeat(filled) + &"░".repeat(20 - filled);
    format!("{}/{} ({:.1}%) {}", day_of_year, total_days, (day_of_year as f32 / total_days as f32) * 100.0, bar)
}

pub fn read_lines(filename: &str) -> Result<Vec<String>> {
    let file = File::open(filename)?;
    Ok(BufReader::new(file).lines().filter_map(Result::ok).collect())
}

pub fn append_line(filename: &str, line: &str) -> Result<()> {
    let mut file = fs::OpenOptions::new().append(true).create(true).open(filename)?;
    writeln!(file, "{}", line)?;
    Ok(())
}

pub fn pick_random<T: Clone>(items: &[T]) -> Option<T> {
    items.choose(&mut rand::thread_rng()).cloned()
}

// Time in local TZ
pub fn get_local_time(config: &crate::config::Config) -> DateTime<chrono_tz::Local> {
    let utc_now = Utc::now();
    utc_now.with_timezone(&config.timezone).naive_local().into()
}
```

**ToDo:** Add tests for progress bar (e.g., Jan 1: 1/365, bar starts empty).

### 3.3 leetcode.rs
Core: Fetch EASY list (paginated), get daily, pick unused EASY.

Define GraphQL structs with `serde`. Adjust queries based on `variant`.

```rust
use reqwest::Client;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use crate::config::Config;
use crate::utils::{read_lines, append_line};

#[derive(Serialize)]
struct GraphQLRequest {
    query: String,
    variables: serde_json::Value,
}

#[derive(Deserialize)]
struct GraphQLResponse<T> {
    data: T,
}

#[derive(Deserialize)]
struct ProblemsetData {
    #[serde(rename = "problemsetQuestionList")]
    problemset: ProblemsetList,
}

#[derive(Deserialize)]
struct ProblemsetList {
    total: i32,
    questions: Vec<Question>,
}

#[derive(Deserialize, Clone)]
pub struct Question {
    #[serde(rename = "questionFrontendId")]
    pub id: String,
    pub title: String,
    #[serde(rename = "titleSlug")]
    pub slug: String,
    pub difficulty: String,  // "Easy"
    #[serde(rename = "isPaidOnly")]
    pub paid_only: bool,
}

pub struct LeetCode {
    client: Client,
    endpoint: String,
    variant: String,
}

impl LeetCode {
    pub fn new(config: &Config) -> Self {
        Self {
            client: Client::new(),
            endpoint: config.leetcode_endpoint.clone(),
            variant: config.leetcode_variant.clone(),
        }
    }

    pub async fn fetch_easy_list(&self, output_file: &str) -> Result<()> {
        let mut skip = 0;
        let limit = 500;
        let mut questions = Vec::new();

        loop {
            let query = r#"
                query problemsetQuestionList($category: QuestionListCategory!, $skip: Int!, $limit: Int!) {
                    problemsetQuestionList(category: $category, skip: $skip, limit: $limit) {
                        total
                        questions {
                            questionFrontendId
                            title
                            titleSlug
                            difficulty
                            isPaidOnly
                        }
                    }
                }
            "#;

            let variables = serde_json::json!({
                "category": "SIMPLE",  // EASY
                "skip": skip,
                "limit": limit
            });

            let req = GraphQLRequest { query: query.to_string(), variables };
            let res: GraphQLResponse<ProblemsetData> = self.client
                .post(&self.endpoint)
                .json(&req)
                .send()
                .await?
                .json()
                .await?;

            let list = res.data.problemset;
            questions.extend(list.questions.into_iter().filter(|q| q.difficulty == "Easy" && !q.paid_only));

            if skip + limit as i32 >= list.total {
                break;
            }
            skip += limit as i32;
        }

        // Write to file: "id-titleSlug"
        let mut file = std::fs::File::create(output_file)?;
        for q in questions {
            writeln!(file, "{}: {}-{}", q.id, q.id, q.slug)?;
        }
        Ok(())
    }

    pub async fn get_daily_challenge(&self) -> Result<Option<Question>> {
        let (query_field, query_name) = if self.variant == "cn" {
            ("todayRecord", "questionOfToday")
        } else {
            ("activeDailyCodingChallengeQuestion", "activeDailyCodingChallengeQuestion")
        };

        let query = format!(r#"
            query {} {{
                {}
                {{
                    question {{
                        questionFrontendId
                        title
                        titleSlug
                        difficulty
                        isPaidOnly
                    }}
                }}
            }}
        "#, query_name, query_field);

        let req = GraphQLRequest {
            query,
            variables: serde_json::Value::Null,
        };

        let res: serde_json::Value = self.client.post(&self.endpoint).json(&req).send().await?.json().await?;
        // Parse dynamically or use conditional Deserialize; for simplicity, assume .cn structure and adjust in code
        if let Some(obj) = res["data"][query_field.as_str().unwrap_or("todayRecord")].as_array() {
            if let Some(record) = obj.first() {
                if let Some(q) = record["question"].as_object() {
                    let id = q["questionFrontendId"].as_str().unwrap_or("").to_string();
                    let title = q["title"].as_str().unwrap_or("").to_string();
                    let slug = q["titleSlug"].as_str().unwrap_or("").to_string();
                    let difficulty = q["difficulty"].as_str().unwrap_or("").to_string();
                    let paid_only = q["isPaidOnly"].as_bool().unwrap_or(false);
                    if difficulty == "Easy" && !paid_only {
                        return Ok(Some(Question { id, title, slug, difficulty, paid_only }));
                    }
                }
            }
        }
        Ok(None)
    }

    pub fn pick_daily_problem(&self, easy_file: &str, used_file: &str) -> Result<Question> {
        let easy_lines = read_lines(easy_file)?;
        let used_lines = read_lines(used_file).unwrap_or_default();
        let used_set: std::collections::HashSet<_> = used_lines.iter().collect();

        let available: Vec<_> = easy_lines.iter()
            .filter(|line| !used_set.contains(line))
            .cloned()
            .collect();

        let selected = pick_random(&available).ok_or(anyhow::anyhow!("No available EASY problems"))?;
        let parts: Vec<_> = selected.split(':').collect();
        let id = parts[0].to_string();
        let slug_title = parts[1].split('-').nth(1).unwrap_or("").to_string();  // Extract slug

        // Parse full Question (fetch details if needed, but use cached)
        append_line(used_file, &selected)?;
        Ok(Question {
            id,
            title: slug_title.clone(),  // Approximate; fetch full if critical
            slug: slug_title,
            difficulty: "Easy".to_string(),
            paid_only: false,
        })
    }

    pub async fn get_today_problem(&self, easy_file: &str, used_file: &str) -> Result<Question> {
        if let Some(daily) = self.get_daily_challenge().await? {
            let used = read_lines(used_file).unwrap_or_default();
            if !used.iter().any(|u| u.contains(&daily.id)) {
                return Ok(daily);
            }
        }
        self.pick_daily_problem(easy_file, used_file)
    }
}
```

**ToDo:**
- Add retry logic for rate limits (e.g., `tokio::time::sleep` on 429).
- Tests: Mock `Client` with `wiremock` crate (add to dev-deps).
- Handle .com query diffs: Use `if variant == "com"` to swap field names in parsing.

### 3.4 api.rs
Fetch poem, history, running stats.

```rust
use reqwest::Client;
use polars::prelude::*;
use anyhow::Result;
use serde::Deserialize;
use crate::config::Config;

#[derive(Deserialize)]
struct PoemResponse {
    data: Poem,
}

#[derive(Deserialize)]
struct Poem {
    content: String,
    author: String,
    dynasty: String,
}

pub async fn fetch_poem(client: &Client) -> Result<(String, String, String)> {
    let res: PoemResponse = client.get("https://v2.jinrishici.com/one.json")
        .send().await?.json().await?;
    Ok((res.data.content, res.data.author.to_string(), res.data.dynasty.to_string()))
}

#[derive(Deserialize)]
struct HistoryEvent {
    year: i32,
    description: String,
    pages: Vec<String>,  // For links
}

pub async fn fetch_history(client: &Client, birth_year: i32, month: u32, day: u32) -> Result<Vec<String>> {
    let url = format!("https://api.wikimedia.org/feed/v1/wikipedia/zh/onthisday/events/{}/{}/10", month, day);
    let events: Vec<HistoryEvent> = client.get(&url).send().await?.json().await?;
    let relevant = events.iter()
        .filter(|e| e.year >= birth_year)
        .take(2)
        .map(|e| {
            let age = 2026 - e.year;  // Project year
            format!("{}年{}岁：{} [详情](https://zh.wikipedia.org/wiki/{}))", e.year, age, e.description, e.pages[0])
        })
        .collect();
    Ok(relevant)
}

pub async fn fetch_running_stats(parquet_file: &str, config: &Config) -> Result<(f64, i32)> {
    // Yesterday, this month, this year distances/sessions
    let yesterday = get_local_time(config).date_naive().pred_opt().unwrap();
    let month_start = yesterday.with_day(1).unwrap();
    let year_start = yesterday.with_ordinal(1).unwrap();

    let df = LazyFrame::scan_parquet(parquet_file, ScanArgsParquet::default())?
        .filter(col("start_date_local").dt().date().eq(lit(yesterday)))
        .select([col("distance").sum()?.alias("yesterday_dist"), col("id").count().alias("yesterday_count")])
        .collect()?;

    // Similar for month/year; aggregate
    let yesterday_dist = df.column("yesterday_dist")?.f64()?.get(0).unwrap_or(0.0);
    let yesterday_count = df.column("yesterday_count")?.i64()?.get(0).unwrap_or(0) as i32;

    Ok((yesterday_dist, yesterday_count))  // km, sessions
}
```

**Notes:** 
- Running: Assume Parquet schema with `start_date_local` (date), `distance` (float km), `id`.
- History: Filter post-birth; format as Markdown links.
- ToDo: Parallel fetches with `tokio::join!`. Error on no data: fallback messages.

### 3.5 message.rs
Assemble the daily message template.

```rust
use crate::Question;
use crate::{get_year_progress, get_local_time};

pub fn build_message(
    time: &str,
    progress: &str,
    poem: &(String, String, String),
    history: &[String],
    running: (f64, i32),
    problem: &Question,
    config: &crate::config::Config,
) -> String {
    let local_time = get_local_time(config);
    let wake_msg = if local_time.hour() < 8 { "早安！" } else { "午安！" };

    format!(
        r#"{}，{}，现在是 {}。

今年进度：{}

今日诗词：
{}

跑步：昨天跑了 {:.1}km，共 {} 次。
本月/本年：... (扩展)

历史上的今天：
{}

今日 LeetCode EASY：
{} #{} {} https://leetcode.com/problems/{}/

Keep going! 🚀"#,
        wake_msg, config.birth_year + local_time.year() as i32 - 1989, time,  // Age
        progress,
        format!("{} —— {} {}", poem.0, poem.1, poem.2),
        running.0, running.1,
        join(history, "\n"),
        problem.title, problem.id, problem.slug, problem.slug
    )
}

// Helper: impl Join for Vec<String>
fn join(items: &[String], sep: &str) -> String { /* ... */ }
```

**ToDo:** Use `tera` crate for templating if complex; validate length (< GitHub limit).

### 3.6 main.rs
CLI orchestration with `clap` (add `clap = { version = "4.5", features = ["derive"] }`).

```rust
use clap::Parser;
use anyhow::Result;
use octocrab::Octocrab;

#[derive(Parser)]
#[command(name = "routine-daily")]
struct Args {
    #[arg(long)]
    fetch_easy: bool,

    #[arg(long)]
    post: bool,

    #[arg(long)]
    telegram: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = crate::config::Config::load()?;

    let client = reqwest::Client::new();
    let leet = crate::leetcode::LeetCode::new(&config);

    if args.fetch_easy {
        leet.fetch_easy_list("data/leetcode_easy.txt").await?;
        return Ok(());
    }

    // Fetch components
    let now = crate::utils::get_local_time(&config);
    let time = now.format("%H:%M").to_string();
    let progress = crate::utils::get_year_progress(&Utc::now());
    let (poem_c, poem_a, poem_d) = crate::api::fetch_poem(&client).await?;
    let month = now.month();
    let day = now.day();
    let history = crate::api::fetch_history(&client, config.birth_year, month, day).await?;
    let running = crate::api::fetch_running_stats("data/running.parquet", &config).await?;
    let problem = leet.get_today_problem("data/leetcode_easy.txt", "data/used_problems.txt").await?;

    let message = crate::message::build_message(&time, &progress, &(poem_c, poem_a, poem_d), &history, running, &problem, &config);

    // Check if posted today (fetch comments)
    let mut crab = Octocrab::builder().personal_token(config.github_token.clone()).build()?;
    let repo = crab.repos(&config.repo_owner, &config.repo_name);
    let issue = repo.get_issue(1).await?;
    let comments = repo.list_comments(1).send().await?.items;
    let today_str = now.format("%Y-%m-%d").to_string();
    let already_posted = comments.iter().any(|c| c.body.as_ref().map_or(false, |b| b.contains(&today_str)));

    if args.post && !already_posted {
        repo.create_comment(1, message.clone()).await?;
    }

    if args.telegram && config.telegram_token.is_some() {
        // Use telegram-bot
        let bot = telegram_bot::Bot::new(config.telegram_token.as_ref().unwrap());
        bot.send_message(config.telegram_chat_id.as_ref().unwrap().parse()?, message).await?;
    }

    println!("Message generated: {}", message.len());  // For dry-run
    Ok(())
}
```

**ToDo:** Add `--dry-run` flag to print message.

---

## 4. Testing & CI

### Unit Tests
Add to `tests/`:
- Mock HTTP for LeetCode/API (use `wiremock` in `[dev-dependencies]`).
- Example: `#[tokio::test] async fn test_fetch_easy() { ... }`

Run: `cargo test`.

### Integration
- Manual: `./target/release/leetcode-daily --fetch-easy`
- Daily dry-run: `cargo run -- --post=false`

### CI (GitHub Actions)
Create `.github/workflows/ci.yml`:
```yaml
name: CI
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
      - run: cargo clippy -- -D warnings
```

---

## 5. Deployment & Scheduling

### Binary Build
`cargo build --release --features telegram`

### Cron Job (Linux/macOS)
Edit crontab: `crontab -e`
```
0 8 * * * /path/to/target/release/leetcode-daily --post=true --telegram=true >> /var/log/leetcode-daily.log 2>&1
```
- 8 AM local time for wake-up.

### Monitoring
- Log errors to file.
- Alert on failures (e.g., via Telegram on panic).

### Enhancements
- Docker: Simple Dockerfile for containerized runs.
- Config for multiple repos/bots.
- Cache fetches (e.g., Redis if scaled).
- Voice mode? Integrate if xAI tools evolve.

---

## 6. Next Steps to Start Coding
1. Init project & deps (30 min).
2. Implement `config.rs` & `utils.rs` (30 min).
3. Code `leetcode.rs` & test with `--fetch-easy` (1 hr).
4. Add `api.rs` & `message.rs` (1 hr).
5. Wire `main.rs`, test posting (30 min).
6. Add Telegram if needed.
7. Schedule & monitor.

Track progress in a branch. Ping for code reviews or stuck points!

*Generated: Feb 15, 2026 – Ready to code, Muhammad!*