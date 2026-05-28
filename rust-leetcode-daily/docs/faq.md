# FAQ & Troubleshooting

This document answers frequently asked questions and provides solutions to common problems.

## Table of Contents

- [Common Errors and Solutions](#common-errors-and-solutions)
- [API Issues](#api-issues)
- [Wake-Up Detection](#wake-up-detection)
- [Problem Management](#problem-management)
- [Rate Limiting](#rate-limiting)
- [Customization](#customization)
- [LeetCode CN vs LeetCode.com](#leetcode-cn-vs-leetcodecom)

## Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `GITHUB_TOKEN must be set` | Missing environment variable | Add `GITHUB_TOKEN=ghp_xxx` to `.env` |
| `REPO_OWNER must be set` | Missing environment variable | Add `REPO_OWNER=username` to `.env` |
| `REPO_NAME must be set` | Missing environment variable | Add `REPO_NAME=repo` to `.env` |
| `BIRTH_YEAR must be set` | Missing environment variable | Add `BIRTH_YEAR=1990` to `.env` |
| `Invalid timezone: XXX` | Wrong timezone format | Use IANA format: `America/New_York` |
| `BIRTH_YEAR must be a valid year` | Non-numeric value | Use numeric year: `1990` not `nineteen ninety` |
| `Failed to fetch EASY problems` | LeetCode API unreachable | Check network, use VPN, or switch to CN |
| `No available EASY problems found` | All problems used | Reset `data/used_problems.txt` |
| `Telegram feature not enabled` | Missing compile feature | Rebuild with `--features telegram` |
| `Discord feature not enabled` | Missing compile feature | Rebuild with `--features discord` |
| `Failed to create GitHub comment` | Invalid token or permissions | Verify token has `repo` scope |
| `Failed to parse weather response` | API format changed | Check API documentation, update parser |

## API Issues

### "Failed to fetch quote" - API Down

**Problem:** The quote API (quotable.io) is unavailable or rate-limited.

```
Error: Failed to fetch quote
  Caused by: HTTP request failed
```

**Solution 1: Check API Status**

```bash
curl -I https://api.quotable.io/random
```

**Solution 2: Use Default Quote**

The application automatically falls back to a default quote:

```
The only way to do great work is to love what you do.

—— Steve Jobs
```

**Solution 3: Use Alternative Quote Source**

Edit `src/api.rs` to use a different API:

```rust
const QUOTE_API: &str = "https://api.quotable.io/random";

pub async fn fetch_quote(client: &reqwest::Client) -> Result<String> {
    let result = client
        .get(QUOTE_API)
        .timeout(Duration::from_secs(5))
        .send()
        .await?;
    // ...
}
```

**Workaround: Add Timeout**

```rust
let result = client
    .get(QUOTE_API)
    .timeout(std::time::Duration::from_secs(10))
    .send()
    .await;
```

### "Failed to fetch history" - Wikimedia API Issues

**Problem:** The Wikimedia API is unavailable.

**Solution:** The application gracefully handles this by returning an empty history list. No action required unless you need history data.

```rust
// Already handled in api.rs
let response = match client.get(&url).send().await {
    Ok(resp) => resp,
    Err(_) => return Ok(vec![]),  // Graceful fallback
};
```

### "Failed to fetch running stats" - Parquet File Issues

**Problem:** Missing or corrupted `running.parquet` file.

**Solution 1: Create Sample File**

```python
import polars as pl
from datetime import date

df = pl.DataFrame({
    "date": [date(2024, 1, 14)],
    "distance_km": [5.0]
})

df.write_parquet("data/running.parquet")
```

**Solution 2: Use Default Stats**

The application returns zeroed stats when the file is missing:

```rust
Err(_) => {
    return Ok(RunningStats {
        yesterday_km: 0.0,
        yesterday_count: 0,
        month_km: 0.0,
        month_count: 0,
        year_km: 0.0,
        year_count: 0,
    });
}
```

## Wake-Up Detection

### "Already posted today" - Expected Behavior

**What it means:** The application found a GitHub comment with today's date and skipped posting.

**Why this happens:**

```rust
// In main.rs
if issue.comments > 0 {
    let comments = octocrab
        .issues(&config.repo_owner, &config.repo_name)
        .list_comments(1)
        .send()
        .await?;

    for comment in comments {
        if comment.body.as_ref().map_or(false, |b| b.contains(&today)) {
            println!("Already posted today, skipping...");
            return Ok(());
        }
    }
}
```

**This is expected behavior** - prevents duplicate posts on the same day.

**To force a new post:**
1. Delete today's comment from GitHub Issue #1
2. Run the application again

### "You wake up late" - Wake-Up Detection Explained

**What it means:** The current time is outside the wake-up window (default: 3 AM - 9 AM).

**How wake-up detection works:**

```rust
let current_hour = now.hour();
let is_early_wake_up = current_hour >= 3 && current_hour <= 9;

if !is_early_wake_up {
    println!("You wake up late");
    println!("\n{}", message);
    return Ok(());  // Posts message but skips notifications
}
```

**The logic:**
- Hour is 0-23 (00:00 to 23:59)
- Wake-up window: 3 AM (hour 3) to 9 AM (hour 9)
- Within window: Posts to GitHub AND sends notifications
- Outside window: Only prints to console

**Visual timeline:**

```
        Wake-Up Window
        ┌─────────────────┐
────3AM─────────────9AM──────────────
    ↑               ↑
  Start            End
    
Outside window: "You wake up late" message
Inside window: Normal posting behavior
```

**Configure wake-up window:**

```bash
# .env
WAKE_UP_START=5   # 5 AM
WAKE_UP_END=10    # 10 AM
```

Note: These environment variables are defined in the configuration but not yet implemented in main.rs. See [Development Guide](./development.md) for adding this feature.

## Problem Management

### How to Reset Used Problems

**Problem:** All EASY problems have been used, or you want to start fresh.

**Solution 1: Clear the used problems file**

```bash
# Empty the file
echo "" > data/used_problems.txt

# Or delete it
rm data/used_problems.txt
```

**Solution 2: Remove specific problems**

```bash
# View used problems
cat data/used_problems.txt

# Edit to remove specific entries
vim data/used_problems.txt
```

**Solution 3: Refresh EASY problem list**

```bash
# Re-fetch all EASY problems from LeetCode
cargo run -- --fetch-easy
```

**File format:**

```
data/used_problems.txt
┌─────────────────────────┐
│ two-sum                 │
│ valid-parentheses       │
│ merge-two-sorted-lists  │
│ ...                     │
└─────────────────────────┘

data/leetcode_easy.txt
┌─────────────────────────────────────────────┐
│ 1|Two Sum|two-sum                           │
│ 20|Valid Parentheses|valid-parentheses      │
│ 21|Merge Two Sorted Lists|merge-two-sorted-lists │
│ ...                                         │
└─────────────────────────────────────────────┘
```

### How Problems Are Selected

```rust
// 1. Try official daily challenge
if let Some(daily) = get_daily_challenge().await? {
    if daily.difficulty == "EASY" && !daily.paid_only {
        if !is_used(&daily.slug) {
            return Ok(daily);
        }
    }
}

// 2. Fallback to seeded random from EASY pool
let seed = year * 1000 + day_of_year;
let problem = seeded_random_pick(available, seed);
```

**Key points:**
- Daily challenge is prioritized if it's EASY and free
- Seeded random ensures same problem for the same day
- Problems are tracked by slug (URL-safe identifier)

## Rate Limiting

### GitHub API Rate Limits

**Limits:**
- Authenticated: 5,000 requests/hour
- Unauthenticated: 60 requests/hour

**Check your rate limit:**

```bash
curl -H "Authorization: token $GITHUB_TOKEN" \
  https://api.github.com/rate_limit
```

**Response:**

```json
{
  "rate": {
    "limit": 5000,
    "remaining": 4999,
    "reset": 1708060800
  }
}
```

**Solutions:**

1. **Use authenticated requests** (already done with `GITHUB_TOKEN`)
2. **Reduce API calls** - the app checks for existing comments before posting
3. **Cache results** for repeated runs

### LeetCode API Rate Limits

**Problem:** Too many requests to LeetCode GraphQL API.

```
Error: HTTP 429 Too Many Requests
```

**Solutions:**

1. **Add delays between requests:**

```rust
use tokio::time::{sleep, Duration};

sleep(Duration::from_secs(1)).await;
```

2. **Use LeetCode CN as alternative:**

```bash
LEETCODE_ENDPOINT=https://leetcode.cn/graphql/
LEETCODE_VARIANT=cn
```

3. **Cache EASY problems locally:**

```bash
# Fetch once and reuse
cargo run -- --fetch-easy
```

### Telegram API Rate Limits

**Limits:**
- 30 messages/second to same chat
- 20 messages/minute to group

**Solution:** The app sends only one message per run, well within limits.

### Discord API Rate Limits

**Limits:**
- 50 requests/second (global)
- 5 requests/5 seconds (per channel)

**Solution:** Single message per run is safe.

## Customization

### How to Force a Post Outside Wake-Up Hours

**Problem:** You want to post even when it's not "early morning".

**Solution 1: Modify the code**

Edit `src/main.rs`:

```rust
// Change this condition
if !is_early_wake_up {
    println!("You wake up late");
    println!("\n{}", message);
    return Ok(());
}

// To this:
if !is_early_wake_up {
    println!("Posting outside wake-up hours...");
    // Continue to post instead of returning
}
```

**Solution 2: Add a force flag**

Add to CLI arguments:

```rust
#[derive(Parser, Debug)]
struct Args {
    // ... existing args
    
    #[arg(long, help = "Force posting outside wake-up hours")]
    force: bool,
}
```

Then modify the logic:

```rust
if !is_early_wake_up && !args.force {
    println!("You wake up late");
    println!("\n{}", message);
    return Ok(());
}
```

Usage:

```bash
cargo run -- --post --force
```

### How to Add Custom Quotes

**Option 1: Add to fallback quotes**

Edit `src/api.rs`:

```rust
const DEFAULT_QUOTES: &[&str] = &[
    "The only way to do great work is to love what you do.\n\n—— Steve Jobs",
    "Code is like humor. When you have to explain it, it's bad.\n\n—— Cory House",
    "First, solve the problem. Then, write the code.\n\n—— John Johnson",
];

pub async fn fetch_quote(client: &reqwest::Client) -> Result<String> {
    let result = client
        .get("https://api.quotable.io/random")
        .send()
        .await?
        .json::<QuoteResponse>()
        .await;

    match result {
        Ok(response) => Ok(format!("{}\n\n—— {}", response.content, response.author)),
        Err(_) => {
            // Use random quote from defaults
            let index = rand::thread_rng().gen_range(0..DEFAULT_QUOTES.len());
            Ok(DEFAULT_QUOTES[index].to_string())
        }
    }
}
```

**Option 2: Load from file**

```rust
pub async fn fetch_custom_quotes(file: &str) -> Result<Vec<String>> {
    let content = tokio::fs::read_to_string(file).await?;
    let quotes: Vec<String> = content
        .split("\n---\n")
        .map(|s| s.to_string())
        .collect();
    Ok(quotes)
}
```

Create `data/quotes.txt`:

```
The only way to do great work is to love what you do.

—— Steve Jobs
---
Code is like humor. When you have to explain it, it's bad.

—— Cory House
---
First, solve the problem. Then, write the code.

—— John Johnson
```

**Option 3: Use alternative quote API**

```rust
const QUOTE_APIS: &[&str] = &[
    "https://api.quotable.io/random",
    "https://zenquotes.io/api/random",
    "https://quotes.rest/qod",
];
```

### How to Change Wake-Up Time Range

Currently, the wake-up time is hardcoded. Here's how to make it configurable:

**Step 1: Add to Config**

Edit `src/config.rs`:

```rust
#[derive(Debug, Clone)]
pub struct Config {
    // ... existing fields
    pub wake_up_start: u32,
    pub wake_up_end: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        // ... existing code
        
        let wake_up_start: u32 = env::var("WAKE_UP_START")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(3);
        
        let wake_up_end: u32 = env::var("WAKE_UP_END")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(9);
        
        Ok(Config {
            // ... existing fields
            wake_up_start,
            wake_up_end,
        })
    }
}
```

**Step 2: Use in main.rs**

```rust
let is_early_wake_up = current_hour >= config.wake_up_start 
                     && current_hour <= config.wake_up_end;
```

**Step 3: Configure in .env**

```bash
# Wake-Up Detection
WAKE_UP_START=5   # 5 AM
WAKE_UP_END=10    # 10 AM
```

## LeetCode CN vs LeetCode.com

### Key Differences

| Feature | LeetCode.com | LeetCode.cn |
|---------|--------------|-------------|
| URL | `leetcode.com` | `leetcode.cn` |
| GraphQL Endpoint | `https://leetcode.com/graphql/` | `https://leetcode.cn/graphql/` |
| Problem URLs | `leetcode.com/problems/two-sum/` | `leetcode.cn/problems/two-sum/` |
| Availability | Global | China-optimized |
| Daily Challenge | Same problems | Same problems |
| API Access | May require VPN in China | Direct access in China |

### Configuration

**LeetCode.com (default):**

```bash
LEETCODE_ENDPOINT=https://leetcode.com/graphql/
LEETCODE_VARIANT=com
```

**LeetCode CN:**

```bash
LEETCODE_ENDPOINT=https://leetcode.cn/graphql/
LEETCODE_VARIANT=cn
```

### Code Implementation

The variant affects problem URLs:

```rust
pub fn format_problem_message(problem: &Question, variant: &LeetCodeVariant) -> String {
    let url = match variant {
        LeetCodeVariant::Cn => format!("https://leetcode.cn/problems/{}/", problem.slug),
        LeetCodeVariant::Com => format!("https://leetcode.com/problems/{}/", problem.slug),
    };
    // ...
}
```

### Problem ID Consistency

Both platforms use the same problem IDs and slugs:

```
Problem #1: Two Sum
Slug: two-sum
URL (com): https://leetcode.com/problems/two-sum/
URL (cn):  https://leetcode.cn/problems/two-sum/
```

**Note:** Premium problems may differ in availability between platforms.

### Switching Between Platforms

Simply change the environment variables:

```bash
# Switch to LeetCode CN
export LEETCODE_ENDPOINT=https://leetcode.cn/graphql/
export LEETCODE_VARIANT=cn

# Or switch back to LeetCode.com
export LEETCODE_ENDPOINT=https://leetcode.com/graphql/
export LEETCODE_VARIANT=com
```

Rebuild not required - configuration is loaded at runtime.

### Troubleshooting Cross-Platform Issues

**Problem:** "Failed to fetch EASY problems" on one platform.

**Solution:** Switch to the other platform temporarily:

```bash
# If leetcode.com is blocked
LEETCODE_VARIANT=cn cargo run -- --fetch-easy

# Then switch back for posting
LEETCODE_VARIANT=com cargo run -- --post
```

**Problem:** Different daily challenge on each platform.

**Solution:** This is rare but possible. The app will use whichever daily challenge is available.

## Additional Resources

- [Getting Started Guide](./getting-started.md) - Initial setup
- [Configuration Guide](./configuration.md) - All configuration options
- [Architecture Overview](./architecture.md) - System design
- [Development Guide](./development.md) - Contributing and extending

## Getting Help

1. Check this FAQ for common issues
2. Review the [Configuration Guide](./configuration.md)
3. Search existing GitHub issues
4. Open a new issue with:
   - Error message (full stack trace)
   - Your `.env` configuration (redact secrets)
   - Steps to reproduce
   - Operating system and Rust version
